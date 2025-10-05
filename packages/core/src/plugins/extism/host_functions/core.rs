use action_items_native::{
    ClipboardReadRequest, ClipboardWriteRequest, HttpRequest, NotificationRequest,
    StorageReadRequest, StorageWriteRequest,
};
use crossbeam_channel::Sender as CrossbeamSender;
use extism::Function;

use crate::plugins::interface::CacheService;

/// User data passed to host functions using modern event-driven architecture
#[derive(Clone)]
pub struct ExtismHostUserData {
    pub plugin_id: String,
    pub storage_read_sender: CrossbeamSender<StorageReadRequest>,
    pub storage_write_sender: CrossbeamSender<StorageWriteRequest>,
    pub clipboard_read_sender: CrossbeamSender<ClipboardReadRequest>,
    pub clipboard_write_sender: CrossbeamSender<ClipboardWriteRequest>,
    pub notification_sender: CrossbeamSender<NotificationRequest>,
    pub http_sender: CrossbeamSender<HttpRequest>,
    pub cache_service: CacheService,
}

/// Create all host functions for Extism plugins with zero-allocation event handling
pub fn create_host_functions(user_data_param: ExtismHostUserData) -> Vec<Function> {
    vec![
        super::storage::create_storage_get_async(user_data_param.clone()),
        super::storage::create_storage_set_async(user_data_param.clone()),
        super::storage::create_storage_delete_async(user_data_param.clone()),
        super::clipboard::create_clipboard_read_async(user_data_param.clone()),
        super::clipboard::create_clipboard_write_async(user_data_param.clone()),
        super::notification::create_notification_show_async(user_data_param.clone()),
        super::http::create_http_request_async(user_data_param.clone()),
        super::cache::create_cache_get_async(user_data_param.clone()),
        super::cache::create_cache_set_async(user_data_param.clone()),
        super::cache::create_cache_delete_async(user_data_param),
    ]
}
