use bevy::prelude::*;
use crossbeam_channel::Sender;

use crate::prelude::*;

/// Handle for sending progress updates from background threads or async
/// contexts
///
/// This allows you to update progress from outside the main Bevy systems,
/// such as from async tasks, background threads, or external libraries.
///
/// # Example
///
/// ```rust
/// # use bevy::prelude::*;
/// # use action_items_ecs_progress::prelude::*;
/// # use std::thread;
/// # use std::time::Duration;
/// #
/// # #[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// # enum GameState { #[default] Loading }
/// #
/// fn start_background_loading(
///     mut monitor: ResMut<ProgressMonitor<GameState>>,
/// ) {
///     let sender = monitor.create_async_sender();
///     
///     // Spawn background task
///     thread::spawn(move || {
///         for i in 0..=100 {
///             // Simulate work
///             thread::sleep(Duration::from_millis(50));
///             
///             // Update progress
///             sender.send_visible(i, 100).unwrap();
///             
///             if i == 100 {
///                 // Mark hidden requirements complete too
///                 sender.send_hidden(1, 1).unwrap();
///             }
///         }
///     });
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ProgressSender {
    /// Unique entry ID for this sender
    pub entry_id: EntryId,
    /// Channel sender for async messages
    sender: Sender<AsyncProgressMessage>,
}

/// Message sent over the async channel
#[derive(Debug, Clone)]
pub struct AsyncProgressMessage {
    /// Entry ID this message is for
    pub entry_id: EntryId,
    /// Visible progress update, if any
    pub visible: Option<Progress>,
    /// Hidden progress update, if any
    pub hidden: Option<HiddenProgress>,
}

impl ProgressSender {
    /// Create a new progress sender
    ///
    /// Note: Usually you should get this from
    /// `ProgressMonitor::create_async_sender()` rather than creating it
    /// directly.
    pub fn new(
        entry_id: EntryId,
        sender: Sender<AsyncProgressMessage>,
    ) -> Self {
        Self { entry_id, sender }
    }

    /// Send a visible progress update
    ///
    /// # Errors
    ///
    /// Returns an error if the receiving end has been dropped (usually means
    /// the Bevy app has shut down).
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

    /// Send a hidden progress update
    ///
    /// # Errors
    ///
    /// Returns an error if the receiving end has been dropped.
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

    /// Send both visible and hidden progress updates in one message
    ///
    /// # Errors
    ///
    /// Returns an error if the receiving end has been dropped.
    pub fn send_both(
        &self,
        visible_done: u32,
        visible_total: u32,
        hidden_done: u32,
        hidden_total: u32,
    ) -> Result<(), crossbeam_channel::SendError<AsyncProgressMessage>> {
        self.sender.send(AsyncProgressMessage {
            entry_id: self.entry_id,
            visible: Some(Progress {
                done: visible_done,
                total: visible_total,
            }),
            hidden: Some(HiddenProgress(Progress {
                done: hidden_done,
                total: hidden_total,
            })),
        })
    }

    /// Send a progress update using the Progress struct
    pub fn send_progress(
        &self,
        progress: Progress,
    ) -> Result<(), crossbeam_channel::SendError<AsyncProgressMessage>> {
        self.send_visible(progress.done, progress.total)
    }

    /// Send a hidden progress update using the HiddenProgress struct
    pub fn send_hidden_progress(
        &self,
        progress: HiddenProgress,
    ) -> Result<(), crossbeam_channel::SendError<AsyncProgressMessage>> {
        self.send_hidden(progress.done, progress.total)
    }

    /// Send a completion signal (both visible and hidden set to 1/1)
    pub fn send_complete(
        &self,
    ) -> Result<(), crossbeam_channel::SendError<AsyncProgressMessage>> {
        self.send_both(1, 1, 1, 1)
    }

    /// Send a visible completion signal (visible set to 1/1, hidden unchanged)
    pub fn send_visible_complete(
        &self,
    ) -> Result<(), crossbeam_channel::SendError<AsyncProgressMessage>> {
        self.send_visible(1, 1)
    }

    /// Send a hidden completion signal (hidden set to 1/1, visible unchanged)
    pub fn send_hidden_complete(
        &self,
    ) -> Result<(), crossbeam_channel::SendError<AsyncProgressMessage>> {
        self.send_hidden(1, 1)
    }

    /// Try to send a visible progress update without blocking
    ///
    /// Returns `Ok(())` if sent successfully, `Err` if the channel is full or
    /// disconnected.
    pub fn try_send_visible(
        &self,
        done: u32,
        total: u32,
    ) -> Result<(), crossbeam_channel::TrySendError<AsyncProgressMessage>> {
        self.sender.try_send(AsyncProgressMessage {
            entry_id: self.entry_id,
            visible: Some(Progress { done, total }),
            hidden: None,
        })
    }

    /// Try to send a hidden progress update without blocking
    pub fn try_send_hidden(
        &self,
        done: u32,
        total: u32,
    ) -> Result<(), crossbeam_channel::TrySendError<AsyncProgressMessage>> {
        self.sender.try_send(AsyncProgressMessage {
            entry_id: self.entry_id,
            visible: None,
            hidden: Some(HiddenProgress(Progress { done, total })),
        })
    }

