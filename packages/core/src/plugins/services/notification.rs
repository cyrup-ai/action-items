use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::*;
use notify_rust::{Notification, NotificationHandle};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Service for managing notification operations in plugins
#[derive(Resource, Clone)]
pub struct NotificationService {
    inner: Arc<RwLock<NotificationServiceInner>>,
    app_name: String,
}

struct NotificationServiceInner {
    // Active notifications storage
    active_notifications: HashMap<String, NotificationData>,
    // Store notification handles for dismissal
    notification_handles: HashMap<String, NotificationHandle>,
}

#[derive(Clone, Debug)]
struct NotificationData {
    id: String,
    title: String,
    body: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new("Action Items".to_string())
    }
}

impl NotificationService {
    pub fn new(app_name: String) -> Self {
        Self {
            inner: Arc::new(RwLock::new(NotificationServiceInner {
                active_notifications: HashMap::new(),
                notification_handles: HashMap::new(),
            })),
            app_name,
        }
    }

    /// Show a notification with title and body
    pub async fn show(&self, title: &str, body: &str) -> Result<String, String> {
        let notification_id = Uuid::new_v4().to_string();

        let notification = NotificationData {
            id: notification_id.clone(),
            title: title.to_string(),
            body: body.to_string(),
            timestamp: chrono::Utc::now(),
        };

        // Create and show system notification
        let handle = Notification::new()
            .appname(&self.app_name)
            .summary(title)
            .body(body)
            .timeout(notify_rust::Timeout::Milliseconds(5000))
            .show()
            .map_err(|e| format!("Failed to show notification: {}", e))?;

        let mut inner = self.inner.write().await;
        inner
            .active_notifications
            .insert(notification_id.clone(), notification);
        inner
            .notification_handles
            .insert(notification_id.clone(), handle);

        log::info!("Notification [{}]: {} - {}", self.app_name, title, body);
        Ok(notification_id)
    }

    /// Show a simple notification with just a message
    pub async fn show_simple(&self, message: &str) -> Result<String, String> {
        self.show(&self.app_name, message).await
    }

    /// Dismiss a notification by ID
    pub async fn dismiss(&self, notification_id: &str) -> Result<(), String> {
        let mut inner = self.inner.write().await;

        if inner.active_notifications.remove(notification_id).is_some() {
            // Remove the system notification handle (notifications auto-dismiss after timeout)
            inner.notification_handles.remove(notification_id);
            log::debug!("Dismissed notification: {}", notification_id);
            Ok(())
        } else {
            Err(format!("Notification not found: {}", notification_id))
        }
    }

    /// Get all active notification IDs
    pub async fn get_active_ids(&self) -> Vec<String> {
        let inner = self.inner.read().await;
        inner.active_notifications.keys().cloned().collect()
    }

    /// Clear all notifications
    pub async fn clear_all(&self) -> Result<(), String> {
        let mut inner = self.inner.write().await;
        let count = inner.active_notifications.len();

        // Clear all system notification handles (notifications auto-dismiss after timeout)
        inner.notification_handles.clear();

        inner.active_notifications.clear();
        log::debug!("Cleared {} notifications", count);
        Ok(())
    }

    /// Get notification data by ID
    pub async fn get_notification(&self, notification_id: &str) -> Option<NotificationInfo> {
        let inner = self.inner.read().await;
        inner
            .active_notifications
            .get(notification_id)
            .map(|data| NotificationInfo {
                id: data.id.clone(),
                title: data.title.clone(),
                body: data.body.clone(),
                timestamp: data.timestamp,
            })
    }

    /// Get all active notifications with their data
    pub async fn get_all_notifications(&self) -> Vec<NotificationInfo> {
        let inner = self.inner.read().await;
        inner
            .active_notifications
            .values()
            .map(|data| NotificationInfo {
                id: data.id.clone(),
                title: data.title.clone(),
                body: data.body.clone(),
                timestamp: data.timestamp,
            })
            .collect()
    }

    /// Get notifications by title pattern
    pub async fn find_notifications_by_title(&self, title_pattern: &str) -> Vec<NotificationInfo> {
        let inner = self.inner.read().await;
        inner
            .active_notifications
            .values()
            .filter(|data| data.title.contains(title_pattern))
            .map(|data| NotificationInfo {
                id: data.id.clone(),
                title: data.title.clone(),
                body: data.body.clone(),
                timestamp: data.timestamp,
            })
            .collect()
    }

    /// Get notification count
    pub async fn notification_count(&self) -> usize {
        let inner = self.inner.read().await;
        inner.active_notifications.len()
    }

    /// Get app name
    pub fn app_name(&self) -> &str {
        &self.app_name
    }
}

/// Public notification information structure
#[derive(Clone, Debug)]
pub struct NotificationInfo {
    pub id: String,
    pub title: String,
    pub body: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
