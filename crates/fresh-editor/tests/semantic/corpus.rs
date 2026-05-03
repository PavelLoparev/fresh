//! Hand-curated corpus of `BufferScenario` values.
//!
//! The corpus is a starting set of scenarios that exercise the bulk
//! of the editor's pure-state surface. It feeds two consumers:
//!
//! - [`shadow_corpus`] — every shadow model runs against every
//!   applicable scenario and must agree with the live editor.
//! - [`corpus_dump`] — serialises the entire set to JSON so external
//!   drivers (proptest soak job, regression harness, CI dashboards)
//!   have a stable, version-controlled starting point.
//!
//! Each entry is a `BufferScenario` value with a unique
//! `description`. As the migration progresses, new scenarios are
//! added here in addition to (not instead of) their domain test
//! file — the per-domain file remains the home for human-readable
//! commentary; this file is the canonical machine-readable list.
//!
//! [`shadow_corpus`]: super::shadow_corpus
//! [`corpus_dump`]: super::corpus_dump

use crate::common::scenario::buffer_scenario::{BufferScenario, CursorExpect};
use fresh::test_api::Action;

pub fn buffer_scenarios() -> Vec<BufferScenario> {
    vec![
        BufferScenario {
            description: "identity: no actions on a non-empty buffer leaves text and cursor".into(),
            initial_text: "hello world".into(),
            actions: vec![],
            expected_text: "hello world".into(),
            expected_primary: CursorExpect::at(0),
            ..Default::default()
        },
        BufferScenario {
            description: "ToUpperCase on a 5-byte selection at byte 0".into(),
            initial_text: "hello world".into(),
            actions: vec![
                Action::SelectRight,
                Action::SelectRight,
                Action::SelectRight,
                Action::SelectRight,
                Action::SelectRight,
                Action::ToUpperCase,
            ],
            expected_text: "HELLO world".into(),
            expected_primary: CursorExpect::at(5),
            expected_selection_text: Some("".into()),
            ..Default::default()
        },
        BufferScenario {
            description: "MoveDocumentEnd then InsertChar appends".into(),
            initial_text: "ab".into(),
            actions: vec![Action::MoveDocumentEnd, Action::InsertChar('c')],
            expected_text: "abc".into(),
            expected_primary: CursorExpect::at(3),
            ..Default::default()
        },
        BufferScenario {
            description: "DeleteForward at start removes the first char".into(),
            initial_text: "abc".into(),
            actions: vec![Action::DeleteForward],
            expected_text: "bc".into(),
            expected_primary: CursorExpect::at(0),
            ..Default::default()
        },
        BufferScenario {
            description: "InsertNewline splits the buffer at the cursor".into(),
            initial_text: "abc".into(),
            actions: vec![
                Action::MoveRight,
                Action::InsertNewline,
            ],
            expected_text: "a\nbc".into(),
            expected_primary: CursorExpect::at(2),
            ..Default::default()
        },
    ]
}
