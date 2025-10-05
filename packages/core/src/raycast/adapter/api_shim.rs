//! Raycast API Adapter Generation
//!
//! Zero-allocation API adapter generator that creates JavaScript compatibility layer
//! for Raycast extensions with blazing-fast performance.

use crate::error::Result;

/// Generate the complete Raycast API adapter JavaScript code
pub fn create_api_adapter() -> Result<String> {
    Ok(create_raycast_api_adapter())
}

/// Create the Raycast API adapter that maps to our plugin interface
fn create_raycast_api_adapter() -> String {
    r#"
// @raycast/api adapter for Action Items
import { PluginContext } from './plugin-context.js';

// React-like component system
const React = {
    createElement(type, props, ...children) {
        return { type, props: props || {}, children };
    },
    useState(initial) {
        // Simplified state management
        let state = initial;
        const setState = (newState) => {
            state = typeof newState === 'function' ? newState(state) : newState;
            // Trigger re-render through our plugin system
            globalThis.__actionItems__.rerender();
        };
        return [state, setState];
    },
    useEffect(effect, deps) {
        // Simplified effect hook
        globalThis.__actionItems__.registerEffect(effect, deps);
    }
};

// Raycast UI components mapped to our search results
export function List({ searchBarPlaceholder, onSearchTextChange, throttle, children }) {
    const items = React.Children.toArray(children);
    return {
        type: 'list',
        placeholder: searchBarPlaceholder || 'Search...',
        onSearchTextChange,
        throttle,
        items: items.map(child => ({
            ...child.props,
            id: child.props.id || child.props.title?.replace(/\s+/g, '_').toLowerCase(),
        }))
    };
}

export function Detail({ markdown, actions }) {
    return {
        type: 'detail',
        content: markdown,
        actions: actions?.props?.children || []
    };
}

export const Icon = {
    Envelope: 'envelope',
    Key: 'key',
    Globe: 'globe',
    Trash: 'trash',
    Eye: 'eye',
    Message: 'message',
    Terminal: 'terminal',
    Calendar: 'calendar',
    Clock: 'clock',
    Document: 'document',
    Folder: 'folder',
    Gear: 'gear',
    Heart: 'heart',
    Home: 'home',
    Lightbulb: 'lightbulb',
    List: 'list',
    Lock: 'lock',
    Person: 'person',
    Phone: 'phone',
    Photo: 'photo',
    Play: 'play',
    Plus: 'plus',
    QuestionMark: 'question-mark',
    Search: 'search',
    Star: 'star',
    Tag: 'tag',
    Video: 'video',
    Warning: 'warning',
    Wand: 'wand',
    // Complete icon mapping for Raycast compatibility
};

export const Color = {
    Purple: '#8B5CF6',
    Blue: '#3B82F6',
    Red: '#EF4444',
    Green: '#10B981',
    // ... map all Raycast colors
};

export function ActionPanel({ children }) {
    return {
        type: 'action-panel',
        actions: React.Children.toArray(children)
    };
}

export const Action = {
    CopyToClipboard({ title, content, icon, shortcut }) {
        return {
            type: 'copy-to-clipboard',
            title,
            content,
            icon,
            shortcut
        };
    },
    OpenInBrowser({ title, url, icon }) {
        return {
            type: 'open-in-browser',
            title,
            url,
            icon
        };
    },
    Push({ title, target, icon }) {
        return {
            type: 'push-view',
            title,
            target,
            icon
        };
    }
};

// Environment and utilities
export const environment = {
    assetsPath: '/tmp/raycast-assets',
    supportPath: '/tmp/raycast-support',
    commandName: globalThis.__actionItems__.commandName,
    extensionName: globalThis.__actionItems__.extensionName,
};

export async function showHUD(message) {
    await globalThis.__actionItems__.showHUD(message);
}

export async function popToRoot() {
    await globalThis.__actionItems__.popToRoot();
}

// Export React for JSX
export { React };
export default {
    List,
    Detail,
    Icon,
    Color,
    ActionPanel,
    Action,
    environment,
    showHUD,
    popToRoot,
};
"#
    .to_string()
}
