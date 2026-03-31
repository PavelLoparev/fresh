//! Completion provider trait and shared types.
//!
//! This module defines the core abstraction for pluggable completion sources.
//! Providers can be implemented in Rust (for performance-critical, buffer-local
//! algorithms) or in TypeScript plugins (for extensibility).

use std::fmt;

/// A single completion candidate produced by a provider.
#[derive(Debug, Clone)]
pub struct CompletionCandidate {
    /// The text to display in the completion popup.
    pub label: String,

    /// The text to insert when the completion is accepted.
    /// If `None`, `label` is used as the insert text.
    pub insert_text: Option<String>,

    /// Optional detail shown alongside the label (e.g., type info).
    pub detail: Option<String>,

    /// Icon hint for the popup (e.g., "v" for variable, "λ" for function).
    pub icon: Option<String>,

    /// Provider-assigned relevance score. Higher is better.
    /// Used by the `CompletionService` to merge and rank results from
    /// multiple providers.
    pub score: i64,

    /// Which provider produced this candidate. Set automatically by the
    /// service; providers should leave this as `None`.
    pub source: Option<CompletionSourceId>,

    /// If `true`, the insert_text contains LSP-style snippet syntax
    /// (`$0`, `${1:placeholder}`, etc.).
    pub is_snippet: bool,

    /// Opaque provider-specific data carried through to acceptance.
    /// For example, the LSP provider stores the serialised `CompletionItem`
    /// so it can request `completionItem/resolve` on accept.
    pub provider_data: Option<String>,
}

impl CompletionCandidate {
    /// Create a simple word candidate (no snippet, no extra data).
    pub fn word(label: String, score: i64) -> Self {
        Self {
            label,
            insert_text: None,
            detail: None,
            icon: None,
            score,
            source: None,
            is_snippet: false,
            provider_data: None,
        }
    }
}

/// Identifies a registered completion provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompletionSourceId(pub String);

impl fmt::Display for CompletionSourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Context passed to every provider when completion is requested.
///
/// All byte ranges are clamped to valid buffer positions by the service
/// before being handed to providers.
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// The prefix the user has already typed (from word start to cursor).
    pub prefix: String,

    /// Byte offset of the cursor in the buffer.
    pub cursor_byte: usize,

    /// Byte offset where the current word starts (for replacement range).
    pub word_start_byte: usize,

    /// Total buffer size in bytes.
    pub buffer_len: usize,

    /// Whether this buffer is lazily loaded (multi-gigabyte).
    pub is_large_file: bool,

    /// The safe scan window: providers MUST NOT read outside this range.
    /// For normal files this covers a generous region around the cursor.
    /// For huge files this is clamped to a small neighbourhood.
    pub scan_range: std::ops::Range<usize>,

    /// Byte position of the first visible line in the viewport.
    /// Useful for proximity scoring—candidates near the viewport rank higher.
    pub viewport_top_byte: usize,

    /// Approximate byte position of the last visible line.
    pub viewport_bottom_byte: usize,

    /// The file extension or language id, if known.
    pub language_id: Option<String>,
}

/// Maximum scan radius (in bytes) around the cursor for normal files.
pub const NORMAL_SCAN_RADIUS: usize = 512 * 1024; // 512 KB

/// Maximum scan radius for large/huge files—keeps completion instant.
pub const LARGE_FILE_SCAN_RADIUS: usize = 32 * 1024; // 32 KB

impl CompletionContext {
    /// Compute the scan range for a given cursor position and buffer size.
    pub fn compute_scan_range(
        cursor_byte: usize,
        buffer_len: usize,
        is_large_file: bool,
    ) -> std::ops::Range<usize> {
        let radius = if is_large_file {
            LARGE_FILE_SCAN_RADIUS
        } else {
            NORMAL_SCAN_RADIUS
        };
        let start = cursor_byte.saturating_sub(radius);
        let end = (cursor_byte + radius).min(buffer_len);
        start..end
    }
}

/// Result returned by a provider's `provide` method.
pub enum ProviderResult {
    /// Synchronous results, available immediately.
    Ready(Vec<CompletionCandidate>),
    /// The provider will deliver results asynchronously (e.g., LSP).
    /// The `u64` is a request id that will be matched later when results
    /// arrive via `CompletionService::supply_async_results`.
    Pending(u64),
}

/// Trait that all completion providers implement.
///
/// # Huge-file contract
///
/// Providers MUST honour `ctx.scan_range`. Reading outside that window on a
/// lazily-loaded buffer will either trigger expensive chunk loads or return
/// garbage bytes. The `CompletionService` enforces this constraint by
/// construction, but providers should also be defensive.
pub trait CompletionProvider: Send {
    /// Unique, stable identifier for this provider (e.g., `"lsp"`, `"dabbrev"`).
    fn id(&self) -> CompletionSourceId;

    /// Human-readable name shown in UI (e.g., "Dynamic Abbreviation").
    fn display_name(&self) -> &str;

    /// Whether this provider should be active for the given context.
    ///
    /// Returning `false` skips the provider entirely (no allocation).
    /// For example, a Rust-only provider might return `false` for markdown
    /// files, or a heavy provider might decline for huge files.
    fn is_enabled(&self, ctx: &CompletionContext) -> bool;

    /// Produce completion candidates.
    ///
    /// Implementations receive the buffer bytes they need through the
    /// `buffer_window` slice, which corresponds exactly to `ctx.scan_range`.
    /// This avoids giving providers direct `Buffer` access (which would be
    /// unsafe for the huge-file contract).
    fn provide(
        &self,
        ctx: &CompletionContext,
        buffer_window: &[u8],
    ) -> ProviderResult;

    /// Priority tier for this provider. Lower numbers run first and their
    /// results are shown higher in the list when scores are equal.
    /// Convention: 0 = LSP, 10 = ctags/index, 20 = buffer words, 30 = dabbrev.
    fn priority(&self) -> u32 {
        20
    }
}
