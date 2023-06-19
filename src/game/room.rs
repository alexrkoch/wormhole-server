use std::collections::HashSet;

use tokio::sync::mpsc::Sender;
use tokio::task::{spawn, JoinHandle};
use tokio::time;
use tracing::{info, instrument, warn};

use crate::game::{Player, RoomId};

const DEFAULT_DELETION_TIMEOUT_SECONDS: u64 = 30;

/// A room is an entity that maintains a collection of [players][Player]
/// and is responsible for orchestrating their interactions
#[derive(Debug)]
pub(crate) struct Room {
    id: RoomId,
    deletion_channel: Sender<RoomId>,
    deletion_handle: Option<JoinHandle<()>>,
    players: HashSet<Player>,
}

impl PartialEq for Room {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Room {
    pub fn new(id: RoomId, deletion_channel: Sender<RoomId>) -> Self {
        let mut this = Self {
            id,
            deletion_channel,
            deletion_handle: None,
            players: Default::default(),
        };

        this.schedule_deletion();

        this
    }

    #[instrument(skip_all)]
    fn schedule_deletion(&mut self) {
        info!(event = "scheduling_deletion", room_id = %self.id);
        let deletion_channel = self.deletion_channel.clone();
        let id = self.id.clone();
        let handle = spawn(async move {
            time::sleep(time::Duration::from_secs(DEFAULT_DELETION_TIMEOUT_SECONDS)).await;

            info!(event = "requesting_deletion", room_id = %id);
            let res = deletion_channel.send(id).await;
            info!(event = "deletion_request_result", res = ?res);
        });

        self.deletion_handle = Some(handle);
    }

    #[instrument(skip_all)]
    fn cancel_deletion(&mut self) {
        match &self.deletion_handle {
            None => {
                warn!(event = "cancel_deletion_task.invalid", room_id = %self.id)
            }
            Some(handle) => {
                info!(event = "cancel_deletion_task.valid", room_id = %self.id);
                handle.abort();
            }
        }

        self.deletion_handle = None;
    }
}
