# PRODFIX_2: HTML/Markdown Sanitization - XSS Prevention

## OBJECTIVE
Replace dangerous string-replacement sanitization with production-grade HTML/Markdown sanitization using proper parsing libraries to prevent XSS vulnerabilities in notification content.

**Critical Security Issue**: The current implementation uses naive string replacements that:
- Break legitimate content (e.g., "button" becomes "butt" after removing "on")
- Miss sophisticated XSS vectors (e.g., `<img src=x onerror=alert('XSS')>`)
- Use blacklist approach (fundamentally insecure) instead of whitelist approach (secure)

## PRIORITY
**P0 - CRITICAL XSS VULNERABILITY**

## LOCATION
`packages/ecs-notifications/src/components/content.rs`

## CURRENT VULNERABLE CODE

### Lines 976-984: Fake HTML Sanitization
```rust
fn sanitize_html(html: &str) -> NotificationResult<String> {
    // Basic HTML sanitization - in production, use a proper HTML sanitizer
    let cleaned = html
        .replace("<script", "&lt;script")
        .replace("javascript:", "")
        .replace("on", ""); // Remove all "on*" event handlers - BREAKS LEGITIMATE CONTENT!

    Ok(cleaned)
}
```
**Call Site**: Line 150 in `sanitize_content()` method (called during content validation)

### Lines 986-989: Unsafe String Sanitization
```rust
fn sanitize_string(input: &str) -> String {
    // Basic string sanitization
    input.replace(['<', '>', '"', '\''], "")
}
```
**Call Site**: Line 155 in `sanitize_content()` for custom_data values

### Lines 991-1002: Fake Markdown to Plain Conversion
```rust
fn convert_markdown_to_plain(markdown: &str) -> String {
    // Basic markdown to plain text conversion - UNSAFE
    markdown
        .replace("**", "")
        .replace("*", "")
        .replace("#", "")
        .replace("[", "")
        .replace("]", "")
        .replace("(", "")
        .replace(")", "")
}
```
**Call Site**: Line 214 in `RichText::to_plain_text()` method

### Lines 1004-1008: Fake Markdown to HTML Conversion  
```rust
fn convert_markdown_to_html(markdown: &str) -> String {
    // Basic markdown to HTML conversion - UNSAFE
    markdown.replace("**", "<strong>").replace("*", "<em>")
}
```
**Call Site**: Line 228 in `RichText::to_html()` method

### Lines 1010-1020: Fake HTML to Plain Conversion
```rust
fn convert_html_to_plain(html: &str) -> String {
    // Basic HTML to plain text conversion - UNSAFE
    html.replace("<br>", "\n")
        .replace("<p>", "")
        .replace("</p>", "\n")
        .replace("<strong>", "")
        .replace("</strong>", "")
        .replace("<em>", "")
        .replace("</em>", "")
}
```
**Call Site**: Line 215 in `RichText::to_plain_text()` method

**Note**: Line 1022 contains `html_escape()` which is safe and should be kept as-is.

---

## IMPLEMENTATION PLAN

## SUBTASK 1: Add Sanitization Dependencies

**File**: `packages/ecs-notifications/Cargo.toml`

Add the following dependencies:
```toml
[dependencies]
# ... existing dependencies ...
ammonia = "4.0"  # HTML sanitization with whitelist approach
pulldown-cmark = "0.12"  # CommonMark parser for safe Markdown processing
```

**Version Notes**:
- ammonia 4.0 is the latest stable version with security patches
- pulldown-cmark 0.12 is the latest stable version
- Both are actively maintained and security-audited

**Research Sources**:
- [Ammonia Library Reference](../../tmp/ammonia/src/lib.rs) - Cloned for API reference
- [Pulldown-cmark Library Reference](../../tmp/pulldown-cmark/pulldown-cmark/src/lib.rs) - Cloned for API reference
- [Ammonia Examples](../../tmp/ammonia/examples/)
- [Pulldown-cmark Examples](../../tmp/pulldown-cmark/pulldown-cmark/examples/)

---

## SUBTASK 2: Add NotificationError Variant

**File**: `packages/ecs-notifications/src/components/mod.rs`

**Location**: Add to NotificationError enum at line 422

The current NotificationError enum does NOT have a variant for sanitization failures. Add:

```rust
pub enum NotificationError {
    // ... existing variants ...
    
    /// Sanitization error (HTML/Markdown processing failed)
    SanitizationError {
        content_type: String,  // "html", "markdown", etc.
        message: String,
    },
}
```

Also update the `Display` implementation (around line 455) to handle the new variant:

```rust
impl std::fmt::Display for NotificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // ... existing matches ...
            
            NotificationError::SanitizationError { content_type, message } => {
                write!(f, "Sanitization error for {}: {}", content_type, message)
            },
        }
    }
}
```

---

