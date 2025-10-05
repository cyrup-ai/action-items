//! Service implementations for plugin bridge

use action_items_native::context::{
    ClipboardReadRequest, ClipboardWriteRequest, HttpRequest, NotificationRequest,
    StorageReadRequest, StorageWriteRequest,
};
use crossbeam_channel::{Sender, unbounded};

/// Clipboard access service
#[derive(Debug, Clone)]
pub struct ClipboardAccess {
    read_sender: Sender<ClipboardReadRequest>,
    write_sender: Sender<ClipboardWriteRequest>,
}

impl Default for ClipboardAccess {
    fn default() -> Self {
        let (read_sender, _) = unbounded();
        let (write_sender, _) = unbounded();

        Self {
            read_sender,
            write_sender,
        }
    }
}

impl ClipboardAccess {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_sender(&self) -> Sender<ClipboardReadRequest> {
        self.read_sender.clone()
    }

    pub fn write_sender(&self) -> Sender<ClipboardWriteRequest> {
        self.write_sender.clone()
    }
}

/// Notification service
#[derive(Debug, Clone)]
pub struct NotificationService {
    sender: Sender<NotificationRequest>,
}

impl Default for NotificationService {
    fn default() -> Self {
        let (sender, _) = unbounded();

        Self { sender }
    }
}

impl NotificationService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sender(&self) -> Sender<NotificationRequest> {
        self.sender.clone()
    }
}

/// HTTP client service
#[derive(Debug, Clone)]
pub struct HttpClient {
    sender: Sender<HttpRequest>,
}

impl Default for HttpClient {
    fn default() -> Self {
        let (sender, _) = unbounded();

        Self { sender }
    }
}

impl HttpClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sender(&self) -> Sender<HttpRequest> {
        self.sender.clone()
    }
}

/// Storage service
#[derive(Debug, Clone)]
pub struct StorageService {
    read_sender: Sender<StorageReadRequest>,
    write_sender: Sender<StorageWriteRequest>,
}

impl Default for StorageService {
    fn default() -> Self {
        let (read_sender, _) = unbounded();
        let (write_sender, _) = unbounded();

        Self {
            read_sender,
            write_sender,
        }
    }
}

impl StorageService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_sender(&self) -> Sender<StorageReadRequest> {
        self.read_sender.clone()
    }

    pub fn write_sender(&self) -> Sender<StorageWriteRequest> {
        self.write_sender.clone()
    }
}
