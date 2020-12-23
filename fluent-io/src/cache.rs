use crate::value::ValueStatus;
use std::collections::HashMap;

pub struct AsyncCache<R>(HashMap<String, ValueStatus<R>>);

impl<R> AsyncCache<R> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get_or_insert_with<F>(&mut self, key: &str, f: F) -> ValueStatus<R>
    where
        F: Fn() -> ValueStatus<R>,
        R: Clone,
    {
        self.0.entry(key.to_owned()).or_insert_with(f).clone()
    }

    pub fn get_or_insert_with_sync<F>(&mut self, key: &str, f: F) -> R
    where
        F: Fn() -> R,
        R: Clone,
    {
        let status = self
            .0
            .entry(key.to_owned())
            .or_insert_with(|| ValueStatus::Loaded(f()));
        match status {
            ValueStatus::Loading(_) => {
                println!(
                    "Attempting to synchronously ask for key: {} which is being loaded.",
                    &key
                );
                let v = f();
                self.0
                    .insert(key.to_owned(), ValueStatus::Loaded(v.clone()));
                v
            }
            ValueStatus::Loaded(res) => res.clone(),
        }
    }
}
