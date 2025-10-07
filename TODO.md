# TODO: Fix All Errors and Warnings

**Status: Starting Analysis**

## Summary
- **Errors Found**: 13 (3 packages blocked compilation)
- **Warnings Found**: TBD (will be counted after errors are fixed)

---

## ERRORS (Priority 1 - Blocking Compilation)

### ecs-ui (4 errors - BLOCKING)

1. [ ] packages/ecs-ui/tests/bezier_easing_test.rs:21:13 - manual `RangeInclusive::contains` implementation
   - Change: `result >= 0.0 && result <= 1.0` → `(0.0..=1.0).contains(&result)`

2. [ ] packages/ecs-ui/tests/bezier_easing_test.rs:38:13 - manual `RangeInclusive::contains` implementation
   - Change: `result >= 0.0 && result <= 1.0` → `(0.0..=1.0).contains(&result)`

3. [ ] packages/ecs-ui/tests/bezier_easing_test.rs:55:13 - manual `RangeInclusive::contains` implementation
   - Change: `result >= 0.0 && result <= 1.0` → `(0.0..=1.0).contains(&result)`

4. [ ] packages/ecs-ui/tests/bezier_easing_test.rs:81:13 - manual `RangeInclusive::contains` implementation
   - Change: `result >= 0.0 && result <= 1.0` → `(0.0..=1.0).contains(&result)`

### ecs-permissions (6 errors - BLOCKING)

5. [ ] packages/ecs-permissions/src/wizard/events.rs:146:17 - wrong_self_convention for `is_final`
   - Issue: `is_*` methods should take `&self` not `self`

6. [ ] packages/ecs-permissions/src/wizard/events.rs:149:18 - wrong_self_convention for `is_active`
   - Issue: `is_*` methods should take `&self` not `self`

7. [ ] packages/ecs-permissions/src/wizard/events.rs:152:19 - wrong_self_convention for `is_granted`
   - Issue: `is_*` methods should take `&self` not `self`

8. [ ] packages/ecs-permissions/src/wizard/systems/permissions.rs:598:1 - too_many_arguments (8/7)
   - Function: `monitor_ecs_permission_events`
   - Need to refactor into parameter struct or reduce arguments

9. [ ] packages/ecs-permissions/src/wizard/systems/ui_updates.rs:165:21 - type_complexity
   - Query type: `Query<(Entity, &mut Text), (With<Name>, Without<PermissionCard>)>`
   - Need to create type alias

10. [ ] packages/ecs-permissions/src/wizard/systems/ui_updates.rs:933:23 - type_complexity
    - Query type: `Query<(&mut Text, &mut TextColor), (With<PermissionCardStatus>, Without<PermissionCardTitle>)>`
    - Need to create type alias

### ecs-fetch (3 errors - BLOCKING)

11. [ ] packages/ecs-fetch/src/systems.rs:28:1 - too_many_arguments (12/7)
    - Function: `process_http_requests_system`
    - Need to refactor into parameter struct or reduce arguments

12. [ ] packages/ecs-fetch/src/systems.rs:211:1 - too_many_arguments (8/7)
    - Function: `process_http_responses_system`
    - Need to refactor into parameter struct or reduce arguments

13. [ ] packages/ecs-fetch/src/tracing.rs:69:5 - too_many_arguments (8/7)
    - Method: `start_request_trace`
    - Need to refactor into parameter struct or reduce arguments

---

## WARNINGS (Priority 2 - Will be discovered after fixing errors)

_Note: Additional warnings from `cargo check` will be documented after the blocking errors are resolved._

---

## Work Plan

### Phase 1: Fix Blocking Errors (13 items)
1. Fix ecs-ui test assertions (4 simple replacements)
2. Fix ecs-permissions trait methods (3 self reference changes)
3. Refactor ecs-permissions complex queries (2 type aliases)
4. Refactor ecs-permissions too many arguments (1 function)
5. Refactor ecs-fetch too many arguments (3 functions)

### Phase 2: Identify and Fix Warnings
- Run `cargo check --workspace --all-targets` 
- Document all warnings
- Fix each warning following same pattern

### Phase 3: Quality Assurance
- Each fix will be followed by a QA evaluation
- Any fix scoring < 9/10 will be redone
- Final verification with clean `cargo check` and `cargo clippy`

---

## Notes

- All fixes must be production quality
- No `#[allow(...)]` annotations without written approval from David Maple
- No stubbing - errors are better than incomplete implementations
- Zero allocations, non-blocking, async-first where possible
- Using sequential thinking for each complex fix
