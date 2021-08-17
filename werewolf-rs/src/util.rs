use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};

use rand::Rng;
use serde::{Deserialize, Serialize};

mod id_type {
    pub struct Player;
    pub struct Interaction;
    pub struct Lobby;
}


pub type PlayerId = Id<id_type::Player>;
pub type InteractionId = Id<id_type::Interaction>;
pub type LobbyId = Id<id_type::Lobby>;

/*
A general purpose id type.
The generic parameter T does nothing and is just used to enforce type checking with different kinds of IDs
*/
pub struct Id<T> {
    value: u64,
    id_type: PhantomData<T>
}

impl<T> Id<T> {
    pub fn new<V>(used_ids: &HashMap<Id<T>, V>) -> Self {
        loop {
            let id = Id {
                value: rand::thread_rng().gen(),
                id_type: PhantomData
            };
            if !used_ids.contains_key(&id) {
                return id;
            }
        }
    }
}

//The custom serialize/deserialize implementations are used here to flatten the struct
impl<T> Serialize for Id<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        self.value.serialize(serializer)
    }
}
impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        Ok(Id {
            value: u64::deserialize(deserializer)?,
            id_type: PhantomData
        })
    }
}
impl<T> Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self { value: self.value.clone(), id_type: PhantomData }
    }
}
impl<T> Copy for Id<T> {}
impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl<T> Eq for Id<T> {}
impl<T> Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}