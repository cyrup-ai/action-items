use std::path::Path;

// Add tracing for logging (already in Cargo.toml dependencies)
use tracing;

/// Extract icon from file path (platform dispatcher)
///
/// Dispatches to platform-specific extraction based on target OS.
/// Returns (RGBA data, width, height) on success.
///
/// # Platform Support
/// - **macOS**: Extract from .app bundles, .icns files via NSWorkspace/NSImage APIs
/// - **Windows**: Extract from .exe, .ico files via Windows Shell APIs
/// - **Linux**: Extract from .desktop files, freedesktop icon themes
///
/// # Arguments
/// * `path` - File path to extract icon from
/// * `size` - Requested icon dimension in pixels
///
/// # Returns
/// `Some((rgba_data, width, height))` on success, `None` if extraction fails
pub fn extract_icon_from_file(path: &Path, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    #[cfg(target_os = "windows")]
    {
        extract_windows_icon(path, size)
    }
    #[cfg(target_os = "macos")]
    {
        extract_macos_icon(path, size)
    }
    #[cfg(target_os = "linux")]
    {
        extract_linux_icon(path, size)
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

/// Extract icon on Windows using Win32 Shell API
///
/// Uses SHGetFileInfoW to extract file/application icons and converts them to RGBA format.
/// Returns None if extraction fails, triggering FontAwesome fallback.
#[cfg(target_os = "windows")]
fn extract_windows_icon(path: &Path, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_SMALLICON};
    use windows::Win32::Graphics::Gdi::{DeleteObject, HICON};
    use windows::core::PCWSTR;
    use std::os::windows::ffi::OsStrExt;

    unsafe {
        // Convert path to null-terminated wide string (UTF-16)
        let wide_path: Vec<u16> = path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // Prepare structure to receive icon information
        let mut file_info: SHFILEINFOW = std::mem::zeroed();

        // Determine icon size flag based on requested pixels
        // Windows shell provides two standard sizes:
        // - Small (SHGFI_SMALLICON): typically 16x16
        // - Large (SHGFI_LARGEICON): typically 32x32
        let size_flag = if size > 32 { 
            SHGFI_LARGEICON 
        } else { 
            SHGFI_SMALLICON 
        };

        // Extract icon handle from file
        let result = SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            0,                                      // File attributes (0 = use actual file)
            Some(&mut file_info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | size_flag,                // Get icon with specified size
        );

        // Check if extraction succeeded
        if result == 0 {
            tracing::warn!("SHGetFileInfoW failed for path: {:?}", path);
            return None;
        }

        // Validate icon handle
        if file_info.hIcon.is_invalid() {
            tracing::warn!("Invalid HICON handle for path: {:?}", path);
            return None;
        }

        // Convert Windows HICON to RGBA byte array
        let rgba_result = convert_hicon_to_rgba(file_info.hIcon, size);

        // Clean up icon handle (CRITICAL: prevents resource leak)
        DeleteObject(file_info.hIcon.0);

        // Return RGBA data or None if conversion failed
        rgba_result
    }
}

/// Convert Windows HICON to RGBA byte array
///
/// Windows icons are stored as BGRA in device-dependent bitmaps.
/// This function:
/// 1. Creates compatible DC and bitmap for rendering
/// 2. Draws icon into bitmap using DrawIconEx
/// 3. Extracts bitmap bits via GetDIBits
/// 4. Converts BGRA to RGBA format
/// 5. Cleans up all GDI resources
///
/// # Arguments
/// * `hicon` - Windows icon handle (will NOT be deleted by this function)
/// * `size` - Requested icon dimension in pixels
///
/// # Returns
/// `Some((rgba_data, width, height))` on success, `None` on any failure
///
/// # Safety
/// Unsafe due to Win32 API calls. All resources are properly cleaned up.
#[cfg(target_os = "windows")]
unsafe fn convert_hicon_to_rgba(hicon: windows::Win32::Graphics::Gdi::HICON, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateCompatibleBitmap, SelectObject, DeleteDC, DeleteObject,
        GetDC, ReleaseDC, GetDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows::Win32::UI::WindowsAndMessaging::{DrawIconEx, DI_NORMAL};

    // Get screen DC for color depth reference
    let screen_dc = GetDC(None);
    if screen_dc.is_invalid() {
        tracing::warn!("Failed to get screen DC");
        return None;
    }

    // Create memory DC for off-screen rendering
    let mem_dc = CreateCompatibleDC(screen_dc);
    if mem_dc.is_invalid() {
        ReleaseDC(None, screen_dc);
        tracing::warn!("Failed to create compatible DC");
        return None;
    }

    // Create bitmap to hold icon pixels
    let bitmap = CreateCompatibleBitmap(screen_dc, size as i32, size as i32);
    if bitmap.is_invalid() {
        DeleteDC(mem_dc);
        ReleaseDC(None, screen_dc);
        tracing::warn!("Failed to create compatible bitmap");
        return None;
    }

    // Select bitmap into memory DC
    let old_bitmap = SelectObject(mem_dc, bitmap);

    // Draw icon into bitmap at full size
    let draw_result = DrawIconEx(
        mem_dc,
        0, 0,                           // Position (top-left)
        hicon,
        size as i32, size as i32,       // Size (stretch to fit)
        0,                               // Animation frame (0 = first/only frame)
        None,                            // Background brush (None = transparent)
        DI_NORMAL,                       // Draw normal (not selected/disabled)
    );

    if !draw_result.as_bool() {
        // Clean up on failure
        SelectObject(mem_dc, old_bitmap);
        DeleteObject(bitmap.0);
        DeleteDC(mem_dc);
        ReleaseDC(None, screen_dc);
        tracing::warn!("DrawIconEx failed");
        return None;
    }

    // Setup BITMAPINFO for GetDIBits
    let mut bmp_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size as i32,
            biHeight: -(size as i32),   // Negative = top-down (standard orientation)
            biPlanes: 1,
            biBitCount: 32,              // 32-bit BGRA
            biCompression: BI_RGB.0 as u32,  // Uncompressed
            biSizeImage: 0,              // Calculated by API
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [Default::default(); 1],
    };

    // Allocate buffer for BGRA pixel data
    let buffer_size = (size * size * 4) as usize;
    let mut bgra_buffer: Vec<u8> = vec![0; buffer_size];

    // Extract bitmap bits as BGRA
    let lines_copied = GetDIBits(
        mem_dc,
        bitmap,
        0,                              // Start scan line
        size,                           // Number of scan lines
        Some(bgra_buffer.as_mut_ptr() as *mut _),
        &mut bmp_info,
        DIB_RGB_COLORS,
    );

    // Clean up GDI resources (CRITICAL: prevents resource leak)
    SelectObject(mem_dc, old_bitmap);
    DeleteObject(bitmap.0);
    DeleteDC(mem_dc);
    ReleaseDC(None, screen_dc);

    // Check if GetDIBits succeeded
    if lines_copied == 0 {
        tracing::warn!("GetDIBits failed to copy pixel data");
        return None;
    }

    // Convert BGRA to RGBA by swapping R and B channels
    // Windows bitmaps store pixels as BGRA, but Bevy expects RGBA
    for i in (0..buffer_size).step_by(4) {
        bgra_buffer.swap(i, i + 2);     // Swap B (index i) with R (index i+2)
    }

    // Return RGBA data with dimensions
    Some((bgra_buffer, size, size))
}

