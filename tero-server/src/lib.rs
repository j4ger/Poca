use dyn_clone::DynClone;
use futures_util::pin_mut;
use parking_lot::{Mutex, RwLock};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    any::Any,
    collections::HashMap,
    fmt::Debug,
    marker::PhantomData,
    net::{SocketAddr, ToSocketAddrs},
    ops::Deref,
    sync::Arc,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast,
    task::JoinHandle,
};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

const CHANNEL_SIZE: usize = 32;

type DataElement = Arc<RwLock<Box<dyn Synchronizable>>>;
type Store = Arc<Mutex<HashMap<String, DataElement>>>;

type BroadcastSender = broadcast::Sender<Message>;
type BroadcastReceiver = broadcast::Receiver<Message>;

pub struct Tero {
    state: ServerState,
    addr: SocketAddr,
    server_handle: Option<JoinHandle<()>>,
    handler_handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
    store: Store,
    broadcast: (BroadcastSender, BroadcastReceiver),
}

pub struct DataHandle<T: Synchronizable> {
    key: String,
    sender: broadcast::Sender<Message>,
    data_type: PhantomData<T>,
    data: DataElement,
}

pub trait DataToAny: 'static {
    fn as_any(&self) -> &dyn Any;
    fn to_any(self) -> Box<dyn Any + Send + Sync>;
}

impl<T: 'static + Send + Sync> DataToAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_any(self) -> Box<dyn Any + Send + Sync> {
        Box::new(self)
    }
}

impl<T> DataHandle<T>
where
    T: Synchronizable,
{
    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn set(&self, value: T) {
        let mut guard = self.data.write();
        *guard = value.clone_synchronizable();
        let request = Message::Set {
            key: self.key.to_owned(),
            data: Box::new(value),
        };
        self.sender.send(request).unwrap();
    }

    pub fn get(&self) -> Box<T> {
        let guard = self.data.read();
        guard.deref().deref().clone_any_box().downcast().unwrap()
    }
}

pub trait SynchronizableClone {
    fn clone_any_box(&self) -> Box<dyn Any>;
    fn clone_synchronizable(&self) -> Box<dyn Synchronizable>;
}

pub trait Synchronizable: 'static + Sync + Send + Debug + DynClone + SynchronizableClone {
    fn serialize(&self) -> String;
    fn deserialize(&self, data: &str) -> Box<dyn Synchronizable>;
}

impl<T: 'static + Synchronizable + Clone> SynchronizableClone for T {
    fn clone_any_box(&self) -> Box<dyn Any> {
        Box::new(self.clone())
    }

    fn clone_synchronizable(&self) -> Box<dyn Synchronizable> {
        Box::new(self.clone())
    }
}

impl<T> Synchronizable for T
where
    T: 'static + Sync + Send + Debug + Clone + Serialize + DeserializeOwned,
{
    fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn deserialize(&self, data: &str) -> Box<dyn Synchronizable> {
        let data: T = serde_json::from_str(data).unwrap();
        Box::new(data)
    }
}

dyn_clone::clone_trait_object!(Synchronizable);

#[derive(Debug, Clone)]
pub enum Message {
    Set {
        key: String,
        data: Box<dyn Synchronizable>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WSMessageType {
    Set = 1,
    Emit = 2,
    Get = 3,
    Error = 4,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WSMessage {
    message_type: WSMessageType,
    key: Option<String>,
    data: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    Up,
    Down,
}

impl Tero {
    pub fn data<T: Synchronizable>(&'static self, key: &str, data: T) -> DataHandle<T> {
        let guard = self.store.lock();
        if guard.contains_key(key) {
            panic!("Key {} already exists", key);
        }
        let data = Arc::new(RwLock::new(data.clone_synchronizable()));
        let sender = self.broadcast.0.clone();
        DataHandle {
            key: key.to_string(),
            sender,
            data_type: PhantomData::<T>,
            data,
        }
    }

    pub fn new(addr: impl ToSocketAddrs) -> Tero {
        let channel = broadcast::channel(CHANNEL_SIZE);
        Tero {
            state: ServerState::Down,
            addr: addr.to_socket_addrs().unwrap().next().unwrap(),
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
    broadcast_rx: BroadcastReceiver,
) {
    println!("New connection from {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Failed to accept websocket");
    let (ws_sender, ws_receiver) = futures_util::StreamExt::split(ws_stream);

    //TODO: deal with incoming messages

    let broadcast_stream = BroadcastStream::from(broadcast_rx);
    let broadcast_dealer = futures_util::StreamExt::forward(
        broadcast_stream.filter_map(|message| {
            match message {
                Ok(inner) => match inner {
                    Message::Set { key, data } => Some(Ok(tungstenite::Message::Text(
                        serde_json::to_string(&WSMessage {
                            message_type: WSMessageType::Set,
                            key: Some(key.to_string()),
                            data: Some(data.serialize()),
                        })
                        .unwrap(),
                    ))),
                },
                Err(error) => {
                    //TODO: uniformed logging
                    println!("Error when receiving from broadcast channel: {}", error);
                    None
                }
            }
        }),
        ws_sender,
    );

    let ws_dealer = futures_util::TryStreamExt::try_for_each(ws_receiver, |message| {
        //TODO: actually do something
        println!("{:?}", message);
        let message: WSMessage =
            serde_json::from_str(message.into_text().unwrap().as_str()).unwrap();
        match message.message_type {
            WSMessageType::Set => {
                let key = message.key.unwrap();
                let lock = store.lock();
                let element = lock.get(&key).unwrap().deref();
                let mut handle = element.deref().write();
                let new_data = handle.deserialize(message.data.unwrap().as_str());
                *handle = new_data;
                //TODO: emit events
            }
            _ => {
                todo!("handle other message types")
            }
        }
        futures_util::future::ok(())
    });

    pin_mut!(broadcast_dealer, ws_dealer);
    //TODO: future::select on the dealers
    tokio::select! {
        _ = broadcast_dealer => {},
        _ = ws_dealer => {},
    }
}
