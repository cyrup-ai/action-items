//! String interning and performance optimization
//!
//! This module provides string interning capabilities to reduce memory allocations
//! and improve performance during discovery operations.

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;

use super::types::*;

/// String interning for common values to reduce allocations
pub struct StringInterner {
    common_authors: HashMap<&'static str, &'static str>,
    common_categories: HashMap<&'static str, &'static str>,
    common_modes: HashMap<&'static str, &'static str>,
    common_types: HashMap<&'static str, &'static str>,
}

impl StringInterner {
    pub fn new() -> Self {
        let mut common_authors = HashMap::with_capacity(16);
        common_authors.insert("raycast", "Raycast");
        common_authors.insert("thomas", "Thomas");
        common_authors.insert("peduarte", "Pedro Duarte");
        common_authors.insert("mattisssa", "Mattias");
        common_authors.insert("tonka3000", "Tonka3000");
        common_authors.insert("extensions", "Extensions");
        common_authors.insert("community", "Community");
        common_authors.insert("official", "Official");

        let mut common_categories = HashMap::with_capacity(16);
        common_categories.insert("productivity", "Productivity");
        common_categories.insert("developer tools", "Developer Tools");
        common_categories.insert("system", "System");
        common_categories.insert("web search", "Web Search");
        common_categories.insert("communication", "Communication");
        common_categories.insert("media", "Media");
        common_categories.insert("finance", "Finance");
        common_categories.insert("fun", "Fun");

        let mut common_modes = HashMap::with_capacity(8);
        common_modes.insert("view", "view");
        common_modes.insert("no-view", "no-view");
        common_modes.insert("silent", "silent");

        let mut common_types = HashMap::with_capacity(16);
        common_types.insert("text", "text");
        common_types.insert("textfield", "textfield");
        common_types.insert("password", "password");
        common_types.insert("checkbox", "checkbox");
        common_types.insert("dropdown", "dropdown");
        common_types.insert("file", "file");
        common_types.insert("directory", "directory");

        Self {
            common_authors,
            common_categories,
            common_modes,
            common_types,
        }
    }

    #[inline]
    pub fn intern_author<'a>(&self, author: &'a str) -> Cow<'a, str> {
        match self.common_authors.get(author.to_lowercase().as_str()) {
            Some(&interned) => Cow::Borrowed(interned),
            None => Cow::Borrowed(author),
        }
    }

    #[inline]
    pub fn intern_category<'a>(&self, category: &'a str) -> Cow<'a, str> {
        match self.common_categories.get(category.to_lowercase().as_str()) {
            Some(&interned) => Cow::Borrowed(interned),
            None => Cow::Borrowed(category),
        }
    }

    #[inline]
    pub fn intern_mode<'a>(&self, mode: &'a str) -> Cow<'a, str> {
        match self.common_modes.get(mode) {
            Some(&interned) => Cow::Borrowed(interned),
            None => Cow::Borrowed(mode),
        }
    }

    #[inline]
    pub fn intern_type<'a>(&self, type_str: &'a str) -> Cow<'a, str> {
        match self.common_types.get(type_str) {
            Some(&interned) => Cow::Borrowed(interned),
            None => Cow::Borrowed(type_str),
        }
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Cached extension data for performance optimization
#[derive(Debug, Clone)]
pub struct CachedExtension {
    pub extension: IsolatedRaycastExtension,
    pub last_modified: std::time::SystemTime,
    pub file_size: u64,
}

/// Discovery indexer with caching and string interning
pub struct DiscoveryIndexer {
    pub interner: StringInterner,
    pub cache: HashMap<PathBuf, CachedExtension>,
}

impl DiscoveryIndexer {
    /// Create a new discovery indexer
    pub fn new() -> Self {
        Self {
            interner: StringInterner::new(),
            cache: HashMap::new(),
        }
    }

    /// Apply string interning optimizations to extension
    pub fn intern_extension(&self, extension: &mut IsolatedRaycastExtension) {
        // Intern author
        extension.author = self.interner.intern_author(&extension.author).into_owned();

        // Intern categories
        for category in &mut extension.categories {
            *category = self.interner.intern_category(category).into_owned();
        }

        // Intern command modes and types
        for command in &mut extension.commands {
            command.mode = self.interner.intern_mode(&command.mode).into_owned();

            // Intern argument types
            for arg in &mut command.arguments {
                arg.argument_type = self.interner.intern_type(&arg.argument_type).into_owned();
            }

            // Intern preference types
            for pref in &mut command.preferences {
                pref.preference_type = self
                    .interner
                    .intern_type(&pref.preference_type)
                    .into_owned();
            }
        }

        // Intern extension preference types
        for pref in &mut extension.preferences {
            pref.preference_type = self
                .interner
                .intern_type(&pref.preference_type)
                .into_owned();
        }
    }

    /// Check if extension is cached and up-to-date
    pub fn is_cached(
        &self,
        path: &PathBuf,
        last_modified: std::time::SystemTime,
        file_size: u64,
    ) -> bool {
        if let Some(cached) = self.cache.get(path) {
            cached.last_modified == last_modified && cached.file_size == file_size
        } else {
            false
        }
    }

    /// Get cached extension if available
    pub fn get_cached(&self, path: &PathBuf) -> Option<&CachedExtension> {
        self.cache.get(path)
    }

    /// Cache extension data
    pub fn cache_extension(
        &mut self,
        path: PathBuf,
        extension: IsolatedRaycastExtension,
        last_modified: std::time::SystemTime,
        file_size: u64,
    ) {
        let cached = CachedExtension {
            extension,
            last_modified,
            file_size,
        };
        self.cache.insert(path, cached);
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.cache.capacity())
    }
}

impl Default for DiscoveryIndexer {
    fn default() -> Self {
        Self::new()
    }
}