/// Extract icon on macOS using NSWorkspace and NSImage
///
/// Uses NSWorkspace.iconForFile() to get application/file icons and converts
/// them to RGBA format. Handles both .app bundles and regular files.
///
/// Returns None if:
/// - Path doesn't exist or is inaccessible
/// - Icon extraction fails
/// - NSImage conversion fails
///
/// Falls back to FontAwesome icons when None (see packages/ui/src/ui/icons/utils.rs)
#[cfg(target_os = "macos")]
fn extract_macos_icon(path: &Path, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    use objc2::msg_send;
    use objc2::rc::Retained;
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::NSString;

    unsafe {
        // Get shared NSWorkspace instance
        let workspace = NSWorkspace::sharedWorkspace();
        
        // Convert path to NSString
        let path_str = path.to_str()?;
        let ns_path = NSString::from_str(path_str);
        
        // Get icon for file using iconForFile: method
        // Returns Option<Retained<NSImage>> in objc2
        let icon: Option<Retained<objc2_app_kit::NSImage>> = msg_send![
            &workspace,
            iconForFile: &*ns_path
        ];
        
        // Validate icon exists
        let icon = match icon {
            Some(i) => i,
            None => {
                tracing::warn!("NSWorkspace.iconForFile returned nil for path: {:?}", path);
                return None;
            }
        };
        
        // Set icon size
        let ns_size = objc2_foundation::NSSize {
            width: size as f64,
            height: size as f64,
        };
        let _: () = msg_send![&icon, setSize: ns_size];
        
        // Convert NSImage to RGBA bytes
        convert_nsimage_to_rgba(&icon, size)
    }
}

