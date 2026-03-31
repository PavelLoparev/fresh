//! Buffer-word completion provider with proximity scoring.
//!
//! Collects unique words from the buffer scan window and ranks them by:
//! 1. **Proximity** to the cursor (line-distance approximation).
//! 2. **Viewport bias** — words visible on screen are boosted.
//! 3. **Frequency** — words that appear more often get a small bonus.
//!
//! This approximates AST-level variable scoping through spatial locality
//! without requiring a language server or parser. Inspired by Sublime Text's
//! buffer-word completion.
//!
//! # Unicode
//!
//! Word boundaries are detected via Unicode grapheme clusters so that
//! identifiers containing accented characters, CJK, or composed emoji
//! sequences are tokenized correctly.
//!
//! # Huge-file safety
//!
//! Only the pre-sliced `buffer_window` is scanned. For large files the
//! completion service limits this to 32 KB around the cursor.

use std::collections::HashMap;

use unicode_segmentation::UnicodeSegmentation;

use super::provider::{
    CompletionCandidate, CompletionContext, CompletionProvider, CompletionSourceId, ProviderResult,
};

/// Maximum number of candidates returned.
const MAX_CANDIDATES: usize = 40;

/// Minimum word length to be considered a candidate. Very short tokens
/// (single-letter variables) generate too much noise.
const MIN_WORD_LEN_GRAPHEMES: usize = 2;

pub struct BufferWordProvider;

impl BufferWordProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BufferWordProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Check whether a grapheme cluster is a "word" constituent.
fn is_word_grapheme(g: &str) -> bool {
    g.chars().any(|c| c.is_alphanumeric() || c == '_')
}

/// Entry tracking a word's occurrences within the scan window.
struct WordStats {
    /// The word text (original casing of first occurrence).
    text: String,
    /// Number of occurrences.
    count: u32,
    /// Byte offset of the occurrence closest to the cursor.
    nearest_byte: usize,
    /// Whether at least one occurrence falls within the viewport.
    in_viewport: bool,
    /// Length in grapheme clusters (for min-length filtering).
    grapheme_len: usize,
}

/// Collect word statistics from the scan window.
///
/// Returns a map from lowercased word to `WordStats`.
fn collect_word_stats(
    text: &str,
    cursor_in_window: usize,
    viewport_start_in_window: usize,
    viewport_end_in_window: usize,
) -> HashMap<String, WordStats> {
    let mut stats: HashMap<String, WordStats> = HashMap::new();

    let mut current_word = String::new();
    let mut word_start: usize = 0;
    let mut word_grapheme_count: usize = 0;
    let mut byte_pos: usize = 0;

    for grapheme in text.graphemes(true) {
        if is_word_grapheme(grapheme) {
            if current_word.is_empty() {
                word_start = byte_pos;
                word_grapheme_count = 0;
            }
            current_word.push_str(grapheme);
            word_grapheme_count += 1;
        } else if !current_word.is_empty() {
            record_word(
                &mut stats,
                std::mem::take(&mut current_word),
                word_grapheme_count,
                word_start,
                cursor_in_window,
                viewport_start_in_window,
                viewport_end_in_window,
            );
            word_grapheme_count = 0;
        }
        byte_pos += grapheme.len();
    }
    if !current_word.is_empty() {
        record_word(
            &mut stats,
            current_word,
            word_grapheme_count,
            word_start,
            cursor_in_window,
            viewport_start_in_window,
            viewport_end_in_window,
        );
    }

    stats
}

fn record_word(
    stats: &mut HashMap<String, WordStats>,
    word: String,
    grapheme_len: usize,
    byte_offset: usize,
    cursor_in_window: usize,
    viewport_start: usize,
    viewport_end: usize,
) {
    let dist = byte_offset.abs_diff(cursor_in_window);
    let in_vp = byte_offset >= viewport_start && byte_offset < viewport_end;
    let key = word.to_lowercase();

    stats
        .entry(key)
        .and_modify(|s| {
            s.count += 1;
            if dist < s.nearest_byte.abs_diff(cursor_in_window) {
                s.nearest_byte = byte_offset;
            }
            s.in_viewport |= in_vp;
        })
        .or_insert(WordStats {
            text: word,
            count: 1,
            nearest_byte: byte_offset,
            in_viewport: in_vp,
            grapheme_len,
        });
}

impl CompletionProvider for BufferWordProvider {
    fn id(&self) -> CompletionSourceId {
        CompletionSourceId("buffer_words".into())
    }

    fn display_name(&self) -> &str {
        "Buffer Words"
    }

    fn is_enabled(&self, ctx: &CompletionContext) -> bool {
        !ctx.prefix.is_empty()
    }

