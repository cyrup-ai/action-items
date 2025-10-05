//! Complete request handling for plugin wrapper
//!
//! Production-grade ActionItem CRUD operations with comprehensive validation,
//! error handling, and search integration.

use std::collections::HashMap;

use action_items_common::plugin_interface::ActionItem;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info};
use uuid::Uuid;

/// Complete request handler with production-grade ActionItem operations
#[derive(Debug)]
pub struct RequestHandler {
    storage_tx: mpsc::Sender<StorageMessage>,
    search_tx: mpsc::Sender<SearchMessage>,
}

impl RequestHandler {
    /// Create new request handler with service channels
    pub fn new(
        storage_tx: mpsc::Sender<StorageMessage>,
        search_tx: mpsc::Sender<SearchMessage>,
    ) -> Self {
        Self {
            storage_tx,
            search_tx,
        }
    }

    /// Handle ActionItem request with comprehensive validation and processing
    pub async fn handle_action_item_request(
        &self,
        request: ActionItemRequest,
    ) -> Result<ActionItemResponse, String> {
        let start_time = std::time::Instant::now();
        debug!("Processing ActionItem request: {:?}", request.action_type());

        let result = match request.action {
            ActionItemAction::Create(item) => self.handle_create_request(item).await,
            ActionItemAction::Update { id, updates } => {
                self.handle_update_request(id, updates).await
            },
            ActionItemAction::Delete(id) => self.handle_delete_request(id).await,
            ActionItemAction::Search(query) => self.handle_search_request(query).await,
            ActionItemAction::BatchCreate(items) => self.handle_batch_create_request(items).await,
        };

        let duration = start_time.elapsed();
        match &result {
            Ok(_) => info!("Request processed successfully in {:?}", duration),
            Err(e) => error!("Request failed after {:?}: {}", duration, e),
        }

        result
    }

    /// Handle create ActionItem request with validation
    async fn handle_create_request(
        &self,
        mut item: ActionItem,
    ) -> Result<ActionItemResponse, String> {
        // Comprehensive validation
        if item.title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        if item.title.len() > 500 {
            return Err("Title too long (max 500 characters)".to_string());
        }
        if let Some(ref description) = item.description
            && description.len() > 2000
        {
            return Err("Description too long (max 2000 characters)".to_string());
        }
        if item.tags.len() > 50 {
            return Err("Too many tags (max 50)".to_string());
        }

        // Generate UUID and timestamps
        item.id = Uuid::new_v4().to_string();
        let now = Utc::now();
        item.created_at = Some(now);
        item.updated_at = Some(now);

        // Sanitize input
        item.title = item.title.trim().to_string();
        if let Some(ref mut desc) = item.description {
            *desc = desc.trim().to_string();
        }
        item.tags = item
            .tags
            .into_iter()
            .map(|tag| tag.trim().to_string())
            .collect();

        // Send to storage service
        self.storage_tx
            .send(StorageMessage::CreateItem(item.clone()))
            .await
            .map_err(|e| format!("Storage service unavailable: {}", e))?;

        info!("Created ActionItem: {} - {}", item.id, item.title);
        Ok(ActionItemResponse::Created(item))
    }

    /// Handle update ActionItem request with validation
    async fn handle_update_request(
        &self,
        id: String,
        updates: ActionItemUpdates,
    ) -> Result<ActionItemResponse, String> {
        // Validate ID format
        Uuid::parse_str(&id).map_err(|_| "Invalid item ID format".to_string())?;

        // Validate update fields
        if let Some(ref title) = updates.title {
            if title.trim().is_empty() {
                return Err("Title cannot be empty".to_string());
            }
            if title.len() > 500 {
                return Err("Title too long (max 500 characters)".to_string());
            }
        }
        if let Some(ref description) = updates.description
            && description.len() > 2000
        {
            return Err("Description too long (max 2000 characters)".to_string());
        }
        if let Some(ref tags) = updates.tags
            && tags.len() > 50
        {
            return Err("Too many tags (max 50)".to_string());
        }

        // Sanitize updates
        let mut sanitized_updates = updates;
        if let Some(ref mut title) = sanitized_updates.title {
            *title = title.trim().to_string();
        }
        if let Some(ref mut description) = sanitized_updates.description {
            *description = description.trim().to_string();
        }
        if let Some(ref mut tags) = sanitized_updates.tags {
            *tags = tags.iter().map(|tag| tag.trim().to_string()).collect();
        }
        sanitized_updates.updated_at = Some(Utc::now());

        // Send update to storage service
        self.storage_tx
            .send(StorageMessage::UpdateItem(id.clone(), sanitized_updates))
            .await
            .map_err(|e| format!("Storage service unavailable: {}", e))?;

        // Retrieve updated item
        let (response_tx, response_rx) = oneshot::channel();
        self.storage_tx
            .send(StorageMessage::GetItem {
                id: id.clone(),
                response: response_tx,
            })
            .await
            .map_err(|e| format!("Storage service unavailable: {}", e))?;

        match response_rx.await {
            Ok(Some(item)) => {
                info!("Updated ActionItem: {} - {}", item.id, item.title);
                Ok(ActionItemResponse::Updated(item))
            },
            Ok(None) => Err("Item not found".to_string()),
            Err(e) => Err(format!("Failed to retrieve updated item: {}", e)),
        }
    }