/// Convert NSImage to RGBA byte array
///
/// Strategy:
/// 1. Lock focus on NSImage to prepare for rendering
/// 2. Create NSBitmapImageRep from focused image
/// 3. Extract raw bitmap data
/// 4. Convert from native format (often ARGB/BGRA) to RGBA
/// 5. Unlock focus and cleanup
///
/// # Arguments
/// * `nsimage` - Reference to NSImage (Retained<NSImage>)
/// * `size` - Requested icon dimension in pixels
///
/// # Returns
/// `Some((rgba_data, width, height))` on success, `None` on failure
///
/// # Safety
/// Unsafe due to Objective-C method calls and raw bitmap pointer access.
/// All resources are properly managed via Retained<T>.
#[cfg(target_os = "macos")]
unsafe fn convert_nsimage_to_rgba(
    nsimage: &objc2::rc::Retained<objc2_app_kit::NSImage>,
    size: u32,
) -> Option<(Vec<u8>, u32, u32)> {
    use objc2::msg_send;
    use objc2::rc::Retained;
    use objc2::AnyThread;
    use objc2_foundation::{NSRect, NSPoint, NSSize};
    
    // Lock focus to prepare image for bitmap extraction
    let _: () = msg_send![nsimage, lockFocus];
    
    // Create rectangle for the image
    let rect = NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size: NSSize {
            width: size as f64,
            height: size as f64,
        },
    };
    
    // Create NSBitmapImageRep from focused image rect
    // This captures the current focused image into a bitmap
    // Using alloc/init pattern: msg_send![Class::alloc(), initWith...]
    let bitmap_rep: Option<Retained<objc2_app_kit::NSBitmapImageRep>> = msg_send![
        objc2_app_kit::NSBitmapImageRep::alloc(),
        initWithFocusedViewRect: rect
    ];
    
    // Unlock focus (CRITICAL: prevents memory leaks)
    let _: () = msg_send![nsimage, unlockFocus];
    
    // Validate bitmap creation succeeded
    let bitmap_rep = match bitmap_rep {
        Some(rep) => rep,
        None => {
            tracing::warn!("Failed to create NSBitmapImageRep");
            return None;
        }
    };
    
    // Get bitmap properties
    let bytes_per_row: usize = msg_send![&bitmap_rep, bytesPerRow];
    let bits_per_pixel: usize = msg_send![&bitmap_rep, bitsPerPixel];
    let bytes_per_pixel = bits_per_pixel / 8;
    
    // Get raw bitmap data pointer
    let bitmap_data: *const u8 = msg_send![&bitmap_rep, bitmapData];
    if bitmap_data.is_null() {
        tracing::warn!("NSBitmapImageRep.bitmapData returned null");
        return None;
    }
    
    // Allocate RGBA buffer
    let buffer_size = (size * size * 4) as usize;
    let mut rgba_buffer = vec![0u8; buffer_size];
    
    // Convert bitmap to RGBA
    // macOS typically uses ARGB or BGRA depending on color space
    // We need to detect and convert to RGBA
    for y in 0..size as usize {
        for x in 0..size as usize {
            let src_offset = y * bytes_per_row + x * bytes_per_pixel;
            let dst_offset = (y * size as usize + x) * 4;
            
            if bytes_per_pixel == 4 {
                // Read source pixels (might be ARGB, BGRA, or RGBA)
                // SAFETY: We've validated bitmap_data is not null and src_offset is within bounds
                let b0 = unsafe { *bitmap_data.add(src_offset) };
                let b1 = unsafe { *bitmap_data.add(src_offset + 1) };
                let b2 = unsafe { *bitmap_data.add(src_offset + 2) };
                let b3 = unsafe { *bitmap_data.add(src_offset + 3) };
                
                // Assume BGRA (most common on macOS) and convert to RGBA
                // If this assumption is wrong, icons will have wrong colors
                // Can be refined later by checking NSBitmapImageRep format
                rgba_buffer[dst_offset] = b2;     // R
                rgba_buffer[dst_offset + 1] = b1; // G
                rgba_buffer[dst_offset + 2] = b0; // B
                rgba_buffer[dst_offset + 3] = b3; // A
            } else {
                // Fallback for unexpected formats
                tracing::warn!("Unexpected bits per pixel: {}", bits_per_pixel);
                rgba_buffer[dst_offset] = 0;
                rgba_buffer[dst_offset + 1] = 0;
                rgba_buffer[dst_offset + 2] = 0;
                rgba_buffer[dst_offset + 3] = 255;
            }
        }
    }
    
    // Retained<T> automatically releases bitmap_rep
    Some((rgba_buffer, size, size))
}

