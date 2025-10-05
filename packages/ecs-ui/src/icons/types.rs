/// Standard icon sizes with pixel dimensions
///
/// Provides consistent icon sizing across the application.
/// Each size maps to exact pixel dimensions for rendering.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum IconSize {
    /// 16x16 pixels - for compact UI elements
    Small,
    /// 32x32 pixels - standard size for most UI
    Medium,
    /// 64x64 pixels - for prominent display
    Large,
    /// 128x128 pixels - for detailed viewing
    XLarge,
}

impl IconSize {
    /// Get pixel dimension for this icon size
    ///
    /// Returns the square dimension (width/height are equal).
    #[inline]
    pub fn pixels(&self) -> u32 {
        match self {
            IconSize::Small => 16,
            IconSize::Medium => 32,
            IconSize::Large => 64,
            IconSize::XLarge => 128,
        }
    }
}

/// Universal icon type categories
///
/// These categories are generic across all applications - any file browser,
/// launcher, or content management system needs these same icon types.
/// This is NOT launcher-specific - these are universal content categories.
///
/// # Categories
/// - Application types: Application, Command, Terminal
/// - File system: Folder, File
/// - Code/Config: Code, Config, Database
/// - Documents: Document, Text, Spreadsheet, Presentation
/// - Media: Image, Video, Audio
/// - Archives/Fonts: Archive, Font
/// - Web/API: Web, Api
/// - System: Log, Lock
/// - Fallback: Unknown
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum IconType {
    // Application types
    Application,
    Command,
    Terminal,

    // File system types
    Folder,
    File,

    // Code and config
    Code,
    Config,
    Database,

    // Documents
    Document,
    Text,
    Spreadsheet,
    Presentation,

    // Media
    Image,
    Video,
    Audio,

    // Archives and fonts
    Archive,
    Font,

    // Web and API
    Web,
    Api,

    // System
    Log,
    Lock,

    // Fallback
    Unknown,
}
