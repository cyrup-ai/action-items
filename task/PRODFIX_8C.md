# PRODFIX_8C: Implement Linux Icon Extraction

## OBJECTIVE
Implement platform-specific icon extraction for Linux using GTK/GIO APIs to enable file/application icons in the UI.

## PRIORITY
**P2 - MEDIUM (UI Degradation)**

## LOCATION
`packages/ecs-ui/src/icons/extraction/platform.rs`

## CURRENT STATE
Line 75 (Linux section) returns None, causing all Linux applications and files to display without icons.

## SUBTASK 1: Add Linux Dependencies
Add required GTK/GIO crates for icon extraction.

**Changes needed in** `packages/ecs-ui/Cargo.toml`:
```toml
[target.'cfg(target_os = "linux")'.dependencies]
gtk = { version = "0.18", features = ["v3_24"] }
gio = "0.18"
gdk-pixbuf = "0.18"
```

## SUBTASK 2: Implement GIO Icon Extraction
Use GIO's file info API to get icon information from files.

**Changes needed in** `packages/ecs-ui/src/icons/extraction/platform.rs` **around line 75:**
```rust
#[cfg(target_os = "linux")]
fn extract_platform_icon(path: &Path, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    use gtk::prelude::*;
    use gio::prelude::*;

    // Initialize GTK if not already initialized
    if gtk::init().is_err() {
        warn!("Failed to initialize GTK for icon extraction");
        return None;
    }

    // Create GFile for the path
    let file = gio::File::for_path(path);

    // Query file info with icon attribute
    let file_info = match file.query_info(
        "standard::icon",
        gio::FileQueryInfoFlags::NONE,
        gio::Cancellable::NONE,
    ) {
        Ok(info) => info,
        Err(e) => {
            warn!("Failed to query file info for {:?}: {}", path, e);
            return None;
        }
    };

    // Get icon from file info
    let icon = match file_info.icon() {
        Some(icon) => icon,
        None => {
            warn!("No icon found for file: {:?}", path);
            return None;
        }
    };

    // Get default icon theme
    let icon_theme = match gtk::IconTheme::default() {
        Some(theme) => theme,
        None => {
            warn!("Failed to get default icon theme");
            return None;
        }
    };

    // Lookup icon in theme
    let icon_info = match icon_theme.lookup_by_gicon(
        &icon,
        size as i32,
        gtk::IconLookupFlags::empty(),
    ) {
        Some(info) => info,
        None => {
            warn!("Failed to lookup icon in theme for: {:?}", path);
            return None;
        }
    };

    // Load icon as pixbuf
    let pixbuf = match icon_info.load_icon() {
        Ok(buf) => buf,
        Err(e) => {
            warn!("Failed to load icon: {}", e);
            return None;
        }
    };

    // Convert pixbuf to RGBA bytes
    convert_pixbuf_to_rgba(&pixbuf)
}
```

## SUBTASK 3: Implement GdkPixbuf to RGBA Conversion
Create helper function to convert GdkPixbuf to RGBA byte array.

**Add new function:**
```rust
#[cfg(target_os = "linux")]
fn convert_pixbuf_to_rgba(pixbuf: &gdk_pixbuf::Pixbuf) -> Option<(Vec<u8>, u32, u32)> {
    let width = pixbuf.width() as u32;
    let height = pixbuf.height() as u32;
    let n_channels = pixbuf.n_channels();
    let has_alpha = pixbuf.has_alpha();
    let rowstride = pixbuf.rowstride() as usize;

    // Get pixel data
    let pixels = unsafe {
        let ptr = pixbuf.pixels();
        std::slice::from_raw_parts(ptr.as_ptr(), rowstride * height as usize)
    };

    // Allocate RGBA buffer
    let buffer_size = (width * height * 4) as usize;
    let mut rgba_buffer = vec![0u8; buffer_size];

    // Convert to RGBA format
    for y in 0..height as usize {
        for x in 0..width as usize {
            let src_offset = y * rowstride + x * n_channels as usize;
            let dst_offset = (y * width as usize + x) * 4;

            rgba_buffer[dst_offset] = pixels[src_offset]; // R
            rgba_buffer[dst_offset + 1] = pixels[src_offset + 1]; // G
            rgba_buffer[dst_offset + 2] = pixels[src_offset + 2]; // B
            rgba_buffer[dst_offset + 3] = if has_alpha && n_channels == 4 {
                pixels[src_offset + 3] // A
            } else {
                255 // Opaque
            };
        }
    }

    Some((rgba_buffer, width, height))
}
```

## SUBTASK 4: Handle Icon Scaling
Properly scale icons to requested size if needed.

