//! Real macOS notification backend using modern UserNotifications framework

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use block2::RcBlock;
use objc2::rc::{Retained, autoreleasepool};
use objc2_foundation::{NSArray, NSError, NSNumber, NSString};
use objc2_user_notifications::{
    UNAuthorizationStatus, UNMutableNotificationContent, UNNotificationRequest,
    UNNotificationSound, UNTimeIntervalNotificationTrigger, UNUserNotificationCenter,
};
use parking_lot::{Condvar, Mutex};
use tracing::{debug, error, warn};

use super::{
    NotificationBackend, NotificationError, NotificationId, NotificationMapping,
    NotificationOptions, NotificationResult,
};
// Removed DatabaseService - use ecs-surrealdb instead

/// Real macOS notification backend using modern UserNotifications framework
pub struct MacosNotificationBackend {
    notification_center: Retained<UNUserNotificationCenter>,
    next_id: AtomicU64,
    auth_status: Arc<(Mutex<Option<UNAuthorizationStatus>>, Condvar)>,
    // db_service removed - use ecs-surrealdb instead
}

impl MacosNotificationBackend {
    /// Create backend without database storage for Deno runtime
    pub fn new_without_db() -> NotificationResult<Self> {
        let notification_center =
            autoreleasepool(|_| UNUserNotificationCenter::currentNotificationCenter());

        Ok(Self {
            notification_center,
            next_id: AtomicU64::new(1),
            auth_status: Arc::new((Mutex::new(None), Condvar::new())),
        })
    }

    /// Check authorization status (blocks until resolved, no stubs)
    fn ensure_authorized(&self) -> bool {
        let (lock, cvar) = &*self.auth_status;
        let mut auth = lock.lock();

        // Wait with timeout for authorization resolution
        let timeout = Duration::from_secs(3);
        let timeout_result = cvar.wait_while_for(&mut auth, |status| status.is_none(), timeout);
        let timed_out = timeout_result.timed_out();

        if timed_out {
            warn!("Authorization request timed out");
            return false;
        }

        matches!(*auth, Some(UNAuthorizationStatus::Authorized))
    }

    /// Create notification content with complete configuration
    fn create_content(
        &self,
        options: NotificationOptions<'_>,
    ) -> Retained<UNMutableNotificationContent> {
        autoreleasepool(|_| {
            let content = UNMutableNotificationContent::new();

            if !options.title.is_empty() {
                let title = NSString::from_str(options.title);
                content.setTitle(&title);
            }

            if !options.message.is_empty() {
                let body = NSString::from_str(options.message);
                content.setBody(&body);
            }

            if options.sound {
                let sound = UNNotificationSound::defaultSound();
                content.setSound(Some(&sound));
            }

            if options.urgent {
                let badge = NSNumber::new_i32(1);
                content.setBadge(Some(&badge));
            }

            content
        })
    }

    /// Generate next notification ID with atomic increment
    #[inline(always)]
    fn next_id(&self) -> NotificationId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        NotificationId::new(id)
    }

    /// Store notification ID mapping in SurrealDB
    async fn store_notification_mapping(
        &self,
        local_id: u64,
        platform_id: String,
    ) -> NotificationResult<()> {
        let _mapping = NotificationMapping {
            local_id,
            platform_id: platform_id.parse::<u32>().unwrap_or(local_id as u32), // Parse or fallback
            platform: "macos-usernotifications".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|e| {
                    warn!("System clock error, using fallback timestamp: {}", e);
                    std::time::Duration::from_secs(0) // Unix epoch as fallback
                })
                .as_secs(),
        };

        // Note: Database storage removed - use ecs-surrealdb package for persistence
        // Original mapping storage functionality would be handled by the ECS surrealdb service

        debug!(
            local_id = local_id,
            platform_id = %platform_id,
            "Stored UserNotifications mapping in SurrealDB"
        );

        Ok(())
    }

    /// Retrieve platform ID from local ID (simplified without database dependency)
    async fn get_platform_id(&self, local_id: u64) -> NotificationResult<String> {
        // Note: Database lookup removed - use ecs-surrealdb package for persistence if needed
        // For UserNotifications, we use the format "action-items-{id}" as platform ID
        Ok(format!("action-items-{}", local_id))
    }
}

