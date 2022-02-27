use crate::{
    event_handler::{EventHandler, EventHandlerFn},
    message::Message,
    poca::DataElement,
    synchronizable::Synchronizable,
};
use parking_lot::RwLock;
use std::{marker::PhantomData, ops::Deref, sync::Arc};
use tokio::sync::broadcast;

pub struct DataHandle<T>
where
    T: Synchronizable + 'static,
{
    key: String,
    sender: broadcast::Sender<Message>,
    data_type: PhantomData<T>,
    data_element: DataElement,
    on_change: EventHandlerFn<T>,
}

impl<T> DataHandle<T>
where
    T: Synchronizable + 'static,
{
    pub fn new(key: String, sender: broadcast::Sender<Message>, data_element: DataElement) -> Self {
        Self {
            key,
            sender,
            data_type: PhantomData,
            data_element,
            on_change: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn set(&self, value: T) {
        {
            let mut guard = self.data_element.write();
            guard.data = value.clone_synchronizable();
        }
        {
            let handle = self.data_element.read();
            for each in &handle.on_change {
                let handler = each.deref();
                handler.execute();
            }
        }
        let request = Message::Set {
            key: self.key.to_owned(),
            data: Box::new(value),
        };
        self.sender.send(request).unwrap();
    }

    pub fn get(&self) -> Box<T> {
        let guard = self.data_element.read();
        guard.data.clone_any_box().downcast().unwrap()
    }

    pub fn on_change(&'static self, handler: impl Fn(T) + Send + Sync + 'static) {
        let boxed_handler = Box::new(handler);
        let mut lock = self.on_change.write();
        let current_index = lock.len();
        lock.push(boxed_handler);
        let new_handler = move || {
            let mut guard = self.on_change.write();
            let handler = guard.get_mut(current_index).unwrap();
            let value = self.get();
            (*handler)(*value);
        };
        let dyn_handler = Box::new(new_handler) as Box<dyn Fn() + Send + Sync>;
        let mut guard = self.data_element.write();
        guard.on_change.push(dyn_handler);
    }
}