    /// Get the entry ID for this sender
    pub fn entry_id(&self) -> EntryId {
        self.entry_id
    }

    /// Check if the receiver is still connected
    pub fn is_connected(&self) -> bool {
        !self.sender.is_empty() || self.sender.len() > 0
    }
}

/// System that processes async progress messages and updates the monitor
///
/// This system runs automatically when async support is enabled. It receives
/// messages sent via `ProgressSender` and applies them to the
/// `ProgressMonitor`.
pub fn process_async_progress<S: States>(
    mut monitor: ResMut<ProgressMonitor<S>>,
    mut progress_writer: EventWriter<Progress>,
    mut hidden_writer: EventWriter<HiddenProgress>,
) {
    monitor.process_async_messages(&mut progress_writer, &mut hidden_writer);
}

/// Resource containing statistics about async progress operations
#[derive(Resource, Default, Debug, Clone)]
#[allow(dead_code)]
pub struct AsyncProgressStats {
    /// Total messages sent since app start
    pub total_messages_sent: u64,
    /// Total messages processed since app start
    pub total_messages_processed: u64,
    /// Messages currently in flight
    pub messages_pending: u32,
    /// Number of active senders
    pub active_senders: u32,
}

impl AsyncProgressStats {
    /// Create new stats
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a message being sent
    #[allow(dead_code)]
    pub fn record_sent(&mut self) {
        self.total_messages_sent += 1;
        self.messages_pending += 1;
    }

    /// Record a message being processed
    #[allow(dead_code)]
    pub fn record_processed(&mut self) {
        self.total_messages_processed += 1;
        self.messages_pending = self.messages_pending.saturating_sub(1);
    }

    /// Record a new sender being created
    #[allow(dead_code)]
    pub fn record_sender_created(&mut self) {
        self.active_senders += 1;
    }

    /// Record a sender being dropped
    #[allow(dead_code)]
    pub fn record_sender_dropped(&mut self) {
        self.active_senders = self.active_senders.saturating_sub(1);
    }

    /// Get the success rate as a percentage
    #[allow(dead_code)]
    pub fn success_rate(&self) -> f32 {
        if self.total_messages_sent == 0 {
            100.0
        } else {
            (self.total_messages_processed as f32
                / self.total_messages_sent as f32)
                * 100.0
        }
    }
}

/// System to update async progress statistics
#[allow(dead_code)]
pub fn update_async_stats<S: States>(
    monitor: Res<ProgressMonitor<S>>,
    mut stats: Option<ResMut<AsyncProgressStats>>,
) {
    if let Some(ref mut stats) = stats {
        // Update pending messages count based on channel length
        if let Some((_, ref receiver)) = monitor.async_channel {
            stats.messages_pending = receiver.len() as u32;
        }
    }
}

#[cfg(test)]
mod tests {


    use bevy::prelude::*;

    use super::*;

    #[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[allow(dead_code)]
    enum TestState {
        #[default]
        Loading,
    }

    #[test]
    fn test_progress_sender_creation() {
        let (sender, _receiver) = crossbeam_channel::unbounded();
        let entry_id = EntryId::new();
        let progress_sender = ProgressSender::new(entry_id, sender);

        assert_eq!(progress_sender.entry_id(), entry_id);
    }

    #[test]
    fn test_sending_progress() {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let progress_sender = ProgressSender::new(EntryId::new(), sender);

        // Send visible progress
        progress_sender.send_visible(5, 10).unwrap();

        // Receive and verify
        let msg = receiver.try_recv().unwrap();
        assert!(msg.visible.is_some());
        assert_eq!(msg.visible.unwrap().done, 5);
        assert_eq!(msg.visible.unwrap().total, 10);
        assert!(msg.hidden.is_none());
    }

    #[test]
    fn test_sending_both_progress() {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let progress_sender = ProgressSender::new(EntryId::new(), sender);

        // Send both visible and hidden
        progress_sender.send_both(3, 10, 1, 2).unwrap();

        // Receive and verify
        let msg = receiver.try_recv().unwrap();
        assert!(msg.visible.is_some());
        assert!(msg.hidden.is_some());
        assert_eq!(msg.visible.unwrap().done, 3);
        assert_eq!(msg.hidden.unwrap().done, 1);
    }

    #[test]
    fn test_async_progress_stats() {
        let mut stats = AsyncProgressStats::new();

        stats.record_sent();
        stats.record_sent();
        stats.record_processed();

        assert_eq!(stats.total_messages_sent, 2);
        assert_eq!(stats.total_messages_processed, 1);
        assert_eq!(stats.messages_pending, 1);
        assert_eq!(stats.success_rate(), 50.0);
    }

    #[test]
    fn test_completion_signals() {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let progress_sender = ProgressSender::new(EntryId::new(), sender);

        // Send completion
        progress_sender.send_complete().unwrap();

        let msg = receiver.try_recv().unwrap();
        assert!(msg.visible.unwrap().is_complete());
        assert!(msg.hidden.unwrap().is_complete());
    }
}
