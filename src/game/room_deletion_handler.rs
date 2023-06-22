//! Houses utilities related to orchestrating automatic idle room cleanup

use std::sync::Arc;
use std::sync::Mutex;

use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::game::{RoomId, RoomRegistry};

/// The number of room deletion messages that can sit in the room deletion channel before sending
/// on it becomes blocking
const DEFAULT_DELETION_CHANNEL_BUFFER_SIZE: usize = 100;

/// Cleans up rooms within a given registry when their IDs are sent over the provided channel
pub(crate) struct RoomDeletionHandler {
    registry_mutex: Arc<Mutex<RoomRegistry>>,
    receiver: Receiver<RoomId>,
}

impl RoomDeletionHandler {
    /// Creates a new deletion handler for a provided [registry][RoomRegistry]
    pub fn new_with_registry(registry_mutex: Arc<Mutex<RoomRegistry>>) -> (Self, Sender<RoomId>) {
        let (sender, receiver) = mpsc::channel::<RoomId>(DEFAULT_DELETION_CHANNEL_BUFFER_SIZE);

        let handler = Self {
            receiver,
            registry_mutex,
        };

        (handler, sender)
    }

    /// Begins watching for and handling room deletion requests
    pub async fn watch(&mut self) {
        while let Some(room_id) = &self.receiver.recv().await {
            let mut registry = self.registry_mutex.lock().unwrap();
            registry.delete_room(room_id);
        }
    }
}
