//! ECS Service Bridge Integration
//!
//! Direct usage of ECS service bridge events for inter-plugin communication.

use bevy::prelude::*;
use ecs_service_bridge::events::{
    ClipboardEvent, ClipboardOperation, HttpEvent, HttpOperation, NotificationEvent, StorageEvent,
    StorageOperation,
};
use ecs_service_bridge::types::TimeStamp;

/// Clipboard service using ECS events directly
#[derive(Resource, Debug, Clone)]
pub struct ClipboardService {
    request_id: String,
}

impl ClipboardService {
    pub fn new(request_id: String) -> Self {
        Self { request_id }
    }

    /// Send clipboard read event
    pub fn read(&self, event_writer: &mut EventWriter<ClipboardEvent>) {
        event_writer.write(ClipboardEvent {
            request_id: self.request_id.clone(),
            operation: ClipboardOperation::Read,
            timestamp: TimeStamp::now(),
        });
    }

    /// Send clipboard write event
    pub fn write(&self, content: String, event_writer: &mut EventWriter<ClipboardEvent>) {
        event_writer.write(ClipboardEvent {
            request_id: self.request_id.clone(),
            operation: ClipboardOperation::Write(content),
            timestamp: TimeStamp::now(),
        });
    }
}

/// Storage service using ECS events directly
#[derive(Resource, Debug, Clone)]
pub struct StorageService {
    request_id: String,
}

impl StorageService {
    pub fn new(request_id: String) -> Self {
        Self { request_id }
    }

    /// Read value by key
    pub fn read(&self, key: String, event_writer: &mut EventWriter<StorageEvent>) {
        let event = StorageEvent {
            request_id: self.request_id.clone(),
            operation: StorageOperation::Read(key),
            timestamp: TimeStamp::now(),
        };
        event_writer.write(event);
    }

    /// Write key-value pair
    pub fn write(&self, key: String, value: String, event_writer: &mut EventWriter<StorageEvent>) {
        let event = StorageEvent {
            request_id: self.request_id.clone(),
            operation: StorageOperation::Write(key, value),
            timestamp: TimeStamp::now(),
        };
        event_writer.write(event);
    }
}

/// HTTP service using ECS events directly
#[derive(Resource, Debug, Clone)]
pub struct HttpService {
    request_id: String,
}

impl HttpService {
    pub fn new(request_id: String) -> Self {
        Self { request_id }
    }

    /// Send HTTP request
    pub fn request(
        &self,
        url: String,
        method: String,
        body: Option<String>,
        event_writer: &mut EventWriter<HttpEvent>,
    ) {
        let event = HttpEvent {
            request_id: self.request_id.clone(),
            operation: HttpOperation::Request { url, method, body },
            timestamp: TimeStamp::now(),
        };
        event_writer.write(event);
    }
}

/// Notification service using ECS events directly
#[derive(Resource, Debug, Clone)]
pub struct NotificationService {
    request_id: String,
}

impl NotificationService {
    pub fn new(request_id: String) -> Self {
        Self { request_id }
    }

    /// Send notification
    pub fn notify(
        &self,
        title: String,
        body: String,
        event_writer: &mut EventWriter<NotificationEvent>,
    ) {
        let event = NotificationEvent {
            request_id: self.request_id.clone(),
            title,
            body,
            timestamp: TimeStamp::now(),
        };
        event_writer.write(event);
    }
}
