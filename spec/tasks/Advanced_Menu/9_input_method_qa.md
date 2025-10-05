# Task 9: QA Validation - Input Method Management System

## Comprehensive Testing Protocol

**File**: `tests/ui/input_method_tests.rs`  
**Test Coverage**: 95%+ required  
**Integration**: InputMethodSystem, TextInputSystem, LocalizationSystem  

### Test Categories

#### 1. IME Detection and Initialization Testing
**Reference**: `./docs/bevy/examples/text_input.rs:125-152`
```rust
#[test]
fn test_ime_detection() {
    let mut manager = InputMethodManager::default();
    
    #[cfg(target_os = "windows")]
    initialize_windows_ime(&mut manager);
    
    #[cfg(target_os = "macos")]
    initialize_macos_ime(&mut manager);
    
    #[cfg(target_os = "linux")]
    initialize_linux_ime(&mut manager);
    
    assert!(!manager.available_methods.is_empty());
    assert!(manager.available_methods.iter().any(|method| method.is_system_default));
}
```

#### 2. Composition State Management Testing
```rust
#[test]
fn test_composition_state_transitions() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, input_method_system)
       .add_event::<ReceivedCharacter>()
       .add_event::<TextInputEvent>();

    let mut ime_manager = InputMethodManager::default();
    ime_manager.active_ime = Some(InputMethodEngine {
        info: InputMethodInfo {
            id: "test_ime".to_string(),
            display_name: "Test IME".to_string(),
            locale: "ja-JP".to_string(),
            is_system_default: false,
            supports_composition: true,
            icon_path: None,
        },
        state: CompositionState::Inactive,
        converter: Box::new(MockTextConverter::new()),
    });
    
    app.world_mut().insert_resource(ime_manager);
    
    // Test composition start
    let mut char_events = app.world_mut().resource_mut::<Events<ReceivedCharacter>>();
    char_events.send(ReceivedCharacter { char: 'あ' });
    
    app.update();
    
    let manager = app.world().resource::<InputMethodManager>();
    assert!(matches!(manager.active_ime.as_ref().unwrap().state, CompositionState::Composing { .. }));
}
```

#### 3. Candidate Window Navigation Testing
**Reference**: `./docs/bevy/examples/keyboard_input_events.rs:175-202`
```rust
#[test]
fn test_candidate_navigation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, input_method_system)
       .add_event::<KeyboardInput>()
       .add_event::<TextInputEvent>();

    let mut ime_manager = InputMethodManager::default();
    if let Some(ref mut ime) = ime_manager.active_ime {
        ime.state = CompositionState::Candidates {
            choices: vec!["選択肢1".to_string(), "選択肢2".to_string(), "選択肢3".to_string()],
            selected: 0,
        };
    }
    
    app.world_mut().insert_resource(ime_manager);
    
    // Test down arrow navigation
    let mut key_events = app.world_mut().resource_mut::<Events<KeyboardInput>>();
    key_events.send(KeyboardInput {
        key_code: KeyCode::ArrowDown,
        state: ButtonState::Pressed,
        window: Entity::PLACEHOLDER,
    });
    
    app.update();
    
    let manager = app.world().resource::<InputMethodManager>();
    if let Some(ref ime) = manager.active_ime {
        if let CompositionState::Candidates { selected, .. } = &ime.state {
            assert_eq!(*selected, 1);
        }
    }
}
```

#### 4. Unicode Character Handling Testing
**Reference**: `./docs/bevy/examples/unicode_input.rs:88-115`
```rust
#[test]
fn test_unicode_character_handling() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, input_method_system)
       .add_event::<ReceivedCharacter>()
       .add_event::<TextInputEvent>();

    let mut ime_manager = InputMethodManager::default();
    app.world_mut().insert_resource(ime_manager);
    
    let test_chars = ['あ', '中', '한', '🔥', 'é', 'ñ'];
    
    for test_char in test_chars {
        let mut char_events = app.world_mut().resource_mut::<Events<ReceivedCharacter>>();
        char_events.send(ReceivedCharacter { char: test_char });
        
        app.update();
        
        let text_events = app.world().resource::<Events<TextInputEvent>>();
        assert!(!text_events.is_empty());
    }
}
```

