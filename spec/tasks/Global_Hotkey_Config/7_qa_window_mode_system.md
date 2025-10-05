# Global Hotkey Config - QA Validation for Window Mode System

## Task: Act as an Objective QA Rust Developer

Rate the work performed previously on the window mode system implementation and verify compliance with all specified constraints.

### QA Validation Checklist

#### Core System Verification  
- [ ] Verify NO usage of `unwrap()` in window mode systems
- [ ] Verify NO usage of `expect()` in window mode systems
- [ ] Confirm proper error handling for mode switching failures
- [ ] Validate zero-allocation mode switching implementation
- [ ] Check blazing-fast performance for instant UI adaptation

#### Bevy Integration Compliance
- [ ] Verify button.rs patterns correctly implemented for card selection (lines 24-45)
- [ ] Confirm ui_texture_atlas.rs patterns used for card background management
- [ ] Check ui/ui.rs patterns followed for responsive UI hierarchy
- [ ] Validate ecs/system_param.rs patterns implemented for UI queries

#### File Implementation Verification
- [ ] Confirm `ui/src/systems/window_mode.rs` implements mode switching (lines 1-123)
- [ ] Validate `ui/src/components/mode_selector_cards.rs` implements visual cards (lines 1-89)
- [ ] Check `ui/src/systems/compact_mode_features.rs` implements favorites control (lines 1-56)
- [ ] Verify `ui/src/systems/mode_previews.rs` implements preview generation (lines 1-78)
- [ ] Confirm `ui/src/components/responsive_ui.rs` implements responsive components (lines 1-67)
- [ ] Validate `ui/src/systems/text_size_control.rs` implements text scaling (lines 1-45)

#### Mode Selection Functionality Testing
- [ ] Test visual mode selection cards show correct previews (Default=purple, Compact=gray)
- [ ] Verify click interaction highlights selected mode correctly
- [ ] Confirm instant UI layout adaptation when mode changes
- [ ] Test mode selection persistence across application sessions
- [ ] Validate preview generation accuracy reflects actual interface state

#### Responsive UI Testing
- [ ] Test all UI components correctly adapt to Compact mode layout
- [ ] Verify favorites list visibility control in compact mode
- [ ] Confirm smooth transitions between Default and Compact layouts
- [ ] Test text size toggle affects entire interface consistently
- [ ] Validate ResponsiveUI and ConditionalVisibility components work correctly

#### Integration Testing
- [ ] Test integration with ui/src/ui/components.rs responsive behavior (lines 234-312)
- [ ] Verify integration with ui/src/ui/systems.rs layout system (lines 67-145)
- [ ] Confirm integration with app/src/window/ window management
- [ ] Test settings persistence integration with core settings system

#### Performance Requirements
- [ ] Verify < 100ms for complete mode switching transition
- [ ] Confirm preview generation doesn't impact main UI performance
- [ ] Test smooth animations during mode transitions
- [ ] Verify memory usage stable during repeated mode switching
- [ ] Confirm UI adaptation doesn't cause layout thrashing

#### Visual Design Validation
- [ ] Test Default mode shows full-featured interface with complete functionality
- [ ] Verify Compact mode shows streamlined interface with reduced complexity
- [ ] Confirm "Show favorites in compact mode" checkbox functions correctly
- [ ] Test text size changes apply globally and consistently
- [ ] Validate card selection visual feedback is clear and immediate

#### Error Handling and Edge Cases
- [ ] Test graceful handling of invalid mode configurations
- [ ] Verify fallback behavior when mode switching fails
- [ ] Confirm recovery from corrupted mode preference data
- [ ] Test handling of simultaneous mode change requests
- [ ] Validate proper cleanup when mode components are destroyed

### Acceptance Criteria
All checklist items must pass before proceeding to UI components architecture implementation. Focus on instant mode switching and flawless responsive design.

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.