## SUBTASK 3: Implement HTML Sanitizer

**File**: `packages/ecs-notifications/src/components/content.rs`

**Replace**: Lines 976-984 (sanitize_html function)

**New Implementation**:
```rust
fn sanitize_html(html: &str) -> NotificationResult<String> {
    use ammonia::Builder;

    // Note: ammonia::Builder.clean() returns a String directly (not Result)
    // It never fails - malicious content is stripped, not errored
    let cleaned = Builder::default()
        // Allow only safe formatting tags
        .add_tags(&["p", "br", "strong", "em", "b", "i", "u", "ul", "ol", "li", "a", "span", "div"])
        // Allow href and title attributes on links
        .add_tag_attributes("a", &["href", "title"])
        // Allow class attribute on spans and divs for styling
        .add_tag_attributes("span", &["class"])
        .add_tag_attributes("div", &["class"])
        // Add rel="noopener noreferrer" to all links for security
        .link_rel(Some("noopener noreferrer"))
        // Only allow http/https URLs
        .url_schemes(&["https", "http"])
        // Clean the HTML (strips all disallowed tags, attributes, and scripts)
        .clean(html)
        .to_string();

    Ok(cleaned)
}
```

**Whitelist Security Policy**:
- **Allowed Tags**: Only safe formatting tags (p, br, strong, em, b, i, u, lists, links, spans, divs)
- **Link Security**: Automatically adds `rel="noopener noreferrer"` to prevent tab-nabbing attacks
- **URL Schemes**: Restricted to http/https only (blocks javascript:, data:, vbscript:, etc.)
- **Event Handlers**: All event handlers (onclick, onerror, etc.) automatically stripped
- **Script Tags**: Automatically removed with contents
- **Style Injection**: Inline styles automatically removed (unless explicitly allowed)

