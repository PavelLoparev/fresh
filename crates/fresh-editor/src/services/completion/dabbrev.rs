//! Dynamic Abbreviation (dabbrev) completion provider.
//!
//! Scans buffer text near the cursor for words that share the typed prefix,
//! ordered by proximity to the cursor (nearest first). This mirrors the
//! behaviour of Emacs `dabbrev-expand` / `hippie-expand`.
//!
//! # Huge-file safety
//!
//! Only the byte window supplied by the completion service (`buffer_window`)
//! is read. For normal files this is up to 512 KB around the cursor; for
//! lazily-loaded huge files it shrinks to 32 KB.
//!
//! # Unicode
//!
//! Word boundaries are detected using Unicode grapheme clusters via the
//! `unicode-segmentation` crate, so identifiers containing accented
//! characters, CJK, or emoji are handled correctly.

use std::collections::HashSet;

use unicode_segmentation::UnicodeSegmentation;

use super::provider::{
    CompletionCandidate, CompletionContext, CompletionProvider, CompletionSourceId, ProviderResult,
};

/// Maximum number of candidates the dabbrev provider returns.
const MAX_CANDIDATES: usize = 30;

pub struct DabbrevProvider;

impl DabbrevProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DabbrevProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Check whether a grapheme cluster is a "word" grapheme.
///
/// A grapheme is considered a word constituent if it contains at least one
/// alphanumeric character or underscore. This matches the convention used by
/// the existing `word_navigation` module while being grapheme-aware.
fn is_word_grapheme(g: &str) -> bool {
    g.chars().any(|c| c.is_alphanumeric() || c == '_')
}

/// Extract all word tokens from `text`, returning `(byte_offset, word)` pairs.
///
/// A "word" is a maximal sequence of consecutive word-class grapheme clusters.
/// The byte offsets are relative to the start of `text`.
fn extract_words(text: &str) -> Vec<(usize, String)> {
    let mut words = Vec::new();
    let mut current_word = String::new();
    let mut word_start: usize = 0;
    let mut byte_pos: usize = 0;

    for grapheme in text.graphemes(true) {
        if is_word_grapheme(grapheme) {
            if current_word.is_empty() {
                word_start = byte_pos;
            }
            current_word.push_str(grapheme);
        } else if !current_word.is_empty() {
            words.push((word_start, std::mem::take(&mut current_word)));
        }
        byte_pos += grapheme.len();
    }
    // Flush trailing word
    if !current_word.is_empty() {
        words.push((word_start, current_word));
    }
    words
}

impl CompletionProvider for DabbrevProvider {
    fn id(&self) -> CompletionSourceId {
        CompletionSourceId("dabbrev".into())
    }

    fn display_name(&self) -> &str {
        "Dynamic Abbreviation"
    }

    fn is_enabled(&self, ctx: &CompletionContext) -> bool {
        // Dabbrev works for any language and any file size (scan window
        // already constrains the work). Only skip if the prefix is empty.
        !ctx.prefix.is_empty()
    }

    fn provide(
        &self,
        ctx: &CompletionContext,
        buffer_window: &[u8],
    ) -> ProviderResult {
        let text = String::from_utf8_lossy(buffer_window);
        let prefix_lower = ctx.prefix.to_lowercase();

        // Offset of the cursor *within* the buffer_window.
        let cursor_in_window = ctx.cursor_byte.saturating_sub(ctx.scan_range.start);

        let words = extract_words(&text);

        // Deduplicate while preserving the first (nearest) occurrence.
        let mut seen = HashSet::new();
        // The word currently being typed should not appear as a candidate.
        seen.insert(ctx.prefix.to_lowercase());

        // Sort by proximity to cursor (absolute byte distance within window).
        let mut scored: Vec<(usize, &str)> = words
            .iter()
            .filter(|(_, w)| {
                w.len() > ctx.prefix.len() && w.to_lowercase().starts_with(&prefix_lower)
            })
            .map(|(off, w)| {
                let dist = if *off < cursor_in_window {
                    cursor_in_window - off
                } else {
                    off - cursor_in_window
                };
                (dist, w.as_str())
            })
            .collect();

        scored.sort_by_key(|(dist, _)| *dist);

        let mut candidates = Vec::new();
        for (dist, word) in scored {
            let lower = word.to_lowercase();
            if !seen.insert(lower) {
                continue;
            }
            // Score: higher for closer words. Use a large base so dabbrev
            // scores are in a comparable range with other providers.
            let score = 1_000_000i64.saturating_sub(dist as i64);
            candidates.push(CompletionCandidate::word(word.to_string(), score));
            if candidates.len() >= MAX_CANDIDATES {
                break;
            }
        }

        ProviderResult::Ready(candidates)
    }

