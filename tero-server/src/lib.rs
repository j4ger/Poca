use dyn_clone::DynClone;
use futures_channel::oneshot;
use futures_util::StreamExt;
use parking_lot::{Mutex, RwLock};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    any::Any, collections::HashMap, fmt::Debug, marker::PhantomData, net::SocketAddr, ops::Deref,
    sync::Arc,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast,
    task::JoinHandle,
};

const CHANNEL_SIZE: usize = 32;

type DataElement = Arc<RwLock<Box<dyn Any + Send + Sync>>>;
type Store = Arc<Mutex<HashMap<String, DataElement>>>;

type BroadcastSender<'a> = broadcast::Sender<Message<'a>>;
type BroadcastReceiver<'a> = broadcast::Receiver<Message<'a>>;

pub struct Tero<'a> {
    state: ServerState,
    addr: SocketAddr,
    server_handle: Option<JoinHandle<()>>,
    handler_handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
    store: Store,
    broadcast: (BroadcastSender<'a>, BroadcastReceiver<'a>),
}

pub struct DataHandle<'a, T: Synchronizable> {
    key: String,
    sender: broadcast::Sender<Message<'a>>,
    data_type: PhantomData<T>,
    data: DataElement,
}

pub trait DataToAny: 'static {
    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static> DataToAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<'a, T> DataHandle<'a, T>
where
    T: Synchronizable,
{
    pub fn set(&'a self, value: T) {
        let mut guard = self.data.write();
        *guard = value.clone_any_box();
        let request = Message::Set {
            key: &self.key,
            data: Box::new(value),
        };
        self.sender.send(request).unwrap();
    }

    pub fn execute<F, R>(&self, action: F) -> R
    where
        F: Fn(&T) -> R,
    {
        let guard = self.data.read();
        let data = guard.deref().downcast_ref::<T>().unwrap();
        action(data)
    }

    pub fn get(&self) -> Box<T> {
        let guard = self.data.read();
        guard
            .deref()
            .downcast_ref::<T>()
            .unwrap()
            .clone_any_box()
            .downcast()
            .unwrap()
    }
}

pub trait SynchronizableClone {
    fn clone_any_box(&self) -> Box<dyn Any + Send + Sync>;
}

pub trait Synchronizable: 'static + Sync + Send + Debug + DynClone + SynchronizableClone {}

impl<T: 'static + Synchronizable + Clone> SynchronizableClone for T {
    fn clone_any_box(&self) -> Box<dyn Any + Send + Sync> {
        Box::new(self.clone())
    }
}

impl<T> Synchronizable for T where T: 'static + Sync + Send + Debug + Clone {}

dyn_clone::clone_trait_object!(Synchronizable);

#[derive(Debug, Clone)]
pub enum Message<'a> {
    Set {
        key: &'a str,
        data: Box<dyn Synchronizable>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WSMessageType {
    Set = 1,
    Emit = 2,
    Get = 3,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WSMessage {
    message_type: WSMessageType,
    key: String,
    data: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    Up,
    Down,
}

impl Tero<'static> {
    pub fn new(addr: SocketAddr) -> Tero<'static> {
        let channel = broadcast::channel(CHANNEL_SIZE);
        Tero {
            state: ServerState::Down,
            addr,
            server_handle: None,
            handler_handles: Arc::new(Mutex::new(Vec::new())),
            store: Arc::new(Mutex::new(HashMap::new())),
            broadcast: channel,
        }
    }

    pub fn get_state(&self) -> ServerState {
        self.state
    }

    pub async fn start(&mut self) {
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
        self.server_handle = Some(server_handle);
        self.state = ServerState::Up;
    }

    pub fn stop(&mut self) {
        if self.state == ServerState::Up {
            for each in &(*(self.handler_handles.lock())) {
                each.abort();
            }
            self.handler_handles = Arc::new(Mutex::new(Vec::new()));
            self.server_handle.take().unwrap().abort();
            self.state = ServerState::Down;
        }
    }
}

async fn websocket_handler<'a>(
    raw_stream: TcpStream,
    addr: SocketAddr,
    store: Store,
    broadcast_rx: BroadcastReceiver<'a>,
) -> () {
    println!("New connection from {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Failed to accept websocket");
    let (mut ws_sender, ws_receiver) = ws_stream.split();

    //TODO: deal with incoming messages
}
