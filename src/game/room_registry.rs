use std::collections::HashMap;

use thiserror::Error;
use tokio::sync::mpsc::Sender;
use tracing::{info, instrument, warn};
use uuid::Uuid;

use crate::game::Room;

const MAX_CREATE_ROOM_ID_ATTEMPTS: u8 = 5;

pub(crate) trait ProvideRoomId {
    fn provide_id() -> RoomId;
}

impl ProvideRoomId for Uuid {
    fn provide_id() -> RoomId {
        Uuid::new_v4().as_u128().into()
    }
}

/// An ID that uniquely identifies a [room][Room] within a [registry][RoomRegistry]
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Copy, Clone)]
pub(crate) struct RoomId(u128);

impl std::fmt::Display for RoomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Uuid::from_u128(self.0).fmt(f)
    }
}

impl From<u128> for RoomId {
    fn from(value: u128) -> Self {
        RoomId(value)
    }
}

/// RoomRegistry maintains a list of [rooms][Room]
#[derive(Debug)]
pub(crate) struct RoomRegistry<T: ProvideRoomId = Uuid> {
    rooms: HashMap<RoomId, Room>,
    _provider: std::marker::PhantomData<T>,
}

/// Enumerates the errors that can occur within the context of [room][Room] creation
#[derive(Error, Debug, PartialEq)]
pub(crate) enum RoomCreationError {
    #[error("Unable to create a unique room identifier after {0} attempts")]
    UnableToCreateIdentifier(u8),
}

impl RoomRegistry<Uuid> {
    pub fn new() -> Self {
        Self {
            rooms: Default::default(),
            _provider: std::marker::PhantomData,
        }
    }
}

impl<T: ProvideRoomId> RoomRegistry<T> {
    #[instrument(skip_all)]
    pub fn get_room_for_id(&self, id: impl Into<RoomId>) -> Option<&Room> {
        info!(event = "room_registry.get_room_for_id");
        return self.rooms.get(&id.into());
    }

    #[instrument(skip_all)]
    pub fn create_room(
        &mut self,
        deletion_sender: Sender<RoomId>,
    ) -> Result<RoomId, RoomCreationError> {
        info!(event = "start");
        let mut id = T::provide_id();
        let mut attempts = 0;
        while self.rooms.contains_key(&id) {
            if attempts >= MAX_CREATE_ROOM_ID_ATTEMPTS {
                warn!(
                    event = "room_creation_error",
                    current_room_count = self.rooms.len()
                );
                return Err(RoomCreationError::UnableToCreateIdentifier(
                    MAX_CREATE_ROOM_ID_ATTEMPTS,
                ));
            }
            attempts += 1;
            id = T::provide_id();
        }

        let room = Room::new(id, deletion_sender);
        self.rooms.insert(id, room);
        info!(event = "room_created_successfully", id = %id);
        Ok(id)
    }

    pub fn list_active_rooms(&self) -> Vec<String> {
        self.rooms.keys().map(|rm| format!("{}", rm)).collect()
    }

    #[instrument(skip_all)]
    pub fn delete_room(&mut self, id: &RoomId) {
        info!(event = "deleting_room", room_id = %id);
        self.rooms.remove(id);
    }
}

#[cfg(test)]
mod get_room_for_id {
    use super::*;

    #[tokio::test]
    async fn returns_room_if_one_exists() {
        let room_id = 1234_u128;
        let (sender, _) = tokio::sync::mpsc::channel(1);
        let rooms = HashMap::from([(room_id.into(), Room::new(RoomId(room_id), sender.clone()))]);

        let registry: RoomRegistry<Uuid> = RoomRegistry {
            rooms,
            _provider: std::marker::PhantomData,
        };
        let room = registry.get_room_for_id(room_id);
        assert_eq!(room, Some(&Room::new(RoomId(room_id), sender)));
    }

    #[tokio::test]
    async fn returns_none_if_no_rooms_exist() {
        let room_id = 1234_u128;
        let bad_room_id = 0_u128;
        let (sender, _) = tokio::sync::mpsc::channel(1);
        let rooms = HashMap::from([(room_id.into(), Room::new(RoomId(room_id), sender))]);

        let registry: RoomRegistry<Uuid> = RoomRegistry {
            rooms,
            _provider: std::marker::PhantomData,
        };
        let room = registry.get_room_for_id(bad_room_id);
        assert_eq!(room, None);
    }
}

#[cfg(test)]
mod create_room {
    use super::*;

    #[tokio::test]
    async fn adds_room_to_registry_on_creation() {
        let mut registry = RoomRegistry::new();
        let (sender, _) = tokio::sync::mpsc::channel(1);
        let id = registry.create_room(sender).unwrap();
        let room = registry.get_room_for_id(id);
        assert_ne!(room, Option::None);
    }

    #[tokio::test]
    async fn fails_if_new_room_cant_be_created_after_max_attempts() {
        struct BadIdProvider;
        impl ProvideRoomId for BadIdProvider {
            fn provide_id() -> RoomId {
                0_128.into()
            }
        }

        let mut registry: RoomRegistry<BadIdProvider> = RoomRegistry {
            rooms: Default::default(),
            _provider: std::marker::PhantomData,
        };
        let (sender, _) = tokio::sync::mpsc::channel(1);
        // Bad room id provider only returns 0 so after the first room is created
        // we should be unable to create another one
        let _ = registry.create_room(sender);

        let (sender, _) = tokio::sync::mpsc::channel(1);
        let res = registry.create_room(sender);
        assert_eq!(
            res,
            Err(RoomCreationError::UnableToCreateIdentifier(
                MAX_CREATE_ROOM_ID_ATTEMPTS
            ))
        );
    }
}
