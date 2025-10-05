use std::collections::HashMap;

use action_items_core::service_bridge::message::{MessageAddress, PluginMessage};

#[test]
fn test_message_address_matching() {
    let addr1 = MessageAddress::new("plugin1");
    let addr2 = MessageAddress::new("plugin1");
    let addr3 = MessageAddress::new("plugin2");
    let broadcast = MessageAddress::broadcast();

    assert!(addr1.matches(&addr2));
    assert!(!addr1.matches(&addr3));
    assert!(broadcast.matches(&addr1));
    assert!(addr1.matches(&broadcast));
}

#[test]
fn test_message_creation() {
    let from = MessageAddress::new("plugin1");
    let to = MessageAddress::new("plugin2");
    let mut params = HashMap::new();
    params.insert(
        "key".to_string(),
        serde_json::Value::String("value".to_string()),
    );

    let message = PluginMessage::request(from, to, "test_method", params);

    assert!(message.is_request());
    assert!(!message.is_response());
    assert_eq!(message.from.plugin_id, "plugin1");
    assert_eq!(message.to.plugin_id, "plugin2");
}

#[test]
fn test_message_validation() {
    let from = MessageAddress::new("plugin1");
    let to = MessageAddress::new("plugin2");
    let message = PluginMessage::request(from, to, "test", HashMap::new());

    assert!(message.validate().is_ok());
}