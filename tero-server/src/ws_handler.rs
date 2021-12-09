use std::net::SocketAddr;
use std::ops::Deref;

use futures_util::pin_mut;
use tokio::net::TcpStream;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use crate::{
    message::{Message, WSMessage, WSMessageType},
    tero::{BroadcastReceiver, Store},
};

pub async fn websocket_handler<'a>(
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
        //TODO: uniformed logging
        println!("{:?}", message);
        let message: WSMessage =
            serde_json::from_str(message.into_text().unwrap().as_str()).unwrap();
        match message.message_type {
            WSMessageType::Set => {
                let key = message.key.unwrap();
                let store_lock = store.lock();
                let element_entry = store_lock.get(&key).unwrap();
                let element = element_entry.deref();
                {
                    let mut handle = element.deref().data.write();
                    let new_data = handle.deserialize(message.data.unwrap().as_str());
                    *handle = new_data;
                }
                //TODO: emit events
                let on_change = element.deref().on_change.read();
                for each in on_change.deref() {
                    let handler = each.deref();
                    handler()
                }
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
