use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use moka::sync::Cache as MokaCache;

use super::data_types::{HttpRequest, HttpResponseData};

#[derive(Clone)]
pub struct ClipboardAccess;

impl Default for ClipboardAccess {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardAccess {
    pub fn new() -> Self {
        Self
    }

    /// Read text from clipboard asynchronously
    pub async fn get_text(&self) -> Result<String, std::io::Error> {
        // Use the action_items_ecs_clipboard crate for actual clipboard operations
        action_items_ecs_clipboard::ArboardManager::get_text()
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))
    }

    /// Write text to clipboard asynchronously  
    pub async fn set_text(&self, text: &str) -> Result<(), std::io::Error> {
        let text_owned = text.to_string();
        // Use the action_items_ecs_clipboard crate for actual clipboard operations
        action_items_ecs_clipboard::ArboardManager::set_text(text_owned)
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))
    }
}

#[derive(Clone)]
pub struct NotificationService {
    #[allow(dead_code)]
    app_name: String,
}

impl NotificationService {
    pub fn new(app_name: String) -> Self {
        Self { app_name }
    }

    /// Show a notification using the existing ecs-notifications service
    ///
    /// Integrates with production NotificationResource which provides:
    /// - macOS: Real objc2 UserNotifications framework implementation
    /// - Linux: D-Bus org.freedesktop.Notifications integration
    /// - Windows: WinRT Toast notifications
    /// - Cross-platform fallback handling
    pub async fn show(&self, title: &str, body: &str) -> Result<(), std::io::Error> {
        // Use ecs-notifications directly - no service bridge complexity needed
        use ecs_notifications::NotificationBuilder;
        use ecs_notifications::backends::PlatformBackendFactory;
        use ecs_notifications::components::content::RichText;
        use ecs_notifications::components::platform::Platform;

        // Create notification using the builder pattern
        let notification = NotificationBuilder::new()
            .with_title(title)
            .with_body(RichText::plain(body))
            .build();

        // Get platform-appropriate backend
        let current_platform = if cfg!(target_os = "macos") {
            Platform::MacOS
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            Platform::Linux
        };

        if let Some(backend) = PlatformBackendFactory::create_backend(current_platform) {
            // Use real platform notification system
            let request = ecs_notifications::components::platform::NotificationRequest {
                notification_id: uuid::Uuid::new_v4().to_string(),
                content: notification.content,
                options: ecs_notifications::components::platform::DeliveryOptions::default(),
                correlation_id: uuid::Uuid::new_v4().to_string(),
            };

            match backend.deliver_notification(&request).await {
                Ok(_) => Ok(()),
                Err(e) => Err(std::io::Error::other(e.to_string())),
            }
        } else {
            Err(std::io::Error::other(
                "Platform notifications not supported",
            ))
        }
    }
}

/// Production Notification System Integration Documentation
///
/// Instead of custom wrappers, use the existing production-grade ecs-notifications system:
///
/// ## Event-Driven Notification Pattern
///
/// The production system uses Bevy's event system with NotificationResource and NotificationPlugin.
/// This provides proper integration with the ECS architecture and follows established patterns.
///
/// ### Basic Usage Example:
/// ```rust
/// // In your Bevy system:
/// fn send_notification_system(mut notification_events: EventWriter<NotificationRequested>) {
///     notification_events.write(NotificationRequested {
///         title: "Task Complete".to_string(),
///         body: "Your operation finished successfully".to_string(),
///         urgency: NotificationUrgency::Normal,
///         correlation_id: Some(Uuid::new_v4().to_string()),
///     });
/// }
///
/// // Listen for completion:
/// fn handle_notification_results(mut notification_results: EventReader<NotificationDelivered>) {
///     for result in notification_results.read() {
///         if let Some(correlation_id) = &result.correlation_id {
///             // Handle delivery confirmation
///         }
///     }
/// }
/// ```
///
/// ### App Setup:
/// ```rust
/// App::new()
///     .add_plugins(NotificationPlugin)
///     .add_systems(
///         Update,
///         (send_notification_system, handle_notification_results),
///     )
///     .run();
/// ```
///
/// This approach integrates properly with the existing service bridge architecture
/// and follows the established Request/Response event patterns documented in ARCHITECTURE.md.

