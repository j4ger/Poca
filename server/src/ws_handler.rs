use std::ops::Deref;

use futures_util::pin_mut;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use warp::ws::{self, WebSocket};

use crate::{
    message::{Message, WSMessage, WSMessageType},
    poca::{BroadcastReceiver, BroadcastSender, Store},
};

pub async fn websocket_handler<'a>(
    websocket: WebSocket,
    store: Store,
    broadcast_receiver: BroadcastReceiver,
    broadcast_sender: BroadcastSender,
) {
    let (ws_sender, ws_receiver) = futures_util::StreamExt::split(websocket);

    //TODO: handshake, but let's skip it until basic frontend is done

    let broadcast_stream = BroadcastStream::from(broadcast_receiver);
    let broadcast_dealer = futures_util::StreamExt::forward(
        broadcast_stream.filter_map(|message| {
            match message {
                Ok(inner) => match inner {
                    Message::Set { key, data } => Some(Ok(ws::Message::text(
                        serde_json::to_string(&WSMessage {
                            message_type: WSMessageType::Set,
                            key: Some(key.to_string()),
                            data: Some(data.serialize()),
                        })
                        .unwrap(),
                    ))),
                    Message::Get { key, data } => Some(Ok(ws::Message::text(
                        serde_json::to_string(&WSMessage {
                            message_type: WSMessageType::Get,
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
        //TODO: use bytes instead of string
        let text = message.to_str().unwrap();
        println!("Got Websocket message: {:?}", &text);
        let message: WSMessage = serde_json::from_str(text).unwrap();
        match message.message_type {
            WSMessageType::Set => {
                let key = message.key.unwrap();
                let store_lock = store.lock();
                let element_entry = store_lock.get(&key).unwrap();
                let element = element_entry.deref();
                let new_data;
                {
                    let handle = element.read();
                    new_data = handle.data.deserialize(message.data.unwrap().as_str());
                }
                {
                    let mut handle = element.write();
                    handle.data = new_data;
                }
                //TODO: emit events
                {
                    let handle = element.read();
                    for each in handle.on_change.deref() {
                        let handler = each.deref();
                        handler()
                    }
                }
            }
            WSMessageType::Get => {
                let key = message.key.unwrap();
                let data;
                {
                    let store_lock = store.lock();
                    let element_entry = store_lock.get(&key).unwrap();
                    let element = element_entry.deref();
                    let handle = element.read();
                    data = handle.data.serialize();
                }
                broadcast_sender
                    .send(Message::Get {
                        key,
                        data: Box::new(data),
                    })
                    .ok();
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
