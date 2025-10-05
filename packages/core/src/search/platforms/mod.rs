use crate::error::Result;
use crate::search::index::SearchIndex;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub fn index_applications(search_index: &mut SearchIndex) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        windows::index_applications(search_index)
    }
    #[cfg(target_os = "macos")]
    {
        macos::index_applications(search_index)
    }
    #[cfg(target_os = "linux")]
    {
        linux::index_applications(search_index)
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        // Unsupported platform
        Ok(())
    }
}