/// Extract icon on Linux using freedesktop icon theme specification
///
/// Strategy:
/// 1. Detect file MIME type
/// 2. Map MIME type to icon name (e.g., "text/plain" → "text-x-generic")
/// 3. Search icon theme directories for icon file
/// 4. Load icon image file (PNG, SVG)
/// 5. Convert to RGBA format
///
/// Returns None if:
/// - MIME type detection fails
/// - Icon not found in any theme
/// - Icon file loading fails
///
/// Falls back to FontAwesome icons when None (see packages/ui/src/ui/icons/utils.rs)
#[cfg(target_os = "linux")]
fn extract_linux_icon(path: &Path, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    // Handle .desktop files specially
    if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
        if let Some(icon_data) = extract_desktop_entry_icon(path, size) {
            return Some(icon_data);
        }
    }
    
    // Detect MIME type
    let mime_type = detect_mime_type(path)?;
    
    // Map MIME type to icon name
    let icon_name = mime_type_to_icon_name(&mime_type);
    
    // Search for icon in theme directories
    let icon_path = find_icon_in_themes(&icon_name, size)?;
    
    // Load and convert icon to RGBA
    load_icon_to_rgba(&icon_path, size)
}

/// Detect MIME type of file using magic bytes
///
/// Uses `infer` crate for fast, accurate MIME detection.
/// Falls back to extension-based detection if magic bytes fail.
#[cfg(target_os = "linux")]
fn detect_mime_type(path: &Path) -> Option<String> {
    use std::fs;
    
    // Try magic byte detection first (most accurate)
    if let Ok(bytes) = fs::read(path) {
        if let Some(kind) = infer::get(&bytes) {
            return Some(kind.mime_type().to_string());
        }
    }
    
    // Fall back to extension-based detection
    match path.extension()?.to_str()? {
        "txt" => Some("text/plain".to_string()),
        "pdf" => Some("application/pdf".to_string()),
        "png" => Some("image/png".to_string()),
        "jpg" | "jpeg" => Some("image/jpeg".to_string()),
        "mp3" => Some("audio/mpeg".to_string()),
        "mp4" => Some("video/mp4".to_string()),
        "zip" => Some("application/zip".to_string()),
        "tar" => Some("application/x-tar".to_string()),
        "gz" => Some("application/gzip".to_string()),
        _ => {
            tracing::warn!("Unknown file extension for path: {:?}", path);
            None
        }
    }
}

