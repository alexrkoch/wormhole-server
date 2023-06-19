use std::sync::Arc;
use std::sync::Mutex;

use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::game::{RoomId, RoomRegistry};

const DEFAULT_DELETION_CHANNEL_BUFFER_SIZE: usize = 100;

pub(crate) struct RoomDeletionHandler {
    registry_mutex: Arc<Mutex<RoomRegistry>>,
    receiver: Receiver<RoomId>,
}

impl RoomDeletionHandler {
    pub fn new_with_registry(registry_mutex: Arc<Mutex<RoomRegistry>>) -> (Self, Sender<RoomId>) {
        let (sender, receiver) = mpsc::channel::<RoomId>(DEFAULT_DELETION_CHANNEL_BUFFER_SIZE);

        let handler = Self {
            receiver,
            registry_mutex,
        };

        (handler, sender)
    }

    pub async fn watch(&mut self) {
        while let Some(room_id) = &self.receiver.recv().await {
            let mut registry = self.registry_mutex.lock().unwrap();
            registry.delete_room(room_id);
        }
    }
}
