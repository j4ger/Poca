mod data_handle;
mod event_handler;
mod message;
mod serve_app;
mod synchronizable;
mod type_gen;
mod unroll;
mod ws_handler;

pub use data_handle::DataHandle;
pub use event_handler::EventHandler;
pub use unroll::Unroll;

// probably should be in a common module
pub use message::{WSMessage, WSMessageType};

//TODO: remove unnecessary pubs
//TODO: use SNAFU for error handling