**Changes needed:**
```rust
fn convert_pixbuf_to_rgba(pixbuf: &gdk_pixbuf::Pixbuf) -> Option<(Vec<u8>, u32, u32)> {
    let width = pixbuf.width() as u32;
    let height = pixbuf.height() as u32;

    // Check if pixbuf is already the right size
    // If not, scale it
    let scaled_pixbuf = if width != requested_size || height != requested_size {
        match pixbuf.scale_simple(
            requested_size as i32,
            requested_size as i32,
            gdk_pixbuf::InterpType::Bilinear,
        ) {
            Some(scaled) => scaled,
            None => {
                warn!("Failed to scale icon from {}x{} to {}", width, height, requested_size);
                pixbuf.clone()
            }
        }
    } else {
        pixbuf.clone()
    };

    // Continue with conversion...
}
```

## SUBTASK 5: Handle Desktop Entry Files
Special handling for .desktop files to extract application icons.

**Add helper function:**
```rust
#[cfg(target_os = "linux")]
fn extract_desktop_entry_icon(desktop_file: &Path, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    use std::fs;

    // Read desktop file
    let content = fs::read_to_string(desktop_file).ok()?;

    // Parse Icon= line
    for line in content.lines() {
        if line.starts_with("Icon=") {
            let icon_name = line.trim_start_matches("Icon=").trim();

            // Lookup icon by name in theme
            if let Some(icon_theme) = gtk::IconTheme::default() {
                if let Some(icon_info) = icon_theme.lookup_icon(
                    icon_name,
                    size as i32,
                    gtk::IconLookupFlags::empty(),
                ) {
                    if let Ok(pixbuf) = icon_info.load_icon() {
                        return convert_pixbuf_to_rgba(&pixbuf);
                    }
                }
            }
        }
    }

    None
}
```

## SUBTASK 6: Handle Multiple Icon Themes
Support multiple icon themes with fallback chain.

**Changes needed:**
- Try user's current theme first
- Fall back to Adwaita (GNOME default)
- Fall back to Hicolor (universal fallback)
- Use generic MIME type icon if file-specific icon not found

**Example:**
```rust
fn lookup_icon_with_fallbacks(
    icon_name: &str,
    size: i32,
) -> Option<gtk::IconInfo> {
    let theme = gtk::IconTheme::default()?;

    // Try default theme
    if let Some(info) = theme.lookup_icon(icon_name, size, gtk::IconLookupFlags::empty()) {
        return Some(info);
    }

    // Try Adwaita
    theme.set_custom_theme(Some("Adwaita"));
    if let Some(info) = theme.lookup_icon(icon_name, size, gtk::IconLookupFlags::empty()) {
        return Some(info);
    }

    // Try Hicolor
    theme.set_custom_theme(Some("hicolor"));
    theme.lookup_icon(icon_name, size, gtk::IconLookupFlags::empty())
}
```

## SUBTASK 7: Error Handling and Logging
Add comprehensive error handling for all GTK/GIO API calls.

**Changes needed:**
- Log warnings when icon extraction fails
- Handle GTK initialization failures
- Handle missing icon themes gracefully
- Handle file access errors
- Document common failure cases

## DEFINITION OF DONE
- [ ] Linux dependencies added to Cargo.toml
- [ ] GIO file icon extraction implemented
- [ ] GdkPixbuf to RGBA conversion functional
- [ ] Icon scaling support working
- [ ] Desktop entry file (.desktop) icons supported
- [ ] Multiple icon theme fallbacks implemented
- [ ] Error handling and logging comprehensive
- [ ] Code compiles on Linux without warnings
- [ ] Icons display in Linux UI

## CONSTRAINTS
- **DO NOT write unit tests** - another team handles testing
- **DO NOT write benchmarks** - another team handles performance
- Focus solely on implementation in ./src
- Linux-specific code only (do not modify Windows or macOS sections)

## RESEARCH NOTES
- GIO: GNOME I/O library, provides file icon information
- GTK IconTheme: System for finding and loading icons
- GdkPixbuf: Image data structure in GTK
- Icon themes: Linux uses freedesktop.org icon theme specification
- Desktop entries: .desktop files follow freedesktop.org spec
- Hicolor: Universal fallback icon theme on all Linux systems

## DOCUMENTATION LOCATIONS
- GTK docs: https://gtk-rs.org/gtk3-rs/stable/latest/docs/gtk/
- GIO docs: https://gtk-rs.org/gtk-rs-core/stable/latest/docs/gio/
- Icon theme spec: https://specifications.freedesktop.org/icon-theme-spec/latest/
- Desktop entry spec: https://specifications.freedesktop.org/desktop-entry-spec/latest/
- Existing platform abstractions: `packages/ecs-ui/src/icons/`
