//! Simple arboard wrapper with async compatibility

use std::path::PathBuf;

use arboard::{Clipboard, ImageData};
use tracing::debug;

use crate::types::{ClipboardData, ClipboardError, ClipboardFormat};

/// Async arboard wrapper - no blocking operations
pub struct ArboardManager;

impl ArboardManager {
    /// Get clipboard text content
    pub async fn get_text() -> Result<String, ClipboardError> {
        debug!("Getting clipboard text");
        let task_pool = bevy::tasks::AsyncComputeTaskPool::get();
        task_pool
            .spawn(async move {
                let mut clipboard = Clipboard::new().map_err(ClipboardError::from)?;
                clipboard.get_text().map_err(ClipboardError::from)
            })
            .await
    }

    /// Set clipboard text content
    pub async fn set_text(text: String) -> Result<(), ClipboardError> {
        debug!("Setting clipboard text");
        let task_pool = bevy::tasks::AsyncComputeTaskPool::get();
        task_pool
            .spawn(async move {
                let mut clipboard = Clipboard::new().map_err(ClipboardError::from)?;
                clipboard.set_text(text).map_err(ClipboardError::from)
            })
            .await
    }

    /// Get clipboard HTML content
    pub async fn get_html() -> Result<String, ClipboardError> {
        debug!("Getting clipboard HTML");
        let task_pool = bevy::tasks::AsyncComputeTaskPool::get();
        task_pool
            .spawn(async move {
                let mut clipboard = Clipboard::new().map_err(ClipboardError::from)?;
                clipboard.get().html().map_err(ClipboardError::from)
            })
            .await
    }

    /// Set clipboard HTML content
    pub async fn set_html(html: String, alt_text: Option<String>) -> Result<(), ClipboardError> {
        debug!("Setting clipboard HTML");
        let task_pool = bevy::tasks::AsyncComputeTaskPool::get();
        task_pool
            .spawn(async move {
                let mut clipboard = Clipboard::new().map_err(ClipboardError::from)?;
                clipboard
                    .set_html(html, alt_text)
                    .map_err(ClipboardError::from)
            })
            .await
    }

    /// Get clipboard image content with dimensions
    #[cfg(feature = "image-data")]
    pub async fn get_image() -> Result<ClipboardData, ClipboardError> {
        debug!("Getting clipboard image");
        let task_pool = bevy::tasks::AsyncComputeTaskPool::get();
        task_pool
            .spawn(async move {
                let mut clipboard = Clipboard::new().map_err(ClipboardError::from)?;
                let image_data = clipboard.get_image().map_err(ClipboardError::from)?;
                Ok(ClipboardData::Image {
                    data: image_data.bytes.into_owned(),
                    width: image_data.width,
                    height: image_data.height,
                })
            })
            .await
    }

    /// Set clipboard image content
    #[cfg(feature = "image-data")]
    pub async fn set_image(
        image_bytes: Vec<u8>,
        width: usize,
        height: usize,
    ) -> Result<(), ClipboardError> {
        debug!("Setting clipboard image");
        let task_pool = bevy::tasks::AsyncComputeTaskPool::get();
        task_pool
            .spawn(async move {
                let mut clipboard = Clipboard::new().map_err(ClipboardError::from)?;
                let image_data = ImageData {
                    width,
                    height,
                    bytes: image_bytes.into(),
                };
                clipboard
                    .set_image(image_data)
                    .map_err(ClipboardError::from)
            })
            .await
    }

    /// Get clipboard file list
    pub async fn get_files() -> Result<Vec<PathBuf>, ClipboardError> {
        debug!("Getting clipboard files");
        let task_pool = bevy::tasks::AsyncComputeTaskPool::get();
        task_pool
            .spawn(async move {
                let mut clipboard = Clipboard::new().map_err(ClipboardError::from)?;
                clipboard.get().file_list().map_err(ClipboardError::from)
            })
            .await
    }

    /// Set clipboard file list
    pub async fn set_files(files: Vec<PathBuf>) -> Result<(), ClipboardError> {
        debug!("Setting clipboard files");
        let task_pool = bevy::tasks::AsyncComputeTaskPool::get();
        task_pool
            .spawn(async move {
                let mut clipboard = Clipboard::new().map_err(ClipboardError::from)?;
                clipboard
                    .set()
                    .file_list(&files)
                    .map_err(ClipboardError::from)
            })
            .await
    }

