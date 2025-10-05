use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::state::state::States;
#[cfg(feature = "async")]
use crossbeam_channel::{Receiver, Sender};

use crate::prelude::*;

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

/// Unique identifier for progress entries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntryId(usize);

impl EntryId {
    /// Generate a new unique entry ID
    pub fn new() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for EntryId {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource for monitoring progress without mutex locks, using ECS patterns
#[derive(Resource)]
pub struct ProgressMonitor<S: States> {
    /// Individual progress entries
    entries: HashMap<EntryId, (Progress, HiddenProgress)>,
    /// Sum from entities (computed by systems)
    entity_sum: (Progress, HiddenProgress),
    #[cfg(feature = "async")]
    /// Channel for async progress updates
    pub async_channel:
        Option<(Sender<AsyncProgressMessage>, Receiver<AsyncProgressMessage>)>,
    _phantom: std::marker::PhantomData<S>,
}

#[cfg(feature = "async")]
#[derive(Debug, Clone)]
/// Message sent over async channels for progress updates
pub struct AsyncProgressMessage {
    /// Entry ID this message is for
    pub entry_id: EntryId,
    /// Visible progress update, if any
    pub visible: Option<Progress>,
    /// Hidden progress update, if any
    pub hidden: Option<HiddenProgress>,
}

impl<S: States> Default for ProgressMonitor<S> {
    fn default() -> Self {
        Self {
            entries: HashMap::default(),
            entity_sum: (Progress::default(), HiddenProgress::default()),
            #[cfg(feature = "async")]
            async_channel: None,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S: States> ProgressMonitor<S> {
    /// Clear all progress data
    pub fn reset(&mut self) {
        self.entries.clear();
        self.entity_sum = (Progress::default(), HiddenProgress::default());
        #[cfg(feature = "async")]
        {
            self.async_channel = None;
        }
    }

    /// Update visible progress for an entry
    pub fn update_visible(&mut self, id: EntryId, progress: Progress) {
        self.entries.entry(id).or_default().0 = progress;
    }

    /// Update hidden progress for an entry  
    pub fn update_hidden(&mut self, id: EntryId, progress: HiddenProgress) {
        self.entries.entry(id).or_default().1 = progress;
    }

    /// Set the sum from entities (called by entity summing system)
    pub fn set_entity_sum(
        &mut self,
        visible: Progress,
        hidden: HiddenProgress,
    ) {
        self.entity_sum = (visible, hidden);
    }

    /// Get total visible progress (entries + entities)
    pub fn get_total_visible(&self) -> Progress {
        let entries_sum = self
            .entries
            .values()
            .fold(Progress::default(), |acc, (v, _)| acc + *v);
        entries_sum + self.entity_sum.0
    }

    /// Get total hidden progress (entries + entities)
    pub fn get_total_hidden(&self) -> HiddenProgress {
        let entries_sum = self
            .entries
            .values()
            .fold(HiddenProgress::default(), |acc, (_, h)| acc + *h);
        HiddenProgress(entries_sum.0 + self.entity_sum.1.0)
    }

    /// Check if all progress is complete
    pub fn is_complete(&self) -> bool {
        let visible = self.get_total_visible();
        let hidden = self.get_total_hidden();
        visible.is_complete() && hidden.is_complete()
    }

    /// Get progress for specific entry
    pub fn get_entry(&self, id: EntryId) -> Option<(Progress, HiddenProgress)> {
        self.entries.get(&id).copied()
    }

    #[cfg(feature = "async")]
    /// Create a sender for async progress updates
    pub fn create_async_sender(&mut self) -> ProgressSender {
        if self.async_channel.is_none() {
            let (tx, rx) = crossbeam_channel::unbounded();
            self.async_channel = Some((tx, rx));
        }

        ProgressSender {
            entry_id: EntryId::new(),
            sender: self
                .async_channel
                .as_ref()
                .unwrap_or_else(|| panic!("Critical error: async channel should be initialized before creating progress sender - this indicates improper monitor initialization"))
                .0
                .clone(),
        }
    }

    #[cfg(feature = "async")]
    /// Process pending async messages
    pub fn process_async_messages(
        &mut self,
        progress_writer: &mut EventWriter<Progress>,
        hidden_writer: &mut EventWriter<HiddenProgress>,
    ) {
        let messages: Vec<AsyncProgressMessage> =
            if let Some((_, ref receiver)) = self.async_channel {
                receiver.try_iter().collect()
            } else {
                Vec::new()
            };

        for msg in messages {
            if let Some(visible) = msg.visible {
                self.update_visible(msg.entry_id, visible);
                progress_writer.write(visible);
            }
            if let Some(hidden) = msg.hidden {
                self.update_hidden(msg.entry_id, hidden);
                hidden_writer.write(hidden);
            }
        }
    }
}

/// System parameter for easy progress tracking
#[derive(SystemParam)]
#[allow(dead_code)] // Part of public library API, not yet used by application
pub struct ProgressHandle<'w, 's, S: States> {
    monitor: ResMut<'w, ProgressMonitor<S>>,
    progress_writer: EventWriter<'w, Progress>,
    hidden_writer: EventWriter<'w, HiddenProgress>,
    entry_id: Local<'s, EntryId>,
}

#[allow(dead_code)] // Part of public library API, not yet used by application
impl<'w, 's, S: States> ProgressHandle<'w, 's, S> {
    /// Get this handle's entry ID
    pub fn entry_id(&self) -> EntryId {
        *self.entry_id
    }

    /// Set visible progress and emit event
    pub fn set_visible(&mut self, done: u32, total: u32) {
        let progress = Progress { done, total };
        self.monitor.update_visible(*self.entry_id, progress);
        self.progress_writer.write(progress);
    }

    /// Set hidden progress and emit event
    pub fn set_hidden(&mut self, done: u32, total: u32) {
        let progress = HiddenProgress(Progress { done, total });
        self.monitor.update_hidden(*self.entry_id, progress);
        self.hidden_writer.write(progress);
    }

    /// Add to visible progress
    pub fn add_visible(&mut self, done: u32, total: u32) {
        let current = self
            .monitor
            .get_entry(*self.entry_id)
            .map(|(v, _)| v)
            .unwrap_or_default();
        let new_progress = Progress {
            done: current.done + done,
            total: current.total + total,
        };
        self.monitor.update_visible(*self.entry_id, new_progress);
        self.progress_writer.write(new_progress);
    }

    /// Add to hidden progress
    pub fn add_hidden(&mut self, done: u32, total: u32) {
        let current = self
            .monitor
            .get_entry(*self.entry_id)
            .map(|(_, h)| h)
            .unwrap_or_default();
        let new_progress = HiddenProgress(Progress {
            done: current.0.done + done,
            total: current.0.total + total,
        });
        self.monitor.update_hidden(*self.entry_id, new_progress);
        self.hidden_writer.write(new_progress);
    }

    /// Check if this entry's progress is complete
    pub fn is_complete(&self) -> bool {
        if let Some((visible, hidden)) = self.monitor.get_entry(*self.entry_id)
        {
            visible.is_complete() && hidden.is_complete()
        } else {
            false
        }
    }

    /// Get current progress for this entry
    pub fn get_progress(&self) -> (Progress, HiddenProgress) {
        self.monitor.get_entry(*self.entry_id).unwrap_or_default()
    }
}

#[cfg(feature = "async")]
/// Handle for sending progress updates from async contexts
#[derive(Clone)]
pub struct ProgressSender {
    entry_id: EntryId,
    sender: Sender<AsyncProgressMessage>,
}

#[cfg(feature = "async")]
impl ProgressSender {
    /// Send visible progress update
    pub fn send_visible(
        &self,
        done: u32,
        total: u32,
    ) -> Result<(), crossbeam_channel::SendError<AsyncProgressMessage>> {
        self.sender.send(AsyncProgressMessage {
            entry_id: self.entry_id,
            visible: Some(Progress { done, total }),
            hidden: None,
        })
    }

    /// Send hidden progress update
    pub fn send_hidden(
        &self,
        done: u32,
        total: u32,
    ) -> Result<(), crossbeam_channel::SendError<AsyncProgressMessage>> {
        self.sender.send(AsyncProgressMessage {
            entry_id: self.entry_id,
            visible: None,
            hidden: Some(HiddenProgress(Progress { done, total })),
        })
    }

    /// Send both visible and hidden progress
    pub fn send_both(
        &self,
        visible: (u32, u32),
        hidden: (u32, u32),
    ) -> Result<(), crossbeam_channel::SendError<AsyncProgressMessage>> {
        self.sender.send(AsyncProgressMessage {
            entry_id: self.entry_id,
            visible: Some(Progress {
                done: visible.0,
                total: visible.1,
            }),
            hidden: Some(HiddenProgress(Progress {
                done: hidden.0,
                total: hidden.1,
            })),
        })
    }

    /// Get the entry ID for this sender
    pub fn entry_id(&self) -> EntryId {
        self.entry_id
    }
}