#[derive(Clone)]
pub struct StorageService {
    pub base_path: PathBuf,
}

impl StorageService {
    pub fn new(base_path: PathBuf, plugin_id: String) -> Result<Self, std::io::Error> {
        let plugin_path = base_path.join(&plugin_id);
        std::fs::create_dir_all(&plugin_path)?;
        Ok(Self {
            base_path: plugin_path,
        })
    }

    /// Process a storage read request asynchronously
    pub async fn process_read_request(
        &self,
        request: &super::requests::StorageReadRequest,
    ) -> Result<String, std::io::Error> {
        let file_path = self.base_path.join(&request.key);
        tokio::fs::read_to_string(file_path).await
    }

    /// Process a storage write request asynchronously
    pub async fn process_write_request(
        &self,
        request: &super::requests::StorageWriteRequest,
    ) -> Result<(), std::io::Error> {
        let file_path = self.base_path.join(&request.key);
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(file_path, &request.value).await
    }
}

#[derive(Clone)]
pub struct HttpClient {
    timeout: std::time::Duration,
    max_retries: u32,
}

impl HttpClient {
    pub fn new(timeout: std::time::Duration, max_retries: u32) -> Self {
        Self {
            timeout,
            max_retries,
        }
    }

    /// Perform an HTTP request asynchronously with proper retry logic
    pub async fn request(&self, request: &HttpRequest) -> Result<HttpResponseData, std::io::Error> {
        use tokio::time::{Duration as TokioDuration, sleep};

        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            let mut req = match request.method {
                super::data_types::HttpMethod::Get => client.get(&request.url),
                super::data_types::HttpMethod::Post => client.post(&request.url),
                super::data_types::HttpMethod::Put => client.put(&request.url),
                super::data_types::HttpMethod::Delete => client.delete(&request.url),
                super::data_types::HttpMethod::Head => client.head(&request.url),
                super::data_types::HttpMethod::Patch => client.patch(&request.url),
            };

            // Add headers
            for (key, value) in &request.headers {
                req = req.header(key, value);
            }

            // Add body if present
            if let Some(body) = &request.body {
                req = req.body(body.clone());
            }

            match req.send().await {
                Ok(response) => {
                    let status = response.status().as_u16();
                    let mut headers = HashMap::new();
                    for (key, value) in response.headers() {
                        if let Ok(value_str) = value.to_str() {
                            headers.insert(key.to_string(), value_str.to_string());
                        }
                    }

                    let body = response
                        .bytes()
                        .await
                        .map_err(|e| std::io::Error::other(e.to_string()))?
                        .to_vec();

                    return Ok(HttpResponseData {
                        status,
                        headers,
                        body,
                    });
                },
                Err(e) => {
                    last_error = Some(e);

                    // Don't retry on final attempt
                    if attempt < self.max_retries {
                        // Exponential backoff: 2^attempt * 100ms
                        let delay_ms = (1 << attempt) * 100;
                        sleep(TokioDuration::from_millis(delay_ms)).await;
                    }
                },
            }
        }

        Err(std::io::Error::other(format!(
            "Request failed after {} attempts: {}",
            self.max_retries + 1,
            last_error.unwrap()
        )))
    }
}

#[derive(Clone)]
pub struct CacheService {
    cache: Arc<MokaCache<String, String>>,
}

impl CacheService {
    pub fn new(max_capacity: u64) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(5 * 60))
            .time_to_idle(Duration::from_secs(60))
            .build();
        Self {
            cache: Arc::new(cache),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.cache.get(key)
    }

    pub fn set(&self, key: String, value: String) {
        self.cache.insert(key, value);
    }

    pub fn remove(&self, key: &str) {
        self.cache.invalidate(key);
    }

    pub fn delete(&self, key: &str) {
        self.cache.invalidate(key);
    }
}