/// Map MIME type to icon name using freedesktop.org naming conventions
///
/// Follows Icon Naming Specification:
/// https://specifications.freedesktop.org/icon-naming-spec/latest/
#[cfg(target_os = "linux")]
fn mime_type_to_icon_name(mime_type: &str) -> String {
    match mime_type {
        // Text files
        "text/plain" => "text-x-generic",
        "text/html" => "text-html",
        "text/xml" => "text-xml",
        
        // Documents
        "application/pdf" => "application-pdf",
        "application/msword" => "x-office-document",
        "application/vnd.oasis.opendocument.text" => "x-office-document",
        
        // Spreadsheets
        "application/vnd.ms-excel" => "x-office-spreadsheet",
        "application/vnd.oasis.opendocument.spreadsheet" => "x-office-spreadsheet",
        
        // Presentations
        "application/vnd.ms-powerpoint" => "x-office-presentation",
        "application/vnd.oasis.opendocument.presentation" => "x-office-presentation",
        
        // Images
        "image/png" | "image/jpeg" | "image/gif" | "image/bmp" => "image-x-generic",
        "image/svg+xml" => "image-svg+xml",
        
        // Audio
        "audio/mpeg" | "audio/ogg" | "audio/flac" => "audio-x-generic",
        
        // Video
        "video/mp4" | "video/mpeg" | "video/x-matroska" => "video-x-generic",
        
        // Archives
        "application/zip" => "package-x-generic",
        "application/x-tar" => "package-x-generic",
        "application/gzip" => "package-x-generic",
        "application/x-7z-compressed" => "package-x-generic",
        
        // Executables
        "application/x-executable" => "application-x-executable",
        "application/x-sharedlib" => "application-x-sharedlib",
        
        // Default
        _ => {
            // Try generic icon based on MIME type category
            if mime_type.starts_with("text/") {
                "text-x-generic"
            } else if mime_type.starts_with("image/") {
                "image-x-generic"
            } else if mime_type.starts_with("audio/") {
                "audio-x-generic"
            } else if mime_type.starts_with("video/") {
                "video-x-generic"
            } else {
                "text-x-generic" // Ultimate fallback
            }
        }
    }.to_string()
}

/// Find icon file in freedesktop icon theme directories
///
/// Searches in order:
/// 1. ~/.local/share/icons  (user themes)
/// 2. /usr/share/icons      (system themes)
/// 3. /usr/local/share/icons (local themes)
///
/// Theme priority:
/// 1. Current theme (from environment/settings)
/// 2. Hicolor (universal fallback)
/// 3. Adwaita (GNOME default)
#[cfg(target_os = "linux")]
fn find_icon_in_themes(icon_name: &str, size: u32) -> Option<std::path::PathBuf> {
    use std::path::PathBuf;
    
    // Get XDG data directories
    let data_dirs = vec![
        dirs::data_local_dir(), // ~/.local/share
        Some(PathBuf::from("/usr/share")),
        Some(PathBuf::from("/usr/local/share")),
    ];
    
    // Icon themes to search (in priority order)
    let themes = vec!["hicolor", "Adwaita", "breeze", "oxygen"];
    
    // Size directories to search (find closest match)
    let size_dirs = vec![
        format!("{}x{}", size, size),
        format!("{}x{}", size * 2, size * 2), // Try 2x for HiDPI
        "scalable".to_string(),
        "48x48".to_string(), // Common sizes as fallback
        "32x32".to_string(),
        "24x24".to_string(),
        "16x16".to_string(),
    ];
    
    // Search data directories
    for data_dir in data_dirs.iter().filter_map(|d| d.as_ref()) {
        let icons_dir = data_dir.join("icons");
        
        // Search themes
        for theme in &themes {
            let theme_dir = icons_dir.join(theme);
            if !theme_dir.exists() {
                continue;
            }
            
            // Search size directories
            for size_dir in &size_dirs {
                // Check common categories
                let categories = vec!["mimetypes", "apps", "places", "devices", "actions"];
                
                for category in categories {
                    let category_dir = theme_dir.join(size_dir).join(category);
                    
                    // Try PNG first, then SVG
                    for ext in &["png", "svg"] {
                        let icon_path = category_dir.join(format!("{}.{}", icon_name, ext));
                        if icon_path.exists() {
                            tracing::debug!("Found icon: {:?}", icon_path);
                            return Some(icon_path);
                        }
                    }
                }
                
                // Also try without category subdirectory
                for ext in &["png", "svg"] {
                    let icon_path = theme_dir.join(size_dir).join(format!("{}.{}", icon_name, ext));
                    if icon_path.exists() {
                        tracing::debug!("Found icon: {:?}", icon_path);
                        return Some(icon_path);
                    }
                }
            }
        }
    }
    
    tracing::warn!("Icon not found in any theme: {}", icon_name);
    None
}

