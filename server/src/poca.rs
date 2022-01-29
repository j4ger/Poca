use std::{
    collections::HashMap,
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
};

use parking_lot::{Mutex, RwLock};
use tokio::{net::TcpListener, sync::broadcast, task::JoinHandle};

use crate::{
    data_handle::DataHandle, message::Message, synchronizable::Synchronizable,
    ws_handler::websocket_handler,
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
    addr: SocketAddr,
    server_handle: Mutex<Option<JoinHandle<()>>>,
    handler_handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
    store: Store,
    broadcast: (BroadcastSender, BroadcastReceiver),
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

    pub fn new(addr: impl ToSocketAddrs) -> Poca {
        let channel = broadcast::channel(CHANNEL_SIZE);
        Poca {
            state: Mutex::new(ServerState::Down),
            addr: addr.to_socket_addrs().unwrap().next().unwrap(),
            server_handle: Mutex::new(None),
            handler_handles: Arc::new(Mutex::new(Vec::new())),
            store: Arc::new(Mutex::new(HashMap::new())),
            broadcast: channel,
        }
    }

    pub fn get_state(&self) -> ServerState {
        self.state.lock().clone()
    }

    pub async fn start(&self) {
        let socket = TcpListener::bind(self.addr).await;
        let listener = socket.expect("Failed to bind addr.");
        let store = self.store.clone();
        let handler_handles = self.handler_handles.clone();
        let broadcast_sender = self.broadcast.0.clone();
        let server_handle = tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                let store_clone = store.clone();
                let broadcast_receiver = broadcast_sender.subscribe();
                let new_handler = tokio::spawn(websocket_handler(
                    stream,
                    addr,
                    store_clone,
                    broadcast_receiver,
                ));
                handler_handles.lock().push(new_handler);
            }
        });
        *(self.server_handle.lock()) = Some(server_handle);
        *(self.state.lock()) = ServerState::Up;
    }

    pub fn stop(&self) {
        if *(self.state.lock()) == ServerState::Up {
            for each in &(*(self.handler_handles.lock())) {
                each.abort();
            }
            self.handler_handles.lock().clear();
            (*(self.server_handle.lock())).take().unwrap().abort();
            *(self.state.lock()) = ServerState::Down;
        }
    }
}

impl Drop for Poca {
    fn drop(&mut self) {
        self.stop();
    }
}