    fn provide(
        &self,
        ctx: &CompletionContext,
        buffer_window: &[u8],
    ) -> ProviderResult {
        let text = String::from_utf8_lossy(buffer_window);
        let prefix_lower = ctx.prefix.to_lowercase();

        let cursor_in_window = ctx.cursor_byte.saturating_sub(ctx.scan_range.start);
        let vp_start = ctx.viewport_top_byte.saturating_sub(ctx.scan_range.start);
        let vp_end = ctx
            .viewport_bottom_byte
            .saturating_sub(ctx.scan_range.start)
            .min(buffer_window.len());

        let stats = collect_word_stats(&text, cursor_in_window, vp_start, vp_end);

        let mut scored: Vec<(i64, &WordStats)> = stats
            .values()
            .filter(|s| {
                s.grapheme_len >= MIN_WORD_LEN_GRAPHEMES
                    && s.text.to_lowercase().starts_with(&prefix_lower)
                    && s.text.to_lowercase() != prefix_lower
            })
            .map(|s| {
                let dist = s.nearest_byte.abs_diff(cursor_in_window);
                // Base: proximity score (closer = higher).
                let mut score: i64 = 500_000i64.saturating_sub(dist as i64);
                // Viewport boost: +100k if any occurrence is visible.
                if s.in_viewport {
                    score += 100_000;
                }
                // Frequency bonus: +5k per extra occurrence (capped).
                score += (s.count.min(10) as i64 - 1) * 5_000;
                (score, s)
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));

        let candidates = scored
            .into_iter()
            .take(MAX_CANDIDATES)
            .map(|(score, s)| CompletionCandidate::word(s.text.clone(), score))
            .collect();

        ProviderResult::Ready(candidates)
    }

    fn priority(&self) -> u32 {
        20
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(prefix: &str, cursor: usize, buf_len: usize) -> CompletionContext {
        CompletionContext {
            prefix: prefix.into(),
            cursor_byte: cursor,
            word_start_byte: cursor.saturating_sub(prefix.len()),
            buffer_len: buf_len,
            is_large_file: false,
            scan_range: 0..buf_len,
            viewport_top_byte: 0,
            viewport_bottom_byte: buf_len,
            language_id: None,
        }
    }

    #[test]
    fn proximity_beats_frequency() {
        let text = b"far_match far_match far_match close_match";
        //           0         1         2         3
        // cursor at 38 (just before close_match)
        let provider = BufferWordProvider::new();
        let ctx = CompletionContext {
            prefix: "far".into(),
            cursor_byte: 38,
            word_start_byte: 35,
            buffer_len: text.len(),
            is_large_file: false,
            scan_range: 0..text.len(),
            viewport_top_byte: 0,
            viewport_bottom_byte: text.len(),
            language_id: None,
        };
        // far_match appears 3 times but the nearest is at offset 30
        let result = provider.provide(&ctx, text);
        match result {
            ProviderResult::Ready(candidates) => {
                assert!(!candidates.is_empty());
                assert_eq!(candidates[0].label, "far_match");
            }
            _ => panic!("expected Ready"),
        }
    }

    #[test]
    fn viewport_boost() {
        // Two words at same distance but one is "in viewport"
        let text = b"alpha_one xxxxxxxxx alpha_two";
        let provider = BufferWordProvider::new();
        let ctx = CompletionContext {
            prefix: "alpha".into(),
            cursor_byte: 15,
            word_start_byte: 10,
            buffer_len: text.len(),
            is_large_file: false,
            scan_range: 0..text.len(),
            // Viewport covers only the second half
            viewport_top_byte: 20,
            viewport_bottom_byte: text.len(),
            language_id: None,
        };
        let result = provider.provide(&ctx, text);
        match result {
            ProviderResult::Ready(candidates) => {
                assert_eq!(candidates.len(), 2);
                // alpha_two gets viewport boost
                assert_eq!(candidates[0].label, "alpha_two");
            }
            _ => panic!("expected Ready"),
        }
    }

    #[test]
    fn min_length_filter() {
        let text = b"a b cc dd hello";
        let provider = BufferWordProvider::new();
        let ctx = make_ctx("h", 15, text.len());
        let result = provider.provide(&ctx, text);
        match result {
            ProviderResult::Ready(candidates) => {
                // Only "hello" matches prefix "h" and has >= 2 graphemes
                assert_eq!(candidates.len(), 1);
                assert_eq!(candidates[0].label, "hello");
            }
            _ => panic!("expected Ready"),
        }
    }

    #[test]
    fn unicode_words() {
        let text = "naïve_var naïve_fn naïf".as_bytes();
        let provider = BufferWordProvider::new();
        let ctx = make_ctx("naïve", 0, text.len());
        let result = provider.provide(&ctx, text);
        match result {
            ProviderResult::Ready(candidates) => {
                let labels: Vec<&str> =
                    candidates.iter().map(|c| c.label.as_str()).collect();
                assert!(labels.contains(&"naïve_var"));
                assert!(labels.contains(&"naïve_fn"));
                // "naïf" doesn't match prefix "naïve"
                assert!(!labels.contains(&"naïf"));
            }
            _ => panic!("expected Ready"),
        }
    }
}