**API Reference**: See [ammonia Builder docs](../../tmp/ammonia/src/lib.rs#L357) for complete API

---

## SUBTASK 4: Implement Markdown Sanitizer

**File**: `packages/ecs-notifications/src/components/content.rs`

**Replace**: Lines 1004-1008 (convert_markdown_to_html function)

**New Implementation**:
```rust
fn convert_markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{Parser, html, Options, Event, Tag, TagEnd};

    // Enable safe CommonMark features
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    
    let parser = Parser::new_ext(markdown, options);

    // Filter events to remove dangerous content
    let safe_parser = parser.filter_map(|event| {
        match event {
            // Block ALL raw HTML from markdown
            Event::Html(_) | Event::InlineHtml(_) => None,

            // Sanitize link URLs
            Event::Start(Tag::Link { link_type, dest_url, title, id }) => {
                // Only allow http/https links
                if dest_url.starts_with("http://") || dest_url.starts_with("https://") {
                    Some(Event::Start(Tag::Link { link_type, dest_url, title, id }))
                } else if dest_url.starts_with("#") {
                    // Allow anchor links
                    Some(Event::Start(Tag::Link { link_type, dest_url, title, id }))
                } else {
                    // Block javascript:, data:, and other dangerous protocols
                    None
                }
            }

            // Block potentially dangerous image sources
            Event::Start(Tag::Image { link_type, dest_url, title, id }) => {
                // Only allow http/https images
                if dest_url.starts_with("http://") || dest_url.starts_with("https://") {
                    Some(Event::Start(Tag::Image { link_type, dest_url, title, id }))
                } else {
                    // Block data: URIs and other potentially dangerous sources
                    None
                }
            }

            // Pass through all other safe markdown elements
            _ => Some(event),
        }
    });

    // Generate HTML from filtered events
    let mut html_output = String::new();
    html::push_html(&mut html_output, safe_parser);

    // Double-sanitize: run through ammonia to catch any edge cases
    // This handles any HTML that might have been generated by the markdown parser
    sanitize_html(&html_output).unwrap_or(html_output)
}
```

**Security Approach**:
1. **Parser-Based**: Uses proper markdown parser instead of regex/string replacement
2. **Event Filtering**: Removes dangerous events (raw HTML, dangerous URLs) at parse time
3. **URL Validation**: Only allows http/https/anchor links, blocks javascript:, data:, vbscript:
4. **Double Sanitization**: Runs output through ammonia as second layer of defense
5. **Whitelist Philosophy**: Only allows known-safe markdown features

**API Reference**: See [pulldown-cmark examples](../../tmp/pulldown-cmark/pulldown-cmark/examples/event-filter.rs)

---

## SUBTASK 5: Implement Markdown to Plain Text

**File**: `packages/ecs-notifications/src/components/content.rs`

**Replace**: Lines 991-1002 (convert_markdown_to_plain function)

**New Implementation**:
```rust
fn convert_markdown_to_plain(markdown: &str) -> String {
    use pulldown_cmark::{Parser, Event, Options};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    
    let parser = Parser::new_ext(markdown, options);
    let mut plain_text = String::new();

    // Extract only text content, ignore all formatting
    for event in parser {
        match event {
            Event::Text(text) | Event::Code(text) => {
                plain_text.push_str(&text);
            }
            Event::SoftBreak => {
                plain_text.push(' ');
            }
            Event::HardBreak => {
                plain_text.push('\n');
            }
            // Ignore all other events (tags, HTML, etc.)
            _ => {}
        }
    }

    plain_text
}
```

**Benefits**:
- Properly parses markdown structure
- Extracts only actual text content
- Handles line breaks correctly
- Ignores all formatting and potential XSS vectors

---

## SUBTASK 6: Improve HTML to Plain Text (Optional Enhancement)

**File**: `packages/ecs-notifications/src/components/content.rs`

**Current**: Lines 1010-1020 (convert_html_to_plain function)

The current implementation using string replacement is acceptable for HTML-to-plain conversion since it's only used for display purposes and doesn't create security vulnerabilities. However, for consistency and correctness, consider using html5ever or similar:

**Enhanced Implementation** (optional):
```rust
fn convert_html_to_plain(html: &str) -> String {
    // For now, the existing implementation is acceptable for plain text extraction
    // since it only affects display, not security
    // Future: Consider using html5ever for proper HTML parsing
    html.replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("<p>", "")
        .replace("</p>", "\n\n")
        .replace("<strong>", "")
        .replace("</strong>", "")
        .replace("<em>", "")
        .replace("</em>", "")
        .replace("<b>", "")
        .replace("</b>", "")
        .replace("<i>", "")
        .replace("</i>", "")
        .replace("<u>", "")
        .replace("</u>", "")
        // Strip any remaining tags
        .chars()
        .fold((String::new(), false), |(mut text, in_tag), c| {
            match c {
                '<' => (text, true),
                '>' => (text, false),
                _ if !in_tag => {
                    text.push(c);
                    (text, in_tag)
                }
                _ => (text, in_tag),
            }
        })
        .0
        .trim()
        .to_string()
}
```

**Note**: This is a lower priority enhancement. The security-critical functions are sanitize_html() and convert_markdown_to_html().

---

## SUBTASK 7: Keep sanitize_string As-Is (Optional Review)

**File**: `packages/ecs-notifications/src/components/content.rs`

**Location**: Lines 986-989

The current `sanitize_string()` implementation is acceptable for sanitizing custom_data string values:

```rust
fn sanitize_string(input: &str) -> String {
    input.replace(['<', '>', '"', '\''], "")
}
```

This function is only used for simple string values in custom_data HashMap (line 155) and doesn't process HTML/Markdown, so basic character stripping is sufficient.

**Alternative** (if you want consistency with ammonia):
```rust
fn sanitize_string(input: &str) -> String {
    ammonia::clean_text(input)
}
```

This uses ammonia's `clean_text()` function which properly escapes all HTML special characters.

---

## CALL SITE ANALYSIS

### Functions to Replace and Their Usage:

1. **sanitize_html()** (Line 976)
   - Called from: `sanitize_content()` at line 150
   - Context: Validates and sanitizes HTML in RichText::Html variant
   - Must return: `NotificationResult<String>`

2. **sanitize_string()** (Line 986)
   - Called from: `sanitize_content()` at line 155
   - Context: Sanitizes custom_data HashMap values
   - Returns: `String` (not Result)

3. **convert_markdown_to_plain()** (Line 991)
   - Called from: `RichText::to_plain_text()` at line 214
   - Context: Converts markdown to plain text for platforms without markup support
   - Returns: `String`

4. **convert_markdown_to_html()** (Line 1004)
   - Called from: `RichText::to_html()` at line 228
   - Context: Converts markdown to HTML for platform display
   - Returns: `String`

5. **convert_html_to_plain()** (Line 1010)
   - Called from: `RichText::to_plain_text()` at line 215
   - Context: Converts HTML to plain text for display
   - Returns: `String`

**Important**: The function at line 1022 `html_escape()` is SAFE and should NOT be changed. It properly escapes HTML entities.

---

## ERROR HANDLING UPDATES

After implementing the new sanitizers, review error handling at call sites:

### In sanitize_content() (lines 147-159):

Current code:
```rust
fn sanitize_content(&mut self) -> NotificationResult<()> {
    if let RichText::Html(ref mut html) = self.body {
        *html = sanitize_html(html)?;  // Already handles Result properly
    }

    for (_, value) in self.custom_data.iter_mut() {
        *value = sanitize_string(value);  // No error handling needed (returns String)
    }

    Ok(())
}
```

This is already correct. The `?` operator propagates any sanitization errors up the call chain.

---

## DEFINITION OF DONE

- ✅ `ammonia = "4.0"` and `pulldown-cmark = "0.12"` added to Cargo.toml
- ✅ `NotificationError::SanitizationError` variant added to components/mod.rs
- ✅ `sanitize_html()` replaced with ammonia-based whitelist sanitizer (line 976-984)
- ✅ `convert_markdown_to_html()` replaced with pulldown-cmark parser + event filtering (line 1004-1008)
- ✅ `convert_markdown_to_plain()` replaced with pulldown-cmark text extraction (line 991-1002)
- ✅ All dangerous string replacements removed
- ✅ Code compiles without errors or warnings
- ✅ Manual verification with XSS test vectors confirms all attacks are blocked

### Manual Verification Test Vectors

After implementation, manually verify these malicious inputs are safely handled:

```html
<!-- Test 1: Script tag injection -->
<script>alert('XSS')</script>

<!-- Test 2: Image error handler -->
<img src=x onerror=alert('XSS')>

<!-- Test 3: JavaScript URL -->
<a href="javascript:alert('XSS')">click</a>

<!-- Test 4: Event handler -->
<div onclick=alert('XSS')>content</div>

<!-- Test 5: Data URI -->
<img src="data:text/html,<script>alert('XSS')</script>">

<!-- Test 6: Markdown with raw HTML -->
**Bold** <script>alert('XSS')</script> *italic*

<!-- Test 7: Link with JavaScript -->
[Click me](javascript:alert('XSS'))

<!-- Test 8: Embedded object -->
<object data="javascript:alert('XSS')"></object>
```

Expected result: All XSS vectors should be safely escaped, removed, or blocked. No JavaScript should execute.

---

## CONSTRAINTS

- **NO unit tests required** - Another team handles testing
- **NO benchmarks required** - Another team handles performance analysis  
- **NO documentation required** - Focus solely on implementation
- **Scope**: Only modify files specified above in `packages/ecs-notifications/src/`
- **Do not** add logging, metrics, or instrumentation beyond what exists
- **Do not** refactor code beyond what's needed for security fixes

---

## RESEARCH CITATIONS

### Cloned Library References (in ./tmp)
- [Ammonia Source Code](../../tmp/ammonia/src/lib.rs) - HTML sanitization library
- [Pulldown-cmark Source Code](../../tmp/pulldown-cmark/pulldown-cmark/src/lib.rs) - Markdown parser
- [Ammonia Builder Examples](../../tmp/ammonia/examples/)
- [Pulldown-cmark Filter Example](../../tmp/pulldown-cmark/pulldown-cmark/examples/event-filter.rs)

### Online Documentation
- Ammonia docs: https://docs.rs/ammonia/4.0.0/ammonia/
- Pulldown-cmark docs: https://docs.rs/pulldown-cmark/0.12.0/pulldown_cmark/
- OWASP XSS Prevention: https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html

### Security Principles Applied
- **Whitelist over Blacklist**: Only allow known-safe tags/attributes
- **Defense in Depth**: Multiple layers of sanitization
- **Parser-Based Validation**: Use proper parsers, not regex/string replacement
- **Minimal Attack Surface**: Restrict allowed features to minimum necessary
- **Fail-Safe Defaults**: If uncertain, block it

---

## IMPLEMENTATION NOTES

1. **Start with Dependencies**: Add ammonia and pulldown-cmark to Cargo.toml first
2. **Add Error Variant**: Update NotificationError enum before implementing sanitizers
3. **Test Compilation**: After each function replacement, verify code compiles
4. **Order of Implementation**: 
   - NotificationError variant (components/mod.rs)
   - sanitize_html() (highest priority - directly prevents XSS)
   - convert_markdown_to_html() (second priority - markdown->HTML is XSS vector)
   - convert_markdown_to_plain() (lower priority - display only)
   - convert_html_to_plain() (lowest priority - display only, optional)
5. **Verify Call Sites**: Ensure error handling at lines 150, 155, 214, 228, 215 remains correct

---

## XSS ATTACK VECTORS BLOCKED

After implementation, the following attack vectors will be mitigated:

✅ **Script Injection**: `<script>` tags stripped  
✅ **Event Handlers**: onclick, onerror, onload, etc. removed  
✅ **JavaScript URLs**: javascript: protocol blocked  
✅ **Data URIs**: data: protocol blocked (configurable)  
✅ **Object/Embed Tags**: Removed unless explicitly whitelisted  
✅ **CSS Expression Injection**: Inline styles removed  
✅ **Meta Refresh**: Meta tags not in whitelist  
✅ **Form Injection**: Form tags not in whitelist  
✅ **Markdown HTML Injection**: Raw HTML in markdown stripped  
✅ **Link Tab-nabbing**: rel="noopener noreferrer" added automatically

This implementation follows OWASP recommendations and industry best practices for HTML/Markdown sanitization.
