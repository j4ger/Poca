use crate::{
    event_handler::EventHandler, message::Message, synchronizable::Synchronizable,
    tero::DataElement,
};
use parking_lot::RwLock;
use std::{marker::PhantomData, ops::Deref, sync::Arc};
use tokio::sync::broadcast;

pub struct DataHandle<T>
where
    T: Synchronizable,
{
    pub key: String,
    pub sender: broadcast::Sender<Message>,
    pub data_type: PhantomData<T>,
    pub data_element: DataElement,
    pub on_change: Arc<RwLock<Vec<Box<dyn FnMut(&T) + Send + Sync + 'static>>>>,
}

impl<T> DataHandle<T>
where
    T: Synchronizable,
{
    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn set(&self, value: T) {
        {
            let mut guard = self.data_element.data.write();
            *guard = value.clone_synchronizable();
        }
        let on_change = self.data_element.on_change.read();
        for each in on_change.deref() {
            let handler = each.deref();
            handler.execute();
        }
        let request = Message::Set {
            key: self.key.to_owned(),
            data: Box::new(value),
        };
        self.sender.send(request).unwrap();
    }

    pub fn get(&self) -> Box<T> {
        let guard = self.data_element.data.read();
        guard.deref().deref().clone_any_box().downcast().unwrap()
    }

    pub fn on_change(&'static self, handler: impl Fn(&T) -> () + Send + Sync + 'static) {
        let boxed_handler = Box::new(handler);
        let mut lock = self.on_change.write();
        let current_index = lock.len();
        lock.push(boxed_handler);
        let new_handler = move || {
            let mut guard = self.on_change.write();
            let handler = guard.get_mut(current_index).unwrap();
            let value = self.get();
            (*handler)(&value);
        };
        let dyn_handler = Box::new(new_handler) as Box<dyn Fn() + Send + Sync>;
        let mut guard = self.data_element.on_change.write();
        guard.push(dyn_handler);
    }
}
