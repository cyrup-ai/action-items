//! Tests for plugins/service_bridge_integration/message_translator.rs

use std::collections::HashMap;
use serde_json::Value;
use action_items_core::plugins::service_bridge_integration::message_translator::{
    translate_message_to_ecs_events,
    translate_ecs_response_to_message,
};
use action_items_core::plugins::service_bridge_integration::types::{
    EcsEventType,
    EcsEventResponse,
    OperationId,
};
use ecs_service_bridge::events::PluginMessageEvent;

#[test]
fn test_clipboard_read_translation() {
    let mut payload = HashMap::new();
    payload.insert("format".to_string(), Value::String("text".to_string()));

    let message_event = PluginMessageEvent {
        from: "test_plugin".to_string(),
        to: "service_bridge".to_string(),
        message_type: "clipboard_read".to_string(),
        payload: Value::Object(serde_json::Map::from_iter(payload)),
        correlation_id: None,
        priority: ecs_service_bridge::events::MessagePriority::Normal,
        plugin_id: "test_plugin".to_string(),
        timestamp: ecs_service_bridge::types::TimeStamp::now(),
        request_id: uuid::Uuid::new_v4().to_string(),
    };

    let operation_id = OperationId::new();
    let result = translate_message_to_ecs_events(&message_event, operation_id);

    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        EcsEventType::ClipboardRequest(req) => {
            assert_eq!(req.requester, "test_plugin");
        },
        other => {
            assert!(false, "Expected ClipboardRequest event, got: {:?}", other);
        },
    }
}

#[test]
fn test_notification_show_translation() {
    let mut payload = HashMap::new();
    payload.insert("title".to_string(), Value::String("Test Title".to_string()));
    payload.insert(
        "message".to_string(),
        Value::String("Test Message".to_string()),
    );
    payload.insert("duration".to_string(), Value::Number(5.into()));

    let message_event = PluginMessageEvent {
        from: "test_plugin".to_string(),
        to: "service_bridge".to_string(),
        message_type: "notification_show".to_string(),
        payload: serde_json::Value::Object(serde_json::Map::from_iter(payload)),
        correlation_id: None,
        priority: ecs_service_bridge::events::MessagePriority::Normal,
        plugin_id: "test_plugin".to_string(),
        timestamp: ecs_service_bridge::types::TimeStamp::now(),
        request_id: uuid::Uuid::new_v4().to_string(),
    };

    let operation_id = OperationId::new();
    let result = translate_message_to_ecs_events(&message_event, operation_id);

    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        EcsEventType::NotificationRequest(req) => {
            assert_eq!(req.title, "Test Title");
            assert_eq!(req.message, "Test Message");
            assert_eq!(req.duration, Some(std::time::Duration::from_secs(5)));
        },
        other => {
            assert!(false, "Expected NotificationRequest event, got: {:?}", other);
        },
    }
}

#[test]
fn test_response_translation() {
    let operation_id = OperationId::new();
    let response = EcsEventResponse::ClipboardData {
        data: "test data".to_string(),
        format: "text".to_string(),
    };

    let result = translate_ecs_response_to_message(
        response,
        operation_id.clone(),
        "test_plugin".to_string(),
    );

    assert!(result.is_ok());
    let message = result.unwrap();
    assert_eq!(message.to, "test_plugin");
    assert_eq!(message.message_type, "clipboard_response");
    assert_eq!(message.payload.get("success"), Some(&Value::Bool(true)));
    assert_eq!(
        message.payload.get("data"),
        Some(&Value::String("test data".to_string()))
    );
}
