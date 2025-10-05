//! Clipboard data types and error definitions with arboard integration

use std::fmt;
use std::path::PathBuf;

use bevy::prelude::{Entity, Event};
use serde::{Deserialize, Serialize};

/// Clipboard data formats supported across platforms
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClipboardData {
    /// Plain text content
    Text(String),
    /// HTML content with optional plain text fallback
    Html {
        html: String,
        alt_text: Option<String>,
    },
    /// Image data with explicit dimensions - RGBA format (4 bytes per pixel)
    #[cfg(feature = "image-data")]
    Image {
        /// Raw RGBA pixel data (4 bytes per pixel)
        data: Vec<u8>,
        /// Image width in pixels
        width: usize,
        /// Image height in pixels
        height: usize,
    },
    /// List of file paths
    Files(Vec<PathBuf>),
}

/// Clipboard format types for requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClipboardFormat {
    Text,
    Html,
    #[cfg(feature = "image-data")]
    Image,
    Files,
}

/// Errors that can occur during clipboard operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum ClipboardError {
    #[error("Clipboard is empty")]
    Empty,
    #[error("Unsupported format: {0:?}")]
    UnsupportedFormat(ClipboardFormat),
    #[error("Platform error: {0}")]
    PlatformError(String),
    #[error("Conversion error: {0}")]
    ConversionError(String),
    #[error("Access denied")]
    AccessDenied,
    #[error("Clipboard busy")]
    Busy,
    #[error("Platform not supported")]
    UnsupportedPlatform,
    #[error("Unknown clipboard error")]
    Unknown,
}

impl fmt::Display for ClipboardFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Text => write!(f, "Text"),
            Self::Html => write!(f, "HTML"),
            #[cfg(feature = "image-data")]
            Self::Image => write!(f, "Image"),
            Self::Files => write!(f, "Files"),
        }
    }
}

/// Conversion functions between our types and arboard types
impl From<arboard::Error> for ClipboardError {
    fn from(err: arboard::Error) -> Self {
        use arboard::Error::*;
        match err {
            ContentNotAvailable => ClipboardError::Empty,
            ClipboardNotSupported => ClipboardError::UnsupportedPlatform,
            ClipboardOccupied => ClipboardError::Busy,
            ConversionFailure => {
                ClipboardError::ConversionError("arboard conversion failure".to_string())
            },
            Unknown { description } => ClipboardError::PlatformError(description),
            // Handle future arboard::Error variants (enum is non_exhaustive)
            _ => ClipboardError::Unknown,
        }
    }
}

#[cfg(feature = "image-data")]
impl TryFrom<ClipboardData> for arboard::ImageData<'static> {
    type Error = ClipboardError;

    fn try_from(data: ClipboardData) -> Result<Self, Self::Error> {
        match data {
            ClipboardData::Image {
                data: bytes,
                width,
                height,
            } => {
                // Validate dimensions against data length
                let expected_len = width * height * 4; // RGBA format = 4 bytes per pixel
                if bytes.len() != expected_len {
                    return Err(ClipboardError::ConversionError(format!(
                        "Image data length mismatch: expected {} bytes for {}x{} RGBA, got {} \
                         bytes",
                        expected_len,
                        width,
                        height,
                        bytes.len()
                    )));
                }

                // Validate dimensions are non-zero
                if width == 0 || height == 0 {
                    return Err(ClipboardError::ConversionError(
                        "Image dimensions cannot be zero".to_string(),
                    ));
                }

                Ok(arboard::ImageData {
                    width,
                    height,
                    bytes: bytes.into(),
                })
            },
            _ => Err(ClipboardError::ConversionError(
                "Cannot convert to ImageData".to_string(),
            )),
        }
    }
}

#[cfg(feature = "image-data")]
impl From<arboard::ImageData<'_>> for ClipboardData {
    fn from(image_data: arboard::ImageData<'_>) -> Self {
        ClipboardData::Image {
            data: image_data.bytes.into_owned(),
            width: image_data.width,
            height: image_data.height,
        }
    }
}

/// Event fired when clipboard content changes - enables plugin ecosystems
#[derive(Event, Debug)]
pub struct ClipboardChangeEvent {
    pub sequence: u64,
    pub content: Option<ClipboardData>,
    pub timestamp: std::time::Instant,
    pub watcher: Entity,
}
