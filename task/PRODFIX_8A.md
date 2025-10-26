# PRODFIX_8A: Implement Windows Icon Extraction

## OBJECTIVE
Implement platform-specific icon extraction for Windows using Win32 API to enable file/application icons in the UI.

## PRIORITY
**P2 - MEDIUM (UI Degradation)**

## LOCATION
`packages/ecs-ui/src/icons/extraction/platform.rs`

## CURRENT STATE
Line 47 (Windows section) returns None, causing all Windows applications and files to display without icons.

## SUBTASK 1: Add Windows Dependencies
Add required Windows API crates to enable icon extraction.

**Changes needed in** `packages/ecs-ui/Cargo.toml`:
```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.52", features = [
    "Win32_UI_Shell",
    "Win32_Graphics_Gdi",
    "Win32_Foundation",
    "Win32_System_Com",
] }
```

## SUBTASK 2: Implement SHGetFileInfo Icon Extraction
Use Windows Shell API to extract icon from file/application path.

**Changes needed in** `packages/ecs-ui/src/icons/extraction/platform.rs` **around line 47:**
```rust
#[cfg(target_os = "windows")]
fn extract_platform_icon(path: &Path, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_SMALLICON};
    use windows::Win32::Foundation::MAX_PATH;
    use std::os::windows::ffi::OsStrExt;

    unsafe {
        // Convert path to wide string
        let wide_path: Vec<u16> = path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut file_info: SHFILEINFOW = std::mem::zeroed();

        // Determine icon size flag
        let size_flag = if size > 32 { SHGFI_LARGEICON } else { SHGFI_SMALLICON };

        // Get file icon
        let result = SHGetFileInfoW(
            windows::core::PCWSTR(wide_path.as_ptr()),
            0,
            Some(&mut file_info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | size_flag,
        );

        if result == 0 || file_info.hIcon.is_invalid() {
            warn!("Failed to get icon for path: {:?}", path);
            return None;
        }

        // Convert HICON to RGBA bytes
        let rgba_data = convert_hicon_to_rgba(file_info.hIcon, size)?;

        // Clean up icon handle
        windows::Win32::Graphics::Gdi::DeleteObject(file_info.hIcon.0);

        Some(rgba_data)
    }
}
```

## SUBTASK 3: Implement HICON to RGBA Conversion
Create helper function to convert Windows HICON to RGBA byte array.

**Add new function:**
```rust
#[cfg(target_os = "windows")]
unsafe fn convert_hicon_to_rgba(hicon: HICON, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateCompatibleBitmap, SelectObject, DeleteDC, DeleteObject,
        GetDC, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        GetDIBits, HBITMAP,
    };
    use windows::Win32::UI::WindowsAndMessaging::{DrawIconEx, DI_NORMAL};

    // Get screen DC
    let screen_dc = GetDC(None);
    if screen_dc.is_invalid() {
        return None;
    }

    // Create memory DC
    let mem_dc = CreateCompatibleDC(screen_dc);
    if mem_dc.is_invalid() {
        ReleaseDC(None, screen_dc);
        return None;
    }

    // Create bitmap
    let bitmap = CreateCompatibleBitmap(screen_dc, size as i32, size as i32);
    if bitmap.is_invalid() {
        DeleteDC(mem_dc);
        ReleaseDC(None, screen_dc);
        return None;
    }

    // Select bitmap into DC
    let old_bitmap = SelectObject(mem_dc, bitmap);

    // Draw icon into bitmap
    DrawIconEx(mem_dc, 0, 0, hicon, size as i32, size as i32, 0, None, DI_NORMAL);

    // Setup BITMAPINFO for GetDIBits
    let mut bmp_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size as i32,
            biHeight: -(size as i32), // Negative for top-down bitmap
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0 as u32,
            ..Default::default()
        },
        ..Default::default()
    };

    // Allocate buffer for RGBA data
    let buffer_size = (size * size * 4) as usize;
    let mut rgba_buffer: Vec<u8> = vec![0; buffer_size];

    // Get bitmap bits
    let result = GetDIBits(
        mem_dc,
        bitmap,
        0,
        size,
        Some(rgba_buffer.as_mut_ptr() as *mut _),
        &mut bmp_info,
        DIB_RGB_COLORS,
    );

    // Clean up resources
    SelectObject(mem_dc, old_bitmap);
    DeleteObject(bitmap.0);
    DeleteDC(mem_dc);
    ReleaseDC(None, screen_dc);

    if result == 0 {
        return None;
    }

    // Convert BGRA to RGBA
    for i in (0..buffer_size).step_by(4) {
        rgba_buffer.swap(i, i + 2); // Swap B and R
    }

    Some((rgba_buffer, size, size))
}
```

## SUBTASK 4: Handle Different Icon Sizes
Support both small (16x16) and large (32x32+) icon extraction.

**Changes needed:**
- Use SHGFI_SMALLICON for sizes <= 32
- Use SHGFI_LARGEICON for sizes > 32
- Consider adding SHGFI_SHELLICONSIZE for system-default sizes
- Document size behavior in function comments

## SUBTASK 5: Error Handling and Logging
Add comprehensive error handling for all Win32 API calls.

**Changes needed:**
- Log warnings when icon extraction fails
- Handle invalid paths gracefully
- Handle invalid HICON handles
- Clean up all resources even on error paths
- Document common failure cases

## DEFINITION OF DONE
- [ ] Windows dependencies added to Cargo.toml
- [ ] SHGetFileInfoW icon extraction implemented
- [ ] HICON to RGBA conversion functional
- [ ] Both small and large icon sizes supported
- [ ] Resource cleanup (DeleteObject) for all code paths
- [ ] Error handling and logging comprehensive
- [ ] Code compiles on Windows without warnings
- [ ] Icons display in Windows UI

## CONSTRAINTS
- **DO NOT write unit tests** - another team handles testing
- **DO NOT write benchmarks** - another team handles performance
- Focus solely on implementation in ./src
- Windows-specific code only (do not modify macOS or Linux sections)

## RESEARCH NOTES
- SHGetFileInfoW: Primary Windows API for file icon extraction
- HICON: Windows icon handle that must be cleaned up
- BGRA format: Windows bitmaps use BGRA, need to convert to RGBA
- Top-down bitmap: Use negative height for correct orientation
- Resource management: Critical to clean up DC, bitmap, and icon handles

## DOCUMENTATION LOCATIONS
- Windows Shell docs: https://learn.microsoft.com/en-us/windows/win32/shell/shell-entry
- SHGetFileInfoW: https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shgetfileinfow
- Icon extraction examples: Search codebase for existing Windows icon code
- Existing platform abstractions: `packages/ecs-ui/src/icons/`