    /// Handle delete ActionItem request
    async fn handle_delete_request(&self, id: String) -> Result<ActionItemResponse, String> {
        // Validate ID format
        Uuid::parse_str(&id).map_err(|_| "Invalid item ID format".to_string())?;

        // Send delete to storage service
        self.storage_tx
            .send(StorageMessage::DeleteItem(id.clone()))
            .await
            .map_err(|e| format!("Storage service unavailable: {}", e))?;

        info!("Deleted ActionItem: {}", id);
        Ok(ActionItemResponse::Deleted { id })
    }

    /// Handle search ActionItems request with advanced filtering
    async fn handle_search_request(
        &self,
        query: SearchQuery,
    ) -> Result<ActionItemResponse, String> {
        let search_params = SearchParams {
            query: query.query.unwrap_or_default(),
            filters: query.filters.unwrap_or_default(),
            sort: query.sort,
            limit: query.limit.unwrap_or(50).min(1000), // Max 1000 results for performance
            offset: query.offset.unwrap_or(0),
        };

        // Validate search parameters
        if search_params.limit == 0 {
            return Err("Limit must be greater than 0".to_string());
        }
        if search_params.query.len() > 500 {
            return Err("Search query too long (max 500 characters)".to_string());
        }

        // Send search request
        let (response_tx, response_rx) = oneshot::channel();
        self.search_tx
            .send(SearchMessage::Query {
                params: search_params,
                response: response_tx,
            })
            .await
            .map_err(|e| format!("Search service unavailable: {}", e))?;

        match response_rx.await {
            Ok(results) => {
                debug!("Search returned {} results", results.items.len());
                Ok(ActionItemResponse::SearchResults(results))
            },
            Err(e) => Err(format!("Search failed: {}", e)),
        }
    }

    /// Handle batch create request with transaction support
    async fn handle_batch_create_request(
        &self,
        items: Vec<ActionItem>,
    ) -> Result<ActionItemResponse, String> {
        if items.is_empty() {
            return Err("Batch cannot be empty".to_string());
        }
        if items.len() > 100 {
            return Err("Batch size too large (max 100 items)".to_string());
        }

        let mut validated_items = Vec::new();

        // Validate all items first (fail fast)
        for mut item in items {
            if item.title.trim().is_empty() {
                return Err("All items must have non-empty titles".to_string());
            }
            if item.title.len() > 500 {
                return Err("Title too long (max 500 characters)".to_string());
            }
            if let Some(ref description) = item.description
                && description.len() > 2000
            {
                return Err("Description too long (max 2000 characters)".to_string());
            }
            if item.tags.len() > 50 {
                return Err("Too many tags (max 50)".to_string());
            }

            // Set metadata
            item.id = Uuid::new_v4().to_string();
            let now = Utc::now();
            item.created_at = Some(now);
            item.updated_at = Some(now);

            // Sanitize
            item.title = item.title.trim().to_string();
            if let Some(ref mut desc) = item.description {
                *desc = desc.trim().to_string();
            }
            item.tags = item
                .tags
                .into_iter()
                .map(|tag| tag.trim().to_string())
                .collect();

            validated_items.push(item);
        }

        // Send batch create to storage service
        self.storage_tx
            .send(StorageMessage::BatchCreate(validated_items.clone()))
            .await
            .map_err(|e| format!("Storage service unavailable: {}", e))?;

        info!("Batch created {} ActionItems", validated_items.len());
        Ok(ActionItemResponse::BatchCreated(validated_items))
    }
}

/// ActionItem request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItemRequest {
    pub action: ActionItemAction,
}

impl ActionItemRequest {
    pub fn action_type(&self) -> &'static str {
        match self.action {
            ActionItemAction::Create(_) => "create",
            ActionItemAction::Update { .. } => "update",
            ActionItemAction::Delete(_) => "delete",
            ActionItemAction::Search(_) => "search",
            ActionItemAction::BatchCreate(_) => "batch_create",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionItemAction {
    Create(ActionItem),
    Update {
        id: String,
        updates: ActionItemUpdates,
    },
    Delete(String),
    Search(SearchQuery),
    BatchCreate(Vec<ActionItem>),
}

/// ActionItem response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionItemResponse {
    Created(ActionItem),
    Updated(ActionItem),
    Deleted { id: String },
    SearchResults(SearchResults),
    BatchCreated(Vec<ActionItem>),
    Error(String),
}

/// ActionItem update payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItemUpdates {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub priority: Option<String>,
    pub status: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub filters: Option<HashMap<String, String>>,
    pub sort: Option<SortOrder>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Search parameters for internal processing
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchParams {
    pub query: String,
    pub filters: HashMap<String, String>,
    pub sort: Option<SortOrder>,
    pub limit: usize,
    pub offset: usize,
}

/// Search results with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub items: Vec<ActionItem>,
    pub total_count: usize,
    pub query_time_ms: u64,
    pub has_more: bool,
}

/// Sort order options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    CreatedAsc,
    CreatedDesc,
    UpdatedAsc,
    UpdatedDesc,
    TitleAsc,
    TitleDesc,
    PriorityAsc,
    PriorityDesc,
}

/// Storage service message types
#[derive(Debug)]
pub enum StorageMessage {
    CreateItem(ActionItem),
    UpdateItem(String, ActionItemUpdates),
    DeleteItem(String),
    GetItem {
        id: String,
        response: oneshot::Sender<Option<ActionItem>>,
    },
    BatchCreate(Vec<ActionItem>),
}

/// Search service message types
#[derive(Debug)]
pub enum SearchMessage {
    Query {
        params: SearchParams,
        response: oneshot::Sender<SearchResults>,
    },
    IndexItem(ActionItem),
    RemoveItem(String),
    BatchIndex(Vec<ActionItem>),
}