    /// Clear clipboard contents
    pub async fn clear() -> Result<(), ClipboardError> {
        debug!("Clearing clipboard");
        let task_pool = bevy::tasks::AsyncComputeTaskPool::get();
        task_pool
            .spawn(async move {
                let mut clipboard = Clipboard::new().map_err(ClipboardError::from)?;
                clipboard.clear().map_err(ClipboardError::from)
            })
            .await
    }

    /// Get clipboard content in specified format
    pub async fn get(format: ClipboardFormat) -> Result<ClipboardData, ClipboardError> {
        match format {
            ClipboardFormat::Text => {
                let text = Self::get_text().await?;
                Ok(ClipboardData::Text(text))
            },
            ClipboardFormat::Html => {
                let html = Self::get_html().await?;
                Ok(ClipboardData::Html {
                    html,
                    alt_text: None,
                })
            },
            #[cfg(feature = "image-data")]
            ClipboardFormat::Image => Self::get_image().await,
            #[cfg(not(feature = "image-data"))]
            ClipboardFormat::Image => {
                Err(ClipboardError::UnsupportedFormat(ClipboardFormat::Image))
            },
            ClipboardFormat::Files => {
                let files = Self::get_files().await?;
                Ok(ClipboardData::Files(files))
            },
        }
    }

    /// Set clipboard content
    pub async fn set(data: ClipboardData) -> Result<(), ClipboardError> {
        match data {
            ClipboardData::Text(text) => Self::set_text(text).await,
            ClipboardData::Html { html, alt_text } => Self::set_html(html, alt_text).await,
            #[cfg(feature = "image-data")]
            ClipboardData::Image {
                data,
                width,
                height,
            } => Self::set_image(data, width, height).await,
            #[cfg(not(feature = "image-data"))]
            ClipboardData::Image(_) => {
                Err(ClipboardError::UnsupportedFormat(ClipboardFormat::Image))
            },
            ClipboardData::Files(files) => Self::set_files(files).await,
        }
    }

    /// Check if clipboard has content in specified format efficiently
    pub async fn has_format(format: ClipboardFormat) -> bool {
        debug!("Checking clipboard format: {:?}", format);
        let task_pool = bevy::tasks::AsyncComputeTaskPool::get();
        

        task_pool
            .spawn(async move {
                let mut clipboard = match Clipboard::new() {
                    Ok(c) => c,
                    Err(_) => return false,
                };

                // Use arboard's efficient format checking
                match format {
                    ClipboardFormat::Text => clipboard.get_text().is_ok(),
                    ClipboardFormat::Html => clipboard.get().html().is_ok(),
                    #[cfg(feature = "image-data")]
                    ClipboardFormat::Image => clipboard.get_image().is_ok(),
                    #[cfg(not(feature = "image-data"))]
                    ClipboardFormat::Image => false,
                    ClipboardFormat::Files => clipboard.get().file_list().is_ok(),
                }
            })
            .await
    }

    /// Get available formats in clipboard efficiently
    pub async fn available_formats() -> Vec<ClipboardFormat> {
        debug!("Getting available clipboard formats");
        let task_pool = bevy::tasks::AsyncComputeTaskPool::get();
        

        task_pool
            .spawn(async move {
                let mut clipboard = match Clipboard::new() {
                    Ok(c) => c,
                    Err(_) => return Vec::new(),
                };

                let mut formats = Vec::new();

                // Check text format
                if clipboard.get_text().is_ok() {
                    formats.push(ClipboardFormat::Text);
                }

                // Check HTML format efficiently by trying to get HTML
                if clipboard.get().html().is_ok() {
                    formats.push(ClipboardFormat::Html);
                }

                // Check files format efficiently
                if clipboard.get().file_list().is_ok() {
                    formats.push(ClipboardFormat::Files);
                }

                // Check image format separately
                #[cfg(feature = "image-data")]
                if clipboard.get_image().is_ok() {
                    formats.push(ClipboardFormat::Image);
                }

                formats
            })
            .await
    }
}
