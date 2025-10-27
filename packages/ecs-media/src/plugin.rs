use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on};
use bevy::tasks::futures_lite::future;
use std::path::PathBuf;
use std::sync::Arc;

use crate::events::{MediaRequest, MediaResponse};
use crate::manager::MediaManager;

/// Media configuration resource
#[derive(Resource, Clone)]
pub struct MediaConfig {
    pub storage_base_path: PathBuf,
    pub max_file_size: u64,  // bytes
}

impl Default for MediaConfig {
    fn default() -> Self {
        Self {
            storage_base_path: PathBuf::from("./data/media"),
            max_file_size: 100 * 1024 * 1024,  // 100MB
        }
    }
}

/// Media service resource
#[derive(Resource)]
pub struct MediaService {
    manager: Arc<MediaManager>,
}

/// Component for async media tasks
#[derive(Component)]
pub struct MediaTask(pub Task<MediaResponse>);

/// Bevy plugin for media operations
pub struct MediaPlugin {
    config: MediaConfig,
}

impl MediaPlugin {
    pub fn new(config: MediaConfig) -> Self {
        Self { config }
    }
}

impl Default for MediaPlugin {
    fn default() -> Self {
        Self::new(MediaConfig::default())
    }
}

impl Plugin for MediaPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone())
            .add_event::<MediaRequest>()
            .add_event::<MediaResponse>()
            .add_systems(Startup, initialize_media_service)
            .add_systems(Update, (
                handle_media_requests,
                handle_media_tasks,
            ).chain());
    }
}

/// Initialize media service and database schema
fn initialize_media_service(
    mut commands: Commands,
    config: Res<MediaConfig>,
    db: Res<action_items_ecs_surrealdb::DatabaseService>,
) {
    // Initialize schema
    let db_clone = db.clone();
    tokio::spawn(async move {
        if let Err(e) = MediaManager::initialize_schema(&db_clone).await {
            error!("Failed to initialize media schema: {}", e);
        } else {
            info!("Media schema initialized successfully");
        }
    });
    
    // Create manager
    let manager = Arc::new(MediaManager::new(
        config.storage_base_path.clone(),
        config.max_file_size,
    ));
    
    commands.insert_resource(MediaService { manager });
    
    info!("Media ECS service initialized");
}

/// Handle incoming media requests
fn handle_media_requests(
    mut commands: Commands,
    mut events: EventReader<MediaRequest>,
    service: Res<MediaService>,
    db: Res<action_items_ecs_surrealdb::DatabaseService>,
) {
    for event in events.read() {
        let manager = service.manager.clone();
        let db_clone = db.clone();
        
        let task = match event {
            MediaRequest::UploadMedia {
                operation_id,
                user_id,
                conversation_id,
                file_path,
                description,
                requester,
            } => {
                let op_id = *operation_id;
                let user_id = user_id.clone();
                let conv_id = conversation_id.clone();
                let file_path = file_path.clone();
                let desc = description.clone();
                let req = *requester;
                
                AsyncComputeTaskPool::get().spawn(async move {
                    let result = manager.upload_media(
                        &db_clone,
                        op_id,
                        user_id,
                        conv_id,
                        &file_path,
                        desc,
                    ).await;
                    
                    MediaResponse::UploadComplete {
                        operation_id: op_id,
                        requester: req,
                        result,
                    }
                })
            },
            
            MediaRequest::GetMedia {
                operation_id,
                media_id,
                requester,
            } => {
                let op_id = *operation_id;
                let media_id = media_id.clone();
                let req = *requester;
                
                AsyncComputeTaskPool::get().spawn(async move {
                    let result = manager.get_media(&db_clone, &media_id).await;
                    
                    MediaResponse::MediaRetrieved {
                        operation_id: op_id,
                        requester: req,
                        result,
                    }
                })
            },
            
            MediaRequest::UpdateDescription {
                operation_id,
                media_id,
                description,
                requester,
            } => {
                let op_id = *operation_id;
                let media_id = media_id.clone();
                let desc = description.clone();
                let req = *requester;
                
                AsyncComputeTaskPool::get().spawn(async move {
                    let result = manager.update_description(&db_clone, &media_id, desc).await;
                    
                    MediaResponse::DescriptionUpdated {
                        operation_id: op_id,
                        requester: req,
                        result,
                    }
                })
            },
            
            MediaRequest::DeleteMedia {
                operation_id,
                media_id,
                requester,
            } => {
                let op_id = *operation_id;
                let media_id = media_id.clone();
                let req = *requester;
                
                AsyncComputeTaskPool::get().spawn(async move {
                    let result = manager.delete_media(&db_clone, &media_id).await;
                    
                    MediaResponse::MediaDeleted {
                        operation_id: op_id,
                        requester: req,
                        result,
                    }
                })
            },
            
            MediaRequest::ListConversationMedia {
                operation_id,
                conversation_id,
                requester,
            } => {
                let op_id = *operation_id;
                let conv_id = conversation_id.clone();
                let req = *requester;
                
                AsyncComputeTaskPool::get().spawn(async move {
                    let result = manager.list_conversation_media(&db_clone, &conv_id).await;
                    
                    MediaResponse::ConversationMediaListed {
                        operation_id: op_id,
                        requester: req,
                        result,
                    }
                })
            },
        };
        
        commands.spawn(MediaTask(task));
    }
}

/// Handle completed media tasks
fn handle_media_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut MediaTask)>,
    mut response_writer: EventWriter<MediaResponse>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(response) = block_on(future::poll_once(&mut task.0)) {
            response_writer.write(response);
            commands.entity(entity).despawn();
        }
    }
}
