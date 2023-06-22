//! Houses functionality related to player sessions

use std::hash::Hash;
#[derive(Debug, Hash, PartialOrd, Eq, PartialEq, Copy, Clone)]
/// Id type that uniquely identifies a player
pub(crate) struct PlayerId(u128);

impl From<u128> for PlayerId {
    fn from(value: u128) -> Self {
        PlayerId(value)
    }
}

#[derive(Debug, Eq)]
/// A player in a room
pub(crate) struct Player {
    id: PlayerId,
}

impl PartialOrd for Player {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Player {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Player {}
