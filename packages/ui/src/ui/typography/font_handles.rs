use bevy::prelude::*;

/// Font handles for different font families
#[derive(Debug, Clone)]
pub struct FontHandles {
    pub ubuntu_regular: Handle<Font>,
    pub ubuntu_medium: Handle<Font>,
    pub ubuntu_bold: Handle<Font>,
    pub fira_code_regular: Handle<Font>,
    pub fontawesome_solid: Handle<Font>,
}