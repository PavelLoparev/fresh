//! `WorkspaceScenario` — splits, tabs, and buffer-list state.
//!
//! Phase 7 minimal: asserts on the buffer count and the active
//! buffer's display path. Splits/tabs come incrementally as
//! scenarios that need them are added.

use crate::common::harness::EditorTestHarness;
use crate::common::scenario::context::WorkspaceContext;
use crate::common::scenario::failure::ScenarioFailure;
use crate::common::scenario::input_event::InputEvent;
use crate::common::scenario::observable::{Observable, WorkspaceState};
use fresh::test_api::EditorTestApi;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceScenario {
    pub description: String,
    pub workspace: WorkspaceContext,
    pub events: Vec<InputEvent>,
    pub expected: WorkspaceState,
}

pub fn check_workspace_scenario(s: WorkspaceScenario) -> Result<(), ScenarioFailure> {
    if s.workspace.initial_buffers.is_empty() && s.workspace.initial_splits.is_none() {
        return Err(ScenarioFailure::InputProjectionFailed {
            description: s.description,
            reason: "WorkspaceScenario phase: empty workspace context (no buffers or splits)"
                .into(),
        });
    }

    let mut harness = EditorTestHarness::with_temp_project(80, 24)
        .expect("EditorTestHarness::with_temp_project failed");

    // Open every initial buffer; the first becomes active.
    for buf in &s.workspace.initial_buffers {
        let _ = harness
            .load_buffer_from_text_named(&buf.filename, &buf.content)
            .expect("load_buffer_from_text_named failed");
    }

    {
        let api: &mut dyn EditorTestApi = harness.api_mut();
        for ev in &s.events {
            match ev {
                InputEvent::Action(a) => api.dispatch(a.clone()),
                other => {
                    return Err(ScenarioFailure::InputProjectionFailed {
                        description: s.description,
                        reason: format!(
                            "WorkspaceScenario phase: {other:?} not yet routable"
                        ),
                    });
                }
            }
        }
    }

    let actual = WorkspaceState::extract(&mut harness);
    if actual.buffer_count != s.expected.buffer_count {
        return Err(ScenarioFailure::WorkspaceStateMismatch {
            description: s.description,
            field: "buffer_count".into(),
            expected: s.expected.buffer_count.to_string(),
            actual: actual.buffer_count.to_string(),
        });
    }
    // active_buffer_path: only assert if the expectation is non-empty.
    // None on the expected side acts as a wildcard so callers can
    // assert just on count without knowing the exact temp-file path.
    if let Some(want) = &s.expected.active_buffer_path {
        if actual.active_buffer_path.as_deref() != Some(want.as_str()) {
            return Err(ScenarioFailure::WorkspaceStateMismatch {
                description: s.description,
                field: "active_buffer_path".into(),
                expected: format!("{want:?}"),
                actual: format!("{:?}", actual.active_buffer_path),
            });
        }
    }
    Ok(())
}

pub fn assert_workspace_scenario(s: WorkspaceScenario) {
    if let Err(f) = check_workspace_scenario(s) {
        panic!("{f}");
    }
}
