# PRODFIX_8B: Implement macOS Icon Extraction

## OBJECTIVE
Implement platform-specific icon extraction for macOS using NSWorkspace and NSImage APIs to enable file/application icons in the UI.

## PRIORITY
**P2 - MEDIUM (UI Degradation)**

## LOCATION
`packages/ecs-ui/src/icons/extraction/platform.rs`

## CURRENT STATE
Line 61 (macOS section) returns None, causing all macOS applications and files to display without icons.

## SUBTASK 1: Add macOS Dependencies
Add required Cocoa framework crates for icon extraction.

**Changes needed in** `packages/ecs-ui/Cargo.toml`:
```toml
[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.25"
objc = "0.2"
core-graphics = "0.23"
```

## SUBTASK 2: Implement NSWorkspace Icon Extraction
Use macOS NSWorkspace API to get application/file icons.

**Changes needed in** `packages/ecs-ui/src/icons/extraction/platform.rs` **around line 61:**
```rust
#[cfg(target_os = "macos")]
fn extract_platform_icon(path: &Path, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    use cocoa::appkit::{NSWorkspace, NSImage};
    use cocoa::foundation::{NSString, NSAutoreleasePool, NSSize};
    use cocoa::base::{id, nil};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        // Create autorelease pool for Objective-C objects
        let pool = NSAutoreleasePool::new(nil);

        // Get shared workspace
        let workspace: id = NSWorkspace::sharedWorkspace(nil);
        if workspace == nil {
            let _: () = msg_send![pool, drain];
            return None;
        }

        // Convert path to NSString
        let path_str = path.to_str()?;
        let ns_path = NSString::alloc(nil).init_str(path_str);
        if ns_path == nil {
            let _: () = msg_send![pool, drain];
            return None;
        }

        // Get icon for file
        let icon: id = msg_send![workspace, iconForFile: ns_path];
        if icon == nil {
            let _: () = msg_send![pool, drain];
            return None;
        }

        // Set icon size
        let ns_size = NSSize::new(size as f64, size as f64);
        let _: () = msg_send![icon, setSize: ns_size];

        // Convert NSImage to RGBA bytes
        let rgba_data = convert_nsimage_to_rgba(icon, size);

        // Drain autorelease pool
        let _: () = msg_send![pool, drain];

        rgba_data
    }
}
```

## SUBTASK 3: Implement NSImage to RGBA Conversion
Create helper function to convert NSImage to RGBA byte array.

**Add new function:**
```rust
#[cfg(target_os = "macos")]
unsafe fn convert_nsimage_to_rgba(nsimage: id, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    use cocoa::appkit::{NSBitmapImageRep, NSDeviceRGBColorSpace};
    use cocoa::foundation::{NSRect, NSPoint, NSSize};
    use objc::{msg_send, sel, sel_impl};
    use core_graphics::base::CGFloat;

    // Lock focus on image
    let _: () = msg_send![nsimage, lockFocus];

    // Create bitmap representation
    let rect = NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(size as f64, size as f64),
    );

    let bitmap_rep: id = msg_send![
        class!(NSBitmapImageRep),
        alloc
    ];

    let bitmap_rep: id = msg_send![
        bitmap_rep,
        initWithFocusedViewRect: rect
    ];

    // Unlock focus
    let _: () = msg_send![nsimage, unlockFocus];

    if bitmap_rep == nil {
        return None;
    }

    // Get bitmap properties
    let bytes_per_row: usize = msg_send![bitmap_rep, bytesPerRow];
    let bits_per_pixel: usize = msg_send![bitmap_rep, bitsPerPixel];
    let has_alpha: bool = msg_send![bitmap_rep, hasAlpha];

    // Get raw bitmap data
    let bitmap_data: *const u8 = msg_send![bitmap_rep, bitmapData];
    if bitmap_data.is_null() {
        return None;
    }

    // Calculate buffer size
    let buffer_size = (size * size * 4) as usize;
    let mut rgba_buffer = vec![0u8; buffer_size];

    // Copy and convert bitmap data to RGBA
    for y in 0..size as usize {
        for x in 0..size as usize {
            let src_offset = y * bytes_per_row + x * (bits_per_pixel / 8);
            let dst_offset = (y * size as usize + x) * 4;

            if bits_per_pixel == 32 {
                // RGBA or ARGB format
                rgba_buffer[dst_offset] = *bitmap_data.add(src_offset); // R
                rgba_buffer[dst_offset + 1] = *bitmap_data.add(src_offset + 1); // G
                rgba_buffer[dst_offset + 2] = *bitmap_data.add(src_offset + 2); // B
                rgba_buffer[dst_offset + 3] = if has_alpha {
                    *bitmap_data.add(src_offset + 3) // A
                } else {
                    255
                };
            } else {
                // Handle other formats if needed
                rgba_buffer[dst_offset] = 0;
                rgba_buffer[dst_offset + 1] = 0;
                rgba_buffer[dst_offset + 2] = 0;
                rgba_buffer[dst_offset + 3] = 255;
            }
        }
    }

    // Release bitmap representation
    let _: () = msg_send![bitmap_rep, release];

    Some((rgba_buffer, size, size))
}
```

