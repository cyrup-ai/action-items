# AI Menu 3 - Experimental Features Management System

## Task: Implement Feature Flag System and Experimental Controls

### File: `ui/src/ai/experimental/mod.rs` (new file)

Create experimental features system with toggles, feedback collection, and rollback capabilities.

### Implementation Requirements

#### Feature Flag Management
- File: `ui/src/ai/experimental/feature_flags.rs` (new file, line 1-134)
- Toggle controls for Auto Models, Chat Branching, Custom Providers
- MCP HTTP Servers and AI Extensions for Ollama Models toggles
- Dynamic feature activation with runtime configuration
- Bevy Example Reference: [`ui/button.rs`](../../../docs/bevy/examples/ui/button.rs) - Toggle button patterns

#### Experimental Features System
- File: `ui/src/ai/experimental/features.rs` (new file, line 1-156)
- Auto Models: Intelligent model selection based on context
- Chat Branching: Conversation flow management
- Custom Providers: User-defined AI provider integration
- Rollback capabilities for problematic features

#### Feature State Management
```rust
#[derive(Resource, Debug, Clone)]
pub struct ExperimentalFeaturesConfig {
    pub auto_models: bool,
    pub chat_branching: bool,
    pub custom_providers: bool,
    pub mcp_http_servers: bool,
    pub ai_extensions_ollama: bool,
    pub feature_feedback: HashMap<String, FeatureFeedback>,
}
```

#### Feedback Collection System
- File: `ui/src/ai/experimental/feedback.rs` (new file, line 1-67)
- User feedback collection for experimental features
- Usage analytics and performance monitoring
- Privacy-conscious data collection with opt-in consent

### Architecture Notes
- Safe feature activation with rollback capabilities
- Telemetry collection for feature improvement
- Integration with existing AI systems
- Performance monitoring for experimental features

### Integration Points
- All AI subsystems for feature activation
- Analytics system for usage tracking
- User preference storage for feature states

### Bevy Example References
- **Toggle Controls**: [`ui/button.rs`](../../../docs/bevy/examples/ui/button.rs) - Toggle patterns
- **Feature Management**: [`ecs/system_param.rs`](../../../docs/bevy/examples/ecs/system_param.rs) - Feature resources

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA.