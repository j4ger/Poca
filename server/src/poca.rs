use std::{
    collections::HashMap,
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
};

use parking_lot::{Mutex, RwLock};
use tokio::{
    sync::{broadcast, oneshot},
    task::JoinHandle,
};
use warp::{path::FullPath, Filter};

use crate::{
    app_routes::AppRoutes, data_handle::DataHandle, message::Message,
    synchronizable::Synchronizable, ws_handler::websocket_handler,
};

const CHANNEL_SIZE: usize = 32;

pub struct DataElementInner {
    pub data: Box<dyn Synchronizable>,
    pub on_change: Vec<Box<dyn Fn() + Send + Sync>>,
}

pub type DataElement = Arc<RwLock<DataElementInner>>;
pub type Store = Arc<Mutex<HashMap<String, DataElement>>>;

pub type BroadcastSender = broadcast::Sender<Message>;
pub type BroadcastReceiver = broadcast::Receiver<Message>;

pub struct Poca {
    state: Mutex<ServerState>,
    address: SocketAddr,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
    store: Store,
    broadcast: (BroadcastSender, BroadcastReceiver),
    server: Mutex<Option<JoinHandle<()>>>,
    app_routes: AppRoutes<'static>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    Up,
    Down,
}

impl Poca {
    pub fn data<T: Synchronizable>(&'static self, key: &str, data: T) -> DataHandle<T> {
        let mut guard = self.store.lock();
        if guard.contains_key(key) {
            panic!("Key {} already exists", key);
        }
        let data = Arc::new(RwLock::new(DataElementInner {
            data: data.clone_synchronizable(),
            on_change: Vec::new(),
        }));
        guard.insert(key.to_string(), data.clone());
        let sender = self.broadcast.0.clone();
        DataHandle::new(key.to_string(), sender, data)
    }

    pub fn new(address: impl ToSocketAddrs, app_routes: AppRoutes<'static>) -> Poca {
        let channel = broadcast::channel(CHANNEL_SIZE);
        Poca {
            state: Mutex::new(ServerState::Down),
            address: address.to_socket_addrs().unwrap().next().unwrap(),
            shutdown: Mutex::new(None),
            store: Arc::new(Mutex::new(HashMap::new())),
            broadcast: channel,
            server: Mutex::new(None),
            app_routes,
        }
    }

    pub fn get_state(&self) -> ServerState {
        self.state.lock().clone()
    }

    pub async fn start(&'static self) {
        let (shutdown_sender, shutdown_receiver) = oneshot::channel();

        let routes = warp::get().and(
            warp::any()
                .and(warp::ws().map(|websocket: warp::ws::Ws| {
                    let store = self.store.clone();
                    let broadcast_receiver = self.broadcast.0.subscribe();
                    websocket.on_upgrade(|websocket| {
                        websocket_handler(websocket, store, broadcast_receiver)
                    })
                }))
                .or(warp::any()
                    .and(warp::path::full())
                    .map(move |path: FullPath| {
                        let path = path
                            .as_str()
                            .trim_start_matches('/')
                            .split("/")
                            .collect::<Vec<&str>>();
                        warp::reply::html(self.app_routes.get_route(&path).unwrap_or(&[]))
                    })),
        );

        let address = self.address.clone();

        *(self.server.lock()) = Some(tokio::spawn(async move {
            let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(address, async {
                shutdown_receiver.await.ok();
            });
            server.await;
        }));

        *(self.shutdown.lock()) = Some(shutdown_sender);
        *(self.state.lock()) = ServerState::Up;
    }

    pub fn stop(&self) {
        if *(self.state.lock()) == ServerState::Up {
            if let Some(sender) = self.shutdown.lock().take() {
                sender.send(());
            }
            *(self.state.lock()) = ServerState::Down;
        }
    }
}

impl Drop for Poca {
    fn drop(&mut self) {
        self.stop();
    }
}