## SUBTASK 4: Handle Application Bundle Icons
Special handling for .app bundles to extract high-quality icons.

**Changes needed:**
- Check if path ends with `.app`
- Extract icon from Info.plist `CFBundleIconFile`
- Fall back to generic application icon if not found
- Support both old .icns files and new asset catalogs

**Additional helper:**
```rust
#[cfg(target_os = "macos")]
fn extract_app_bundle_icon(bundle_path: &Path, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    // Try to get icon from bundle
    // Fall back to standard extraction if bundle parsing fails
}
```

## SUBTASK 5: Memory Management and Autorelease
Ensure proper Objective-C memory management with autorelease pools.

**Changes needed:**
- Wrap all Objective-C calls in autorelease pool
- Release explicitly retained objects
- Document autorelease pool usage
- Handle drain errors gracefully

## SUBTASK 6: Error Handling and Logging
Add comprehensive error handling for all Cocoa API calls.

**Changes needed:**
- Log warnings when icon extraction fails
- Handle nil return values from Objective-C methods
- Handle invalid paths gracefully
- Document common failure cases (permissions, missing files, etc.)

## DEFINITION OF DONE
- [ ] macOS dependencies added to Cargo.toml
- [ ] NSWorkspace icon extraction implemented
- [ ] NSImage to RGBA conversion functional
- [ ] Application bundle icons supported
- [ ] Autorelease pool management correct
- [ ] Error handling and logging comprehensive
- [ ] Code compiles on macOS without warnings
- [ ] Icons display in macOS UI

## CONSTRAINTS
- **DO NOT write unit tests** - another team handles testing
- **DO NOT write benchmarks** - another team handles performance
- Focus solely on implementation in ./src
- macOS-specific code only (do not modify Windows or Linux sections)

## RESEARCH NOTES
- NSWorkspace: Primary API for file/app icons on macOS
- NSImage: macOS image representation, can be converted to bitmap
- NSBitmapImageRep: Bitmap representation for pixel data extraction
- Autorelease pools: Required for Objective-C memory management in Rust
- .app bundles: macOS application packages with Info.plist metadata

## DOCUMENTATION LOCATIONS
- Apple NSWorkspace docs: https://developer.apple.com/documentation/appkit/nsworkspace
- Apple NSImage docs: https://developer.apple.com/documentation/appkit/nsimage
- Cocoa crate docs: https://docs.rs/cocoa/latest/cocoa/
- Existing platform abstractions: `packages/ecs-ui/src/icons/`
- objc usage examples: Search codebase for existing Objective-C bridge code
