mod app_routes;
mod data_handle;
mod event_handler;
mod message;
mod poca;
mod serve_app;
mod synchronizable;
mod ws_handler;

pub use app_routes::AppRoutes as _AppRoutes;
pub use data_handle::DataHandle;
pub use poca::Poca;

// macro-related functions
pub use app_routes::generate_app_routes as _g_a_r;
pub use app_routes::AppRoutes as _AR;
pub use app_routes::RouteNode as _N;

// probably should be in a common module
pub use message::{WSMessage as _WSMessage, WSMessageType as _WSMessageType};

//TODO: remove unnecessary pubs
//TODO: use SNAFU for error handling