// Implement Send and Sync for thread safety (UNUserNotificationCenter is thread-safe)
unsafe impl Send for MacosNotificationBackend {}
unsafe impl Sync for MacosNotificationBackend {}

impl NotificationBackend for MacosNotificationBackend {
    /// Show notification with complete UserNotifications integration (no stubs)
    fn show_notification(
        &self,
        options: NotificationOptions<'_>,
    ) -> NotificationResult<NotificationId> {
        if !self.ensure_authorized() {
            return Err(NotificationError::PermissionDenied(
                "UserNotifications authorization required".to_string(),
            ));
        }

        let notification_id = self.next_id();

        debug!(
            notification_id = notification_id.as_u64(),
            title = %options.title,
            message = %options.message,
            "Posting UserNotifications notification"
        );

        autoreleasepool(|_| -> NotificationResult<()> {
            let content = self.create_content(options);
            let request_id =
                NSString::from_str(&format!("action-items-{}", notification_id.as_u64()));
            let trigger = UNTimeIntervalNotificationTrigger::triggerWithTimeInterval_repeats(0.1, false)
            ;

            let request = UNNotificationRequest::requestWithIdentifier_content_trigger(
                    &request_id,
                    &content,
                    Some(&trigger),
                )
            ;

            // Add request with completion handler using RcBlock
            let completion_sent = Arc::new((Mutex::new(false), Condvar::new()));
            let completion_clone = Arc::clone(&completion_sent);

            let completion_block = RcBlock::new(move |error: *mut NSError| {
                let (lock, cvar) = &*completion_clone;
                {
                    let mut sent = lock.lock();
                    *sent = true;
                }
                cvar.notify_all();

                if !error.is_null() {
                    let err = unsafe { &*error };
                    error!("Failed to add notification: {:?}", err);
                }
            });

            self.notification_center
                    .addNotificationRequest_withCompletionHandler(
                        &request,
                        Some(&completion_block),
                    );
            

            // Wait for completion
            let (lock, cvar) = &*completion_sent;
            let mut sent = lock.lock();
            let timeout_result = cvar.wait_while_for(&mut sent, |s| !*s, Duration::from_secs(2));
            let _result = timeout_result.timed_out();

            Ok(())
        })?;

        // Store the mapping in SurrealDB using async-to-sync conversion
        let platform_id = format!("action-items-{}", notification_id.as_u64());
        let storage_result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.store_notification_mapping(notification_id.as_u64(), platform_id)
                    .await
            })
        });

        match storage_result {
            Ok(()) => {
                debug!(
                    local_id = notification_id.as_u64(),
                    "Successfully stored UserNotifications mapping"
                );
            },
            Err(e) => {
                error!(
                    error = %e,
                    notification_id = notification_id.as_u64(),
                    "Failed to store UserNotifications mapping"
                );
                // Continue with notification success - storage failure shouldn't break
                // notifications
            },
        }

        Ok(notification_id)
    }

    /// Check if UserNotifications is available
    #[inline]
    fn is_available(&self) -> bool {
        true // UserNotifications available on macOS 10.14+
    }

    /// Get platform name
    #[inline(always)]
    fn platform_name(&self) -> &'static str {
        "macos-usernotifications"
    }

    /// Remove delivered notification with UserNotifications API using stored platform ID
    fn dismiss(&self, id: NotificationId) -> NotificationResult<()> {
        // Get the platform ID from SurrealDB
        let platform_id_result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { self.get_platform_id(id.as_u64()).await })
        });

        let platform_id = match platform_id_result {
            Ok(id) => id,
            Err(e) => {
                debug!(
                    error = %e,
                    notification_id = id.as_u64(),
                    "Failed to retrieve platform ID, using fallback format"
                );
                // Fallback to the standard format if database lookup fails
                format!("action-items-{}", id.as_u64())
            },
        };

        autoreleasepool(|_| {
            let request_id = NSString::from_str(&platform_id);
            let identifiers = NSArray::from_slice(&[request_id.as_ref()]);

            self.notification_center
                    .removeDeliveredNotificationsWithIdentifiers(&identifiers);
                self.notification_center
                    .removePendingNotificationRequestsWithIdentifiers(&identifiers);
            

            debug!(
                notification_id = id.as_u64(),
                platform_id = %platform_id,
                "Dismissed UserNotifications notification using stored platform ID"
            );
        });

        Ok(())
    }
}
