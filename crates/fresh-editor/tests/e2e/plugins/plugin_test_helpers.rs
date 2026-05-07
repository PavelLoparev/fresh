//! Shared test helpers for plugin E2E tests

use crate::common::fixtures::TestFixture;
use crate::common::harness::{copy_plugin, copy_plugin_lib, EditorTestHarness};
use crossterm::event::{KeyCode, KeyModifiers};

pub fn select_line(harness: &mut EditorTestHarness, line_num: usize) {
    let _ = harness.send_key(KeyCode::Home, KeyModifiers::CONTROL);
    harness.render().unwrap();

    for _ in 1..line_num {
        let _ = harness.send_key(KeyCode::Down, KeyModifiers::NONE);
    }

    let _ = harness.send_key(KeyCode::End, KeyModifiers::SHIFT);
}

pub fn run_command(harness: &mut EditorTestHarness, command: &str) {
    let _ = harness.send_key(KeyCode::Char('p'), KeyModifiers::CONTROL);
    let _ = harness.render();

    let _ = harness.type_text(command);
    let _ = harness.render();

    let _ = harness.send_key(KeyCode::Enter, KeyModifiers::NONE);
    let _ = harness.render();
}