use std::{any::Any, fmt::Debug};

use dyn_clone::DynClone;
use serde::{de::DeserializeOwned, Serialize};

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

pub trait SynchronizableClone {
    fn clone_any_box(&self) -> Box<dyn Any>;
    fn clone_synchronizable(&self) -> Box<dyn Synchronizable>;
}

pub trait Synchronizable: 'static + Sync + Send + Debug + DynClone + SynchronizableClone {
    fn serialize(&self) -> String;
    fn deserialize(&self, data: &str) -> Box<dyn Synchronizable>;
}

impl<T> SynchronizableClone for T
where
    T: 'static + Synchronizable + Clone,
{
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
