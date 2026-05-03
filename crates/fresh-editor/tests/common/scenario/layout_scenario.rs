//! `LayoutScenario` — layout-dependent observables.
//!
//! Layout state (viewport scroll, hardware cursor screen position,
//! scrollbar geometry) is reconciled by the render pipeline, not by
//! action dispatch alone. `LayoutScenario` runs a single render pass
//! at the end of the action sequence so layout state settles before
//! assertion. Scenarios still avoid `for { send_key; render; }` style
//! imperative transcripts.
//!
//! Phase 1 surface is intentionally narrow: just `viewport_top_byte`.
//! Richer layout observables (cursor screen position, gutter widths,
//! scrollbar thumb extent) belong in a future `RenderSnapshot` and
//! land alongside the LayoutScenario expansion phase.

use crate::common::harness::EditorTestHarness;
use crate::common::scenario::failure::ScenarioFailure;
use fresh::test_api::{Action, EditorTestApi};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LayoutScenario {
    pub description: String,
    pub initial_text: String,
    pub width: u16,
    pub height: u16,
    pub actions: Vec<Action>,
    pub expected_top_byte: usize,
}

impl Default for LayoutScenario {
    fn default() -> Self {
        Self {
            description: String::new(),
            initial_text: String::new(),
            width: 80,
            height: 24,
            actions: Vec::new(),
            expected_top_byte: 0,
        }
    }
}

pub fn check_layout_scenario(s: LayoutScenario) -> Result<(), ScenarioFailure> {
    let mut harness = EditorTestHarness::with_temp_project(s.width, s.height)
        .expect("EditorTestHarness::with_temp_project failed");
    let _fixture = harness
        .load_buffer_from_text(&s.initial_text)
        .expect("load_buffer_from_text failed");

    // Render once after load so the initial viewport reconciles to the
    // buffer geometry — without this, the editor's first layout pass
    // hasn't computed view lines yet and `top_byte` reads 0 even when
    // ensure_visible would otherwise scroll.
    harness.render().expect("initial render failed");

    {
        let api: &mut dyn EditorTestApi = harness.api_mut();
        api.dispatch_seq(&s.actions);
    }

    // Single layout pass *after* the full action sequence. This is the
    // only structural difference from `BufferScenario`.
    harness.render().expect("final render failed");

    let actual = harness.api_mut().viewport_top_byte();
    if actual != s.expected_top_byte {
        return Err(ScenarioFailure::ViewportTopByteMismatch {
            description: s.description,
            expected: s.expected_top_byte,
            actual,
        });
    }
    Ok(())
}

pub fn assert_layout_scenario(s: LayoutScenario) {
    if let Err(f) = check_layout_scenario(s) {
        panic!("{f}");
    }
}
