#[macro_use]
extern crate lazy_static;
use unroll_server::{DataHandle, Unroll, WSMessage};

lazy_static! {
    static ref UNROLL: Unroll = Unroll::new("localhost:1120");
}

#[tokio::main]
async fn main() {
    let message = WSMessage {
        message_type: unroll_server::WSMessageType::Set,
        key: Some("entry1".to_owned()),
        data: Some("43".to_owned()),
    };
    println!(
        "example message: {:?}",
        serde_json::to_string(&message).unwrap()
    );
    let handle = UNROLL.data("entry1", 42);
    println!("Starting websocket server");
    UNROLL.start().await;

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to register CTRL-C handler");
    println!("CTRL-C received, shutting down");
}
