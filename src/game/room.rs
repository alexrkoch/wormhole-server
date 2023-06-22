//! Functionality related to rooms. Conceptually a room represents a collection of player sessions.
//! A room will potentially persist for many games.
use std::collections::HashSet;

use tokio::sync::mpsc::Sender;
use tokio::task::{spawn, JoinHandle};
use tokio::time;
use tracing::{info, instrument, warn};

use crate::game::{Player, RoomId};

/// The default timeout in seconds before an idle room will request cleanup
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
    /// Creates a new room instance
    ///
    /// Builds a new room instance for a given identifier and deletion channel. It then schedules
    /// a task that will sleep for a [given duration][DEFAULT_DELETION_TIMEOUT_SECONDS] and will
    /// then send it's identifier over the deletion channel to request that an [upstream entity][crate::RoomDeletionHandler] will
    /// make sure that it is cleaned up.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier for the room
    /// * `deletion_channel` - The channel that will be used to request room deletion
    ///
    /// # Examples
    /// ```
    /// let (sender, receiver) = tokio::mpsc::channel(1);
    /// let room_id = RoomId(0);
    /// let room: Room = Room::new(room_id, sender);
    /// ```
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
    /// Schedules room deletion
    ///
    /// When this is called a task is scheduled at a point in the future that will request room
    /// cleanup. It will retain a copy of the handle for the scheduled task which can later be used
    /// to abort the cleanup in the event that the room is no longer idle.
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
    /// Cancels a deletion for a room if one is scheduled
    ///
    /// Aborts the task associated with this rooms deletion handle so that it's deletion will not
    /// be requested in the future. If the handle is currently [None] then the cancelation function
    /// will have no affect
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