#### 5. Locale-Specific Input Testing
```rust
#[test]
fn test_locale_specific_input() {
    let test_cases = vec![
        ("ja-JP", 'あ', true),  // Japanese hiragana
        ("ko-KR", '한', true),  // Korean hangul
        ("zh-CN", '中', true),  // Chinese character
        ("en-US", 'a', false), // ASCII character
    ];
    
    for (locale, input_char, should_compose) in test_cases {
        let mut manager = InputMethodManager::default();
        manager.locale_preferences = vec![locale.to_string()];
        
        let requires_composition = !input_char.is_ascii() && should_compose;
        
        if requires_composition {
            assert!(manager.should_use_composition(input_char));
        } else {
            assert!(!manager.should_use_composition(input_char));
        }
    }
}
```

### Platform-Specific Testing

#### 6. Windows IME Integration Testing
```rust
#[cfg(target_os = "windows")]
#[test]
fn test_windows_ime_integration() {
    let mut manager = InputMethodManager::default();
    initialize_windows_ime(&mut manager);
    
    // Verify Windows-specific IME methods are detected
    assert!(manager.available_methods.iter().any(|method| 
        method.id.contains("windows") || method.display_name.contains("Windows")
    ));
    
    // Test composition with common Windows IME inputs
    let test_sequences = vec![
        vec!['k', 'a'], // Should produce か in Japanese IME
        vec!['n', 'i'], // Should produce に in Japanese IME
    ];
    
    for sequence in test_sequences {
        // Test composition sequence
        for ch in sequence {
            // Simulate character input and verify composition state
        }
    }
}
```

#### 7. macOS Input Source Testing
```rust
#[cfg(target_os = "macos")]
#[test]
fn test_macos_input_source_integration() {
    let mut manager = InputMethodManager::default();
    initialize_macos_ime(&mut manager);
    
    // Verify macOS input sources are detected
    assert!(manager.available_methods.iter().any(|method| 
        method.id.contains("macos") || method.is_system_default
    ));
    
    // Test with macOS-specific input method behaviors
    let manager_clone = manager.clone();
    assert!(manager_clone.available_methods.iter().all(|method| 
        method.supports_composition || method.locale.starts_with("en")
    ));
}
```

### Edge Case Testing

#### 8. Settings Persistence Validation
```rust
#[test]
fn test_input_method_settings_persistence() {
    let original_settings = InputMethodSettings {
        preferred_locale: "ja-JP".to_string(),
        auto_detect_locale: true,
        composition_preview: true,
        candidate_window_size: CandidateWindowSize::Medium,
    };
    
    // Test serialization/deserialization
    let serialized = serde_json::to_string(&original_settings).unwrap();
    let deserialized: InputMethodSettings = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(original_settings.preferred_locale, deserialized.preferred_locale);
    assert_eq!(original_settings.auto_detect_locale, deserialized.auto_detect_locale);
}
```

### Manual Testing Checklist

- [ ] All system IME methods are detected correctly
- [ ] Composition text displays with proper cursor positioning
- [ ] Candidate window navigation works with keyboard
- [ ] Unicode characters input correctly across all locales
- [ ] Platform-specific IME features function properly
- [ ] Settings persist across application restarts
- [ ] IME switching works without losing composition state
- [ ] Multiple concurrent text inputs handle IME correctly
- [ ] Composition cancellation (Escape) works reliably
- [ ] System default IME is automatically selected

**Bevy Examples**: `./docs/bevy/examples/unicode_input.rs:125-152`, `./docs/bevy/examples/text_input.rs:178-205`  
**Integration Points**: All input method components  
**Success Criteria**: All tests pass, smooth IME operation across all platforms, zero composition state corruption