# Plugins Directory

This directory contains actual plugin implementations for the action-items application.

## Structure

- Each plugin should be in its own subdirectory
- Plugin implementations can be in Rust (using the plugin-native framework) or WebAssembly (using the plugin-wasm framework)
- Plugin frameworks are located in `../packages/plugin-native` and `../packages/plugin-wasm`

## Plugin Development

Refer to the plugin framework documentation in the packages directory for guidance on developing new plugins.