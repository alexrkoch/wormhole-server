use std::collections::HashSet;

use crate::game::Player;

/// A room is an entity that maintains a collection of [players][Player]
/// and is responsible for orchestrating their interactions
#[derive(Debug, PartialEq)]
pub(crate) struct Room {
    players: HashSet<Player>,
}

impl Room {
    pub fn new() -> Self {
        Self {
            players: Default::default(),
        }
    }
}
