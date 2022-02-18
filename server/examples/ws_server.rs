#[macro_use]
extern crate lazy_static;

use poca_macro::include_app_dir;
use poca_server::Poca;

lazy_static! {
    static ref POCA: Poca = Poca::new("localhost:1120", include_app_dir!("examples/resources/"));
}

#[tokio::main]
async fn main() {
    let _handle = POCA.data("entry1", 42);
    println!("Starting websocket server");
    POCA.start().await;

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to register CTRL-C handler");
    println!("CTRL-C received, shutting down");
}