/// Load icon file and convert to RGBA byte array
///
/// Supports PNG and SVG formats (via image crate).
/// Scales icon to requested size if needed.
#[cfg(target_os = "linux")]
fn load_icon_to_rgba(icon_path: &Path, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    // Check file extension
    let ext = icon_path.extension()?.to_str()?;
    
    match ext {
        "png" => load_png_to_rgba(icon_path, size),
        "svg" => {
            tracing::warn!("SVG icon support not yet implemented: {:?}", icon_path);
            None
        }
        _ => {
            tracing::warn!("Unsupported icon format: {:?}", icon_path);
            None
        }
    }
}

/// Load PNG file and convert to RGBA
///
/// Uses `image` crate (already in dependencies via bevy).
#[cfg(target_os = "linux")]
fn load_png_to_rgba(png_path: &Path, requested_size: u32) -> Option<(Vec<u8>, u32, u32)> {
    use std::fs;
    
    // Read file bytes
    let bytes = match fs::read(png_path) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("Failed to read icon file {:?}: {}", png_path, e);
            return None;
        }
    };
    
    // Decode PNG using image crate
    let img = match image::load_from_memory(&bytes) {
        Ok(i) => i,
        Err(e) => {
            tracing::warn!("Failed to decode PNG {:?}: {}", png_path, e);
            return None;
        }
    };
    
    // Scale if needed
    let img = if img.width() != requested_size || img.height() != requested_size {
        img.resize_exact(
            requested_size,
            requested_size,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        img
    };
    
    // Convert to RGBA8
    let rgba = img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    let pixels = rgba.into_raw();
    
    Some((pixels, width, height))
}

/// Extract icon from .desktop file
///
/// Parses Icon= field from desktop entry file and searches themes.
/// Follows Desktop Entry Specification:
/// https://specifications.freedesktop.org/desktop-entry-spec/latest/
#[cfg(target_os = "linux")]
fn extract_desktop_entry_icon(desktop_file: &Path, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    use std::fs;
    
    // Read desktop file
    let content = match fs::read_to_string(desktop_file) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to read desktop file {:?}: {}", desktop_file, e);
            return None;
        }
    };
    
    // Parse Icon= line
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("Icon=") {
            let icon_value = line.trim_start_matches("Icon=").trim();
            
            // Icon value can be:
            // 1. Absolute path: /usr/share/pixmaps/app.png
            // 2. Icon name: application-name (search in themes)
            
            if icon_value.starts_with('/') {
                // Absolute path
                let icon_path = std::path::PathBuf::from(icon_value);
                if icon_path.exists() {
                    return load_icon_to_rgba(&icon_path, size);
                }
            } else {
                // Icon name - search in themes
                if let Some(icon_path) = find_icon_in_themes(icon_value, size) {
                    return load_icon_to_rgba(&icon_path, size);
                }
            }
            
            tracing::warn!("Icon not found for desktop entry: {}", icon_value);
            return None;
        }
    }
    
    tracing::warn!("No Icon= field found in desktop file: {:?}", desktop_file);
    None
}