    fn priority(&self) -> u32 {
        30
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_words_basic() {
        let words = extract_words("hello world_foo bar");
        let labels: Vec<&str> = words.iter().map(|(_, w)| w.as_str()).collect();
        assert_eq!(labels, vec!["hello", "world_foo", "bar"]);
    }

    #[test]
    fn extract_words_unicode() {
        let words = extract_words("café naïve über_cool");
        let labels: Vec<&str> = words.iter().map(|(_, w)| w.as_str()).collect();
        assert_eq!(labels, vec!["café", "naïve", "über_cool"]);
    }

    #[test]
    fn extract_words_cjk_and_emoji() {
        // CJK characters are alphanumeric and form their own "words"
        let words = extract_words("foo 変数 bar");
        let labels: Vec<&str> = words.iter().map(|(_, w)| w.as_str()).collect();
        assert_eq!(labels, vec!["foo", "変数", "bar"]);
    }

    #[test]
    fn dabbrev_proximity_ordering() {
        let provider = DabbrevProvider::new();
        let text = b"apple_pie banana apple_sauce cherry apple_tree";
        // Cursor is right after "banana " at byte 22
        let ctx = CompletionContext {
            prefix: "apple".into(),
            cursor_byte: 22,
            word_start_byte: 17,
            buffer_len: text.len(),
            is_large_file: false,
            scan_range: 0..text.len(),
            viewport_top_byte: 0,
            viewport_bottom_byte: text.len(),
            language_id: None,
        };
        let result = provider.provide(&ctx, text);
        match result {
            ProviderResult::Ready(candidates) => {
                let labels: Vec<&str> =
                    candidates.iter().map(|c| c.label.as_str()).collect();
                // apple_sauce (nearest after cursor) should come before apple_pie
                // and apple_tree
                assert_eq!(labels[0], "apple_sauce");
                assert!(labels.contains(&"apple_pie"));
                assert!(labels.contains(&"apple_tree"));
            }
            _ => panic!("expected Ready"),
        }
    }

    #[test]
    fn dabbrev_skips_exact_prefix() {
        let provider = DabbrevProvider::new();
        let text = b"hello hello_world";
        let ctx = CompletionContext {
            prefix: "hello".into(),
            cursor_byte: 5,
            word_start_byte: 0,
            buffer_len: text.len(),
            is_large_file: false,
            scan_range: 0..text.len(),
            viewport_top_byte: 0,
            viewport_bottom_byte: text.len(),
            language_id: None,
        };
        let result = provider.provide(&ctx, text);
        match result {
            ProviderResult::Ready(candidates) => {
                // "hello" alone should not appear—only "hello_world"
                assert_eq!(candidates.len(), 1);
                assert_eq!(candidates[0].label, "hello_world");
            }
            _ => panic!("expected Ready"),
        }
    }

    #[test]
    fn dabbrev_case_insensitive_match() {
        let provider = DabbrevProvider::new();
        let text = b"MyVariable myVar myfunction";
        let ctx = CompletionContext {
            prefix: "my".into(),
            cursor_byte: 0,
            word_start_byte: 0,
            buffer_len: text.len(),
            is_large_file: false,
            scan_range: 0..text.len(),
            viewport_top_byte: 0,
            viewport_bottom_byte: text.len(),
            language_id: None,
        };
        let result = provider.provide(&ctx, text);
        match result {
            ProviderResult::Ready(candidates) => {
                assert_eq!(candidates.len(), 3);
                // All three match case-insensitively
                let labels: Vec<&str> =
                    candidates.iter().map(|c| c.label.as_str()).collect();
                assert!(labels.contains(&"MyVariable"));
                assert!(labels.contains(&"myVar"));
                assert!(labels.contains(&"myfunction"));
            }
            _ => panic!("expected Ready"),
        }
    }
}
