# Task: Process Remaining "For Now" Comments

## Objective
Eliminate the last five "for now" placeholders in the workspace by replacing each with production-ready logic that aligns with existing service patterns and plugin architecture. Capture all design choices needed to complete the work so implementation can proceed without re-triage.

## Priority
P2 – High. Clearing these markers unblocks downstream platform stabilization and keeps the codebase free of temporary logic.

## Current Audit Snapshot
- Remaining "for now" markers: **5** (out of 39 originally)
- Impacted packages: `plugin-native`, `core`, `ecs-fetch`, `ecs-permissions`
- No new scan required until the changes below land; re-run `rg "for now" packages` once work completes to confirm zero matches.

## Research Highlights
- `NativePlugin::background_refresh` still spawns an empty task; real handlers already exist in the builder and adapters and just need to be wired through the shared trait default.[`packages/plugin-native/src/native.rs`](../packages/plugin-native/src/native.rs#L1-L62)[`packages/plugin-native/src/builder.rs`](../packages/plugin-native/src/builder.rs#L1-L210)[`packages/core/src/plugins/discovery/adapter.rs`](../packages/core/src/plugins/discovery/adapter.rs#L90-L214)
- Streaming chunk processing currently returns raw bytes; metadata scaffolding (hashes, size checks, statistics) is available and can be extended to include decompression/transform hooks the middleware module already supports.[`packages/ecs-fetch/src/streaming.rs`](../packages/ecs-fetch/src/streaming.rs#L488-L569)[`packages/ecs-fetch/src/middleware/mod.rs`](../packages/ecs-fetch/src/middleware/mod.rs#L260-L360)
- WASM callbacks in the bridge already resolve runtimes and spawn tasks, but rely on mock handlers. Extending the runtime match arms and surfacing structured errors will finish the integration without redesigning the pipeline.[`packages/core/src/plugins/bridge/handlers/processor.rs`](../packages/core/src/plugins/bridge/handlers/processor.rs#L1-L210)[`packages/core/src/plugins/bridge/types.rs`](../packages/core/src/plugins/bridge/types.rs#L1-L120)
- The shared native interface mirrors the plugin-native trait and should document the production background-refresh flow rather than leaving a stub.[`packages/core/src/plugins/interface/native.rs`](../packages/core/src/plugins/interface/native.rs#L1-L70)
- Windows admin requests already query administrator membership; we need to clarify fallbacks so callers know when to prompt for elevation instead of silently accepting current status.[`packages/ecs-permissions/src/platforms/windows/system.rs`](../packages/ecs-permissions/src/platforms/windows/system.rs#L1-L200)

## Detailed Implementation Plan

### B1 – Activate Background Refresh Tasks in `plugin-native`
**Files:**
- `packages/plugin-native/src/native.rs`
- `packages/plugin-native/src/builder.rs`
- `packages/core/src/plugins/discovery/adapter.rs`

**Steps:**
1. Replace the default `background_refresh` stub in `NativePlugin` with logic that checks capability flags and routes to the plugin-specific handler rather than returning `Ok(())`. The builder already exposes optional refresh handlers—reuse that wiring instead of inventing new futures.[`packages/plugin-native/src/native.rs`](../packages/plugin-native/src/native.rs#L43-L62)[`packages/plugin-native/src/builder.rs`](../packages/plugin-native/src/builder.rs#L160-L210)
2. Ensure `PluginBuilder::background_refresh` tasks surface actual refresh work (e.g., invoking registered handler or short-circuiting when capability disabled). Tie this into existing `refresh_handler` storage to avoid duplicating state.
3. Confirm the discovery adapter forwards refresh invocations into the native plugin and propagates errors back through the async task so callers can observe failure paths.[`packages/core/src/plugins/discovery/adapter.rs`](../packages/core/src/plugins/discovery/adapter.rs#L187-L214)
4. Document that native plugins must either register a refresh handler or explicitly return `PluginCapabilities::background_refresh = false`, clarifying expected behaviour in the trait docs (see B4).

**Outcome:** Background refresh executes real plugin-provided work, with unified task spawning across builder, native trait, and discovery adapter.

### B2 – Complete Streaming Chunk Processing in `ecs-fetch`
**Files:**
- `packages/ecs-fetch/src/streaming.rs`
- `packages/ecs-fetch/src/middleware/mod.rs`

**Steps:**
1. Extract or reuse decompression helpers from `MiddlewareProcessor::decompress_response` so chunk processing can decode compressed payloads without duplicating algorithm handling. Consider moving shared routines into a small utility module if direct reuse requires mutable state separation.[`packages/ecs-fetch/src/middleware/mod.rs`](../packages/ecs-fetch/src/middleware/mod.rs#L296-L343)
2. In `StreamHandler::process_chunk_data`, add per-chunk transformations: handle decompression (when chunk metadata indicates compression), perform charset/encoding normalization where headers demand it, and enrich `ChunkMetadata` with `compressed_size`, `encoding`, and integrity fields. Stop returning a raw passthrough value.[`packages/ecs-fetch/src/streaming.rs`](../packages/ecs-fetch/src/streaming.rs#L488-L569)
3. When transformations fail, fall back to the original bytes while recording the failure in logs and metadata (e.g., note `encoding = Some("identity")` and include an error flag) so consumers can react without crashing.
4. Update `StreamingStats` increments only after successful enqueue to avoid double-counting on retries. Preserve the existing backpressure timeout handling.

**Outcome:** Streaming chunks arrive preprocessed with accurate metadata, enabling downstream consumers to rely on consistent encodings and size data.

### B3 – Solidify WASM Bridge Callback Processing in `core`
**Files:**
- `packages/core/src/plugins/bridge/handlers/processor.rs`
- `packages/core/src/plugins/bridge/types.rs`

**Steps:**
1. Replace the mock match arms in `WasmRuntime::call_function` with production handlers: parse `process_data` payloads via `serde_json`, safely handle binary blobs, and extend support to `init`, `cleanup`, `validate_input`, `transform_data`, or other plugin-declared callbacks. Align response payloads with the bridge’s `ServiceResponse::WasmCallback` contract.[`packages/core/src/plugins/bridge/handlers/processor.rs`](../packages/core/src/plugins/bridge/handlers/processor.rs#L1-L210)
2. Surface descriptive error messages (e.g., unknown function, deserialize failure) so callers can log and bubble issues without panics. Ensure `ServiceResponse::WasmCallback` receives `Err(String)` entries instead of generic failures.
3. Integrate with bridge statistics/types as needed—if new callback categories are introduced, update `ServiceRequest::WasmCallback` documentation to reflect the expected function set.[`packages/core/src/plugins/bridge/types.rs`](../packages/core/src/plugins/bridge/types.rs#L13-L102)
4. Keep task spawning via `AsyncComputeTaskPool` but gate runtime acquisition with real plugin registry lookups when available; for now, document the mock fallback clearly in code comments until registry integration lands.

**Outcome:** WASM callbacks execute real plugin logic and deliver structured responses through the bridge, removing the placeholder flow.

### B4 – Align Native Plugin Interface Documentation and Defaults
**File:** `packages/core/src/plugins/interface/native.rs`

**Steps:**
1. Update the trait documentation to mirror the behaviour implemented in B1, including examples of using `AsyncComputeTaskPool` and explaining when to override the default `background_refresh`.
2. Remove stale commentary suggesting the default is temporary. If default behaviour now inspects capabilities or delegates to handlers, summarize that in doc comments so implementors know what happens without overrides.[`packages/core/src/plugins/interface/native.rs`](../packages/core/src/plugins/interface/native.rs#L1-L70)
3. Cross-reference the trait with `plugin-native` so both sides explain the same lifecycle expectations (init, command, action, refresh, cleanup).

**Outcome:** Documentation and defaults consistently represent the production contract for native plugins, preventing future “for now” stubs.

### B5 – Clarify Windows Permission Elevation Flow in `ecs-permissions`
**File:** `packages/ecs-permissions/src/platforms/windows/system.rs`

**Steps:**
1. Refine `request_admin_access` so non-admin tokens return `PermissionStatus::PromptRequired` (or other informative status) and document when the caller should surface a UAC prompt. Avoid returning the current status without guidance.[`packages/ecs-permissions/src/platforms/windows/system.rs`](../packages/ecs-permissions/src/platforms/windows/system.rs#L150-L200)
2. Factor shared administrator SID checks into a helper to keep `check_admin_access` and `request_admin_access` in sync. Ensure both pathways handle API failures consistently and log meaningful diagnostics.
3. Describe in comments how this aligns with macOS/Linux implementations so future contributors know the intended cross-platform contract.

**Outcome:** Permission APIs now communicate actionable status to callers, removing ambiguity around elevation handling.

## Definition of Done
1. Code changes implementing B1–B5 are merged, and running `rg "for now" packages` returns no matches in maintained code.
2. Task documentation (this file) reflects new audit totals and behaviour; no temporary language remains in affected modules.
3. Stakeholders confirm background refresh, streaming, WASM callbacks, and Windows admin prompts behave as described via manual verification or existing monitoring (no new tests required).
