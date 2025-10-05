use bevy::prelude::*;

#[derive(Component)]
pub struct SearchInput;

#[derive(Component)]
pub struct SearchInputContainer;

#[derive(Component)]
pub struct SearchContainer;

#[derive(Component)]
pub struct HotkeyInputField;

/// Search bar state management
#[derive(Resource, Default)]
pub struct SearchBarState {
    pub query: String,
    pub is_focused: bool,
    pub cursor_position: usize,
}

impl SearchBarState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            is_focused: false,
            cursor_position: 0,
        }
    }

    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.cursor_position = self.query.len();
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor_position = 0;
    }

    pub fn insert_char(&mut self, ch: char) {
        self.query.insert(self.cursor_position, ch);
        self.cursor_position += ch.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            let mut chars: Vec<char> = self.query.chars().collect();
            if self.cursor_position <= chars.len() {
                chars.remove(self.cursor_position - 1);
                self.query = chars.into_iter().collect();
                self.cursor_position -= 1;
            }
        }
    }
}
