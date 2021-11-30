use dyn_clone::DynClone;
use futures_channel::oneshot;
use futures_util::StreamExt;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    any::Any,
    collections::HashMap,
    fmt::Debug,
    marker::PhantomData,
    net::SocketAddr,
    sync::{Arc, Mutex, RwLock},
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast,
    task::JoinHandle,
};

const CHANNEL_SIZE: usize = 32;

pub struct Tero<'a> {
    state: ServerState,
    addr: SocketAddr,
    server_handle: Option<JoinHandle<()>>,
    handler_handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
    store: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
    broadcast: (
        broadcast::Sender<Message<'a>>,
        broadcast::Receiver<Message<'a>>,
    ),
}

pub struct DataHandle<'a, T: Synchronizable> {
    key: String,
    sender: broadcast::Sender<Message<'a>>,
    data_type: PhantomData<T>,
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
        let request = Message::Set {
            key: &self.key,
            data: Box::new(value),
        };
        self.sender.send(request).unwrap();
    }
}

pub trait Synchronizable: 'static + Sync + Send + Debug + DynClone {}

impl<T> Synchronizable for T where T: 'static + Sync + Send + Debug + Clone {}

dyn_clone::clone_trait_object!(Synchronizable);

#[derive(Debug, Clone)]
pub enum Message<'a> {
    Set {
        key: &'a str,
        data: Box<dyn Synchronizable>,
    },
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
            store: Arc::new(RwLock::new(HashMap::new())),
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
        let server_handle = tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                let store_clone = store.clone();
                let new_handler = tokio::spawn(websocket_handler(stream, addr, store_clone));
                handler_handles.lock().unwrap().push(new_handler);
            }
        });
        self.server_handle = Some(server_handle);
        self.state = ServerState::Up;
    }

    pub fn stop(&mut self) {
        if self.state == ServerState::Up {
            for each in &(*(self.handler_handles.lock().unwrap())) {
                each.abort();
            }
            self.handler_handles = Arc::new(Mutex::new(Vec::new()));
            self.server_handle.take().unwrap().abort();
            self.state = ServerState::Down;
        }
    }
}

async fn websocket_handler(
    raw_stream: TcpStream,
    addr: SocketAddr,
    store: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
) -> () {
    println!("New connection from {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Failed to accept websocket");
    let (mut ws_sender, ws_receiver) = ws_stream.split();

    //TODO: deal with incoming messages
}
