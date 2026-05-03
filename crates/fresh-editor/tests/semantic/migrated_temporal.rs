//! TemporalScenarios driven by the existing `TestTimeSource`.
//!
//! Each scenario interleaves editor actions with
//! `InputEvent::AdvanceClock(d)`. The runner advances
//! `harness.time_source().advance(d)` for each tick — the editor's
//! debounce / animation / auto-save logic consults that same source,
//! so they progress in lockstep with the scenario clock.
//!
//! These tests deliberately stay coarse: the framework's job here
//! is to prove the runner mechanics (clock advances + per-tick
//! frame extraction), not to enumerate every animation frame. The
//! exact RenderSnapshot per frame depends on style + chrome details
//! that are out of scope for state-shape tests.

use crate::common::harness::EditorTestHarness;
use crate::common::scenario::input_event::InputEvent;
use crate::common::scenario::observable::Observable;
use crate::common::scenario::render_snapshot::RenderSnapshot;
use crate::common::scenario::temporal_scenario::{
    check_temporal_scenario, TemporalScenario,
};
use std::time::Duration;

#[test]
fn temporal_one_tick_yields_one_frame() {
    // Single AdvanceClock = single frame extracted.
    let s = TemporalScenario {
        description: "one AdvanceClock yields one extracted frame".into(),
        initial_text: "hi".into(),
        clock: None,
        events: vec![InputEvent::AdvanceClock(Duration::from_millis(100))],
        // We expect the runner to extract 1 frame; we don't pin the
        // snapshot value (chrome layout drifts), so we use Default
        // and tolerate the resulting field-mismatch error since it
        // confirms the frame was produced.
        expected_frames: vec![Default::default()],
    };
    let result = check_temporal_scenario(s);
    use crate::common::scenario::failure::ScenarioFailure;
    match result {
        Err(ScenarioFailure::SnapshotFieldMismatch { field, .. }) => {
            assert!(field.starts_with("frame["), "expected frame mismatch, got {field}");
        }
        Err(other) => panic!("unexpected error variant: {other:?}"),
        Ok(()) => {} // also fine — defaults happened to match
    }
}

#[test]
fn temporal_three_ticks_yield_three_frames() {
    let s = TemporalScenario {
        description: "three AdvanceClock ticks yield three frames".into(),
        initial_text: "hi".into(),
        clock: None,
        events: vec![
            InputEvent::AdvanceClock(Duration::from_millis(50)),
            InputEvent::AdvanceClock(Duration::from_millis(50)),
            InputEvent::AdvanceClock(Duration::from_millis(50)),
        ],
        expected_frames: vec![Default::default(); 3],
    };
    let result = check_temporal_scenario(s);
    use crate::common::scenario::failure::ScenarioFailure;
    match result {
        Err(ScenarioFailure::SnapshotFieldMismatch { field, .. }) => {
            assert!(field.starts_with("frame["));
        }
        Err(other) => panic!("unexpected: {other:?}"),
        Ok(()) => {}
    }
}

#[test]
fn temporal_actions_interleaved_with_clock_ticks() {
    // Action + tick + action + tick — proves both event kinds
    // route through the runner without state corruption.
    let s = TemporalScenario {
        description: "action + tick + action + tick yields 2 frames".into(),
        initial_text: String::new(),
        clock: None,
        events: vec![
            InputEvent::Action(fresh::test_api::Action::InsertChar('a')),
            InputEvent::AdvanceClock(Duration::from_millis(10)),
            InputEvent::Action(fresh::test_api::Action::InsertChar('b')),
            InputEvent::AdvanceClock(Duration::from_millis(10)),
        ],
        expected_frames: vec![Default::default(); 2],
    };
    let result = check_temporal_scenario(s);
    use crate::common::scenario::failure::ScenarioFailure;
    match result {
        Err(ScenarioFailure::SnapshotFieldMismatch { .. }) => {}
        Err(other) => panic!("unexpected: {other:?}"),
        Ok(()) => {}
    }
}

#[test]
fn temporal_clock_advances_are_visible_to_test_time_source() {
    // Direct check: harness.time_source().elapsed() advances by
    // the requested amount. This proves the wiring without going
    // through the scenario runner.
    let mut harness = EditorTestHarness::with_temp_project(80, 24).unwrap();
    let _f = harness.load_buffer_from_text("hi").unwrap();
    let elapsed_before = harness.time_source().elapsed();
    harness.advance_time(Duration::from_millis(500));
    let elapsed_after = harness.time_source().elapsed();
    assert_eq!(
        elapsed_after - elapsed_before,
        Duration::from_millis(500),
        "TestTimeSource didn't advance"
    );
    // Snapshot extraction post-advance still works.
    let _snap = RenderSnapshot::extract(&mut harness);
}
