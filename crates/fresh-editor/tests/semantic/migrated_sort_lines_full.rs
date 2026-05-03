//! Faithful migration of `tests/e2e/sort_lines.rs`.
//!
//! The keymap dependency (Ctrl+P → "sort lines" → Enter) is
//! lifted to `Action::SortLines`. Each scenario also pins the
//! cursor + selection state at t=∞ — the original e2e tests did
//! not assert on cursor, so these scenarios add coverage.
//!
//! **Finding pinned here** (see
//! `docs/internal/scenario-migration-findings.md` §7):
//! `SelectAll + SortLines` preserves the selection anchor when
//! the buffer is unchanged (already-sorted / single-line cases)
//! but clears it when the buffer is mutated. That asymmetry was
//! invisible to the original e2e tests; pinning it here so a
//! future change is flagged.

use crate::common::scenario::buffer_scenario::{
    assert_buffer_scenario, BufferScenario, CursorExpect,
};
use crate::common::scenario::trace_scenario::{
    assert_trace_scenario, TraceScenario,
};
use fresh::test_api::Action;

#[test]
fn migrated_sort_lines_basic() {
    // Original: `tests/e2e/sort_lines.rs::test_sort_lines_basic`.
    // FINDING: anchor cleared on mutation (cursor at end-of-buffer,
    // anchor None). E2e didn't assert on this.
    assert_buffer_scenario(BufferScenario {
        description: "SortLines on 3 unsorted lines yields alphabetical order".into(),
        initial_text: "cherry\nbanana\napple".into(),
        actions: vec![Action::SelectAll, Action::SortLines],
        expected_text: "apple\nbanana\ncherry".into(),
        expected_primary: CursorExpect::at(19),
        ..Default::default()
    });
}

#[test]
fn migrated_sort_lines_single_line_no_change() {
    // Original: `test_sort_lines_single_line_no_change`.
    // FINDING: anchor preserved (Some(0)) — buffer didn't change.
    assert_buffer_scenario(BufferScenario {
        description: "SortLines on single-line buffer preserves SelectAll anchor".into(),
        initial_text: "only line".into(),
        actions: vec![Action::SelectAll, Action::SortLines],
        expected_text: "only line".into(),
        expected_primary: CursorExpect::range(0, 9),
        ..Default::default()
    });
}

#[test]
fn migrated_sort_lines_with_numbers_uses_lexicographic_order() {
    // Original: `test_sort_lines_with_numbers`.
    assert_buffer_scenario(BufferScenario {
        description: "SortLines uses lexicographic, not numeric, ordering".into(),
        initial_text: "10 items\n2 items\n1 item".into(),
        actions: vec![Action::SelectAll, Action::SortLines],
        expected_text: "1 item\n10 items\n2 items".into(),
        expected_primary: CursorExpect::at(23),
        ..Default::default()
    });
}

#[test]
fn migrated_sort_lines_preserves_trailing_newline() {
    // Original: `test_sort_lines_preserves_trailing_newline`.
    assert_buffer_scenario(BufferScenario {
        description: "SortLines preserves trailing newline".into(),
        initial_text: "zebra\napple\nmango\n".into(),
        actions: vec![Action::SelectAll, Action::SortLines],
        expected_text: "apple\nmango\nzebra\n".into(),
        expected_primary: CursorExpect::at(18),
        ..Default::default()
    });
}

#[test]
fn migrated_sort_lines_undo_restores_original_order() {
    // Original: `test_sort_lines_undo`.
    assert_trace_scenario(TraceScenario {
        description: "SortLines + Undo restores original ordering".into(),
        initial_text: "cherry\napple\nbanana".into(),
        actions: vec![Action::SelectAll, Action::SortLines],
        expected_text: "apple\nbanana\ncherry".into(),
        undo_count: 1,
    });
}

#[test]
fn migrated_sort_lines_already_sorted_is_noop() {
    // Original: `test_sort_lines_already_sorted`.
    // FINDING: anchor preserved (Some(0)) — buffer didn't change.
    assert_buffer_scenario(BufferScenario {
        description: "SortLines is idempotent on already-sorted input; anchor preserved".into(),
        initial_text: "apple\nbanana\ncherry".into(),
        actions: vec![Action::SelectAll, Action::SortLines],
        expected_text: "apple\nbanana\ncherry".into(),
        expected_primary: CursorExpect::range(0, 19),
        ..Default::default()
    });
}

#[test]
fn migrated_sort_lines_case_sensitive_ascii_ordering() {
    // Original: `test_sort_lines_case_sensitive`.
    // ASCII case ordering: uppercase comes first ('A'=0x41, 'a'=0x61).
    assert_buffer_scenario(BufferScenario {
        description: "SortLines uses case-sensitive ASCII order: uppercase first".into(),
        initial_text: "banana\nApple\ncherry\nBerry".into(),
        actions: vec![Action::SelectAll, Action::SortLines],
        expected_text: "Apple\nBerry\nbanana\ncherry".into(),
        expected_primary: CursorExpect::at(25),
        ..Default::default()
    });
}

#[test]
fn migrated_sort_lines_with_empty_lines() {
    // Original: `test_sort_lines_with_empty_lines`. Empty lines
    // sort to the top (empty string < anything else).
    assert_buffer_scenario(BufferScenario {
        description: "SortLines puts empty lines first".into(),
        initial_text: "cherry\n\napple\n\nbanana".into(),
        actions: vec![Action::SelectAll, Action::SortLines],
        expected_text: "\n\napple\nbanana\ncherry".into(),
        expected_primary: CursorExpect::at(21),
        ..Default::default()
    });
}
