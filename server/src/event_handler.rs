use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;

pub type OnChangeEventHandlerStore<T> = Arc<RwLock<Vec<Box<dyn FnMut(T) + Send + Sync + 'static>>>>;
pub type EventHandlerStore =
    Arc<RwLock<HashMap<String, Vec<Box<dyn Fn() + Send + Sync + 'static>>>>>;

pub trait EventHandler: Send + Sync + 'static {
    fn execute(&self);
}

impl EventHandler for dyn Fn() + Send + Sync {
    fn execute(&self) {
        self()
    }
}
trait AsEventHandler {
    fn as_event_handler(self) -> Box<dyn EventHandler>;
}

impl<F> AsEventHandler for F
where
    F: EventHandler,
{
    fn as_event_handler(self) -> Box<dyn EventHandler> {
        Box::new(self)
    }
}
