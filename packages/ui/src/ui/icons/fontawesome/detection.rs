//! Icon type detection utilities for intelligent type inference

use crate::ui::icons::IconType;

/// Icon detection utilities for intelligent type inference
pub struct IconDetection;

impl IconDetection {
    /// Detect icon type from file extension with comprehensive mapping
    pub fn detect_from_extension(extension: &str) -> IconType {
        match extension.to_lowercase().as_str() {
            // Applications
            "app" => IconType::Application,
            "exe" | "msi" | "deb" | "rpm" | "dmg" | "pkg" => IconType::Application,

            // Code files
            "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "vue" | "svelte" => IconType::Code,
            "html" | "css" | "scss" | "sass" | "less" => IconType::Code,
            "java" | "kt" | "scala" | "clj" | "go" | "c" | "cpp" | "h" | "hpp" => IconType::Code,
            "php" | "rb" | "swift" | "dart" | "zig" | "nim" | "lua" => IconType::Code,

            // Documents
            "pdf" | "doc" | "docx" | "rtf" | "odt" => IconType::Document,
            "txt" | "md" | "rst" | "tex" => IconType::Text,
            "xls" | "xlsx" | "csv" | "ods" => IconType::Spreadsheet,
            "ppt" | "pptx" | "odp" => IconType::Presentation,

            // Media files
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" => IconType::Image,
            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" => IconType::Video,
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" => IconType::Audio,

            // Archives
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "z" => IconType::Archive,

            // Configuration
            "json" | "yaml" | "yml" | "toml" | "xml" | "ini" | "cfg" => IconType::Config,

            // Database
            "db" | "sqlite" | "sql" | "mdb" => IconType::Database,

            // Fonts
            "ttf" | "otf" | "woff" | "woff2" | "eot" => IconType::Font,

            // System
            "log" => IconType::Log,
            "lock" => IconType::Lock,

            // Default fallback
            _ => IconType::Unknown,
        }
    }

    /// Detect icon type from file path with intelligent analysis
    pub fn detect_from_path(path: &str) -> IconType {
        // Check if it's a directory
        if path.ends_with('/') || !path.contains('.') {
            return IconType::Folder;
        }

        // Extract extension
        if let Some(extension) = path.split('.').next_back() {
            Self::detect_from_extension(extension)
        } else {
            IconType::Unknown
        }
    }

    /// Detect icon type from application metadata
    pub fn detect_from_app_info(app_name: &str, bundle_id: Option<&str>) -> IconType {
        // Check bundle ID patterns (macOS)
        if let Some(id) = bundle_id {
            if id.contains("developer") || id.contains("code") || id.contains("git") {
                return IconType::Code;
            }
            if id.contains("terminal") || id.contains("iterm") {
                return IconType::Terminal;
            }
            if id.contains("browser") || id.contains("safari") || id.contains("firefox") {
                return IconType::Web;
            }
        }

        // Check application name patterns
        let app_lower = app_name.to_lowercase();
        if app_lower.contains("code") || app_lower.contains("vim") || app_lower.contains("atom") {
            IconType::Code
        } else if app_lower.contains("terminal") || app_lower.contains("iTerm") {
            IconType::Terminal
        } else if app_lower.contains("browser") || app_lower.contains("safari") {
            IconType::Web
        } else {
            IconType::Application
        }
    }

    /// Detect icon type from command metadata
    pub fn detect_from_command(command: &str, description: Option<&str>) -> IconType {
        let cmd_lower = command.to_lowercase();

        // Check command patterns
        if cmd_lower.contains("git") || cmd_lower.contains("npm") || cmd_lower.contains("cargo") {
            return IconType::Code;
        }
        if cmd_lower.contains("ssh") || cmd_lower.contains("curl") || cmd_lower.contains("wget") {
            return IconType::Web;
        }
        if cmd_lower.contains("vim") || cmd_lower.contains("nano") || cmd_lower.contains("emacs") {
            return IconType::Code;
        }

        // Check description if available
        if let Some(desc) = description {
            let desc_lower = desc.to_lowercase();
            if desc_lower.contains("editor") || desc_lower.contains("code") {
                return IconType::Code;
            }
            if desc_lower.contains("terminal") || desc_lower.contains("shell") {
                return IconType::Terminal;
            }
            if desc_lower.contains("web") || desc_lower.contains("browser") {
                return IconType::Web;
            }
        }

        IconType::Command
    }

    /// Detect icon type from action metadata
    pub fn determine_from_action(action: &str, description: &str) -> IconType {
        let action_lower = action.to_lowercase();
        let desc_lower = description.to_lowercase();

        // Check action patterns
        if action_lower.contains("open")
            && (desc_lower.contains("application") || desc_lower.contains("app"))
        {
            return IconType::Application;
        }
        if action_lower.contains("execute") || action_lower.contains("run") {
            if desc_lower.contains("code") || desc_lower.contains("editor") {
                return IconType::Code;
            }
            if desc_lower.contains("terminal") || desc_lower.contains("shell") {
                return IconType::Terminal;
            }
            return IconType::Command;
        }
        if action_lower.contains("browse")
            || action_lower.contains("web")
            || desc_lower.contains("url")
        {
            return IconType::Web;
        }
        if action_lower.contains("edit") || desc_lower.contains("file") {
            if desc_lower.contains("code") || desc_lower.contains("script") {
                return IconType::Code;
            }
            if desc_lower.contains("text") || desc_lower.contains("document") {
                return IconType::Text;
            }
            return IconType::File;
        }
        if action_lower.contains("folder") || action_lower.contains("directory") {
            return IconType::Folder;
        }

        // Check description patterns for more context
        if desc_lower.contains("application") || desc_lower.contains("app") {
            IconType::Application
        } else if desc_lower.contains("folder") || desc_lower.contains("directory") {
            IconType::Folder
        } else if desc_lower.contains("file") {
            IconType::File
        } else if desc_lower.contains("command") || desc_lower.contains("script") {
            IconType::Command
        } else {
            IconType::Unknown
        }
    }
}
