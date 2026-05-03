//! Scenario framework for editor tests.
//!
//! See `docs/internal/e2e-test-migration-design.md` for the design.
//!
//! Tests express claims as data: `(initial state, action sequence,
//! expected final state)`. A runner instantiates a headless editor,
//! applies the actions through `fresh::test_api::EditorTestApi`, and
//! asserts on the resulting state — no `terminal.draw`, no
//! `crossterm::KeyCode`, no screen scraping.
//!
//! Three drivers consume the same scenario value: the regression
//! runner ([`buffer_scenario::assert_buffer_scenario`]), proptest
//! generators ([`property`]), and shadow-model differentials
//! ([`shadow`]).

pub mod buffer_scenario;
pub mod failure;
pub mod layout_scenario;
pub mod property;
pub mod shadow;
pub mod trace_scenario;
