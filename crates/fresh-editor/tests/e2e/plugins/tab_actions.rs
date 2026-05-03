//! E2E tests for tab actions plugin

use crate::common::harness::{copy_plugin, copy_plugin_lib, EditorTestHarness};
use crossterm::event::{KeyCode, KeyModifiers};
use std::time::Duration;
use tempfile::TempDir;

fn tab_actions_harness() -> (EditorTestHarness, tempfile::TempDir) {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_root = temp_dir.path().join("project_root");
    std::fs::create_dir_all(&project_root).unwrap();

    let plugins_dir = project_root.join("plugins");
    std::fs::create_dir_all(&plugins_dir).unwrap();
    copy_plugin(&plugins_dir, "tab_actions");
    copy_plugin_lib(&plugins_dir);

    let mut harness =
        EditorTestHarness::with_config_and_working_dir(80, 24, Default::default(), project_root)
            .unwrap();

    for _ in 0..10 {
        let _ = harness.render();
        std::thread::sleep(Duration::from_millis(50));
    }

    (harness, temp_dir)
}

#[test]
fn test_close_other_buffers() {
    let (mut harness, temp_dir) = tab_actions_harness();
    let project_root = temp_dir.path().join("project_root");

    // Create files in project root so quick open can find them
    let file1 = project_root.join("file1.txt");
    let file2 = project_root.join("file2.txt");
    let file3 = project_root.join("file3.txt");
    std::fs::write(&file1, "Content 1").unwrap();
    std::fs::write(&file2, "Content 2").unwrap();
    std::fs::write(&file3, "Content 3").unwrap();

    // Open all 3 files
    harness.open_file(&file1).unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("file1.txt");

    harness.open_file(&file2).unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("file2.txt");

    harness.open_file(&file3).unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("file3.txt");

    // Verify all 3 tabs are open
    let screen_after_opening = harness.screen_to_string();
    let file1_count = screen_after_opening.matches("file1.txt").count();
    let file2_count = screen_after_opening.matches("file2.txt").count();
    let file3_count = screen_after_opening.matches("file3.txt").count();
    assert!(
        file1_count >= 1 && file2_count >= 1 && file3_count >= 1,
        "Expected all 3 files in tabs, found file1={}, file2={}, file3={}. Screen:\n{}",
        file1_count,
        file2_count,
        file3_count,
        screen_after_opening
    );

    // Switch to file2 using Quick Open buffer mode
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness
        .send_key(KeyCode::Backspace, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("file2").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Verify we're now on file2
    harness.assert_screen_contains("Content 2");

    // Run "Close Other Tabs" command via Ctrl+P
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("Close Other Tabs").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Verify only file2 remains
    let screen = harness.screen_to_string();
    assert!(
        !screen.contains("file1.txt"),
        "Expected file1.txt to be closed. Screen:\n{}",
        screen
    );
    assert!(
        screen.contains("file2.txt"),
        "Expected file2.txt to remain. Screen:\n{}",
        screen
    );
    assert!(
        !screen.contains("file3.txt"),
        "Expected file3.txt to be closed. Screen:\n{}",
        screen
    );
}

#[test]
fn test_close_all_buffers() {
    let (mut harness, temp_dir) = tab_actions_harness();
    let project_root = temp_dir.path().join("project_root");

    // Create files in project root so quick open can find them
    let file1 = project_root.join("file1.txt");
    let file2 = project_root.join("file2.txt");
    let file3 = project_root.join("file3.txt");
    std::fs::write(&file1, "Content 1").unwrap();
    std::fs::write(&file2, "Content 2").unwrap();
    std::fs::write(&file3, "Content 3").unwrap();

    // Open all 3 files
    harness.open_file(&file1).unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("file1.txt");

    harness.open_file(&file2).unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("file2.txt");

    harness.open_file(&file3).unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("file3.txt");

    // Verify all 3 tabs are open
    let screen_after_opening = harness.screen_to_string();
    let file1_count = screen_after_opening.matches("file1.txt").count();
    let file2_count = screen_after_opening.matches("file2.txt").count();
    let file3_count = screen_after_opening.matches("file3.txt").count();
    assert!(
        file1_count >= 1 && file2_count >= 1 && file3_count >= 1,
        "Expected all 3 files in tabs, found file1={}, file2={}, file3={}. Screen:\n{}",
        file1_count,
        file2_count,
        file3_count,
        screen_after_opening
    );

    // Switch to file2 using Quick Open buffer mode
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness
        .send_key(KeyCode::Backspace, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("file2").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Verify we're now on file2
    harness.assert_screen_contains("Content 2");

    // Run "Close All Tabs" command via Ctrl+P
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("Close All Tabs").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Verify no files remain
    let screen = harness.screen_to_string();
    assert!(
        !screen.contains("file1.txt"),
        "Expected file1.txt to be closed. Screen:\n{}",
        screen
    );
    assert!(
        !screen.contains("file2.txt"),
        "Expected file2.txt to be closed. Screen:\n{}",
        screen
    );
    assert!(
        !screen.contains("file3.txt"),
        "Expected file3.txt to be closed. Screen:\n{}",
        screen
    );
}

#[test]
fn test_close_buffers_to_left() {
    let (mut harness, temp_dir) = tab_actions_harness();
    let project_root = temp_dir.path().join("project_root");

    // Create files in project root so quick open can find them
    let file1 = project_root.join("file1.txt");
    let file2 = project_root.join("file2.txt");
    let file3 = project_root.join("file3.txt");
    std::fs::write(&file1, "Content 1").unwrap();
    std::fs::write(&file2, "Content 2").unwrap();
    std::fs::write(&file3, "Content 3").unwrap();

    // Open all 3 files
    harness.open_file(&file1).unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("file1.txt");

    harness.open_file(&file2).unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("file2.txt");

    harness.open_file(&file3).unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("file3.txt");

    // Verify all 3 tabs are open
    let screen_after_opening = harness.screen_to_string();
    let file1_count = screen_after_opening.matches("file1.txt").count();
    let file2_count = screen_after_opening.matches("file2.txt").count();
    let file3_count = screen_after_opening.matches("file3.txt").count();
    assert!(
        file1_count >= 1 && file2_count >= 1 && file3_count >= 1,
        "Expected all 3 files in tabs, found file1={}, file2={}, file3={}. Screen:\n{}",
        file1_count,
        file2_count,
        file3_count,
        screen_after_opening
    );

    // Switch to file2 using Quick Open buffer mode
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness
        .send_key(KeyCode::Backspace, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("file2").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Verify we're now on file2
    harness.assert_screen_contains("Content 2");

    // Run "Close Tabs To Left" command via Ctrl+P
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("Close Tabs To Left").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Verify only file2 and file3 remain
    let screen = harness.screen_to_string();
    assert!(
        !screen.contains("file1.txt"),
        "Expected file1.txt to be closed. Screen:\n{}",
        screen
    );
    assert!(
        screen.contains("file2.txt"),
        "Expected file2.txt to remain. Screen:\n{}",
        screen
    );
    assert!(
        screen.contains("file3.txt"),
        "Expected file3.txt to remain. Screen:\n{}",
        screen
    );
}

#[test]
fn test_close_buffers_to_right() {
    let (mut harness, temp_dir) = tab_actions_harness();
    let project_root = temp_dir.path().join("project_root");

    // Create files in project root so quick open can find them
    let file1 = project_root.join("file1.txt");
    let file2 = project_root.join("file2.txt");
    let file3 = project_root.join("file3.txt");
    std::fs::write(&file1, "Content 1").unwrap();
    std::fs::write(&file2, "Content 2").unwrap();
    std::fs::write(&file3, "Content 3").unwrap();

    // Open all 3 files
    harness.open_file(&file1).unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("file1.txt");

    harness.open_file(&file2).unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("file2.txt");

    harness.open_file(&file3).unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("file3.txt");

    // Verify all 3 tabs are open
    let screen_after_opening = harness.screen_to_string();
    let file1_count = screen_after_opening.matches("file1.txt").count();
    let file2_count = screen_after_opening.matches("file2.txt").count();
    let file3_count = screen_after_opening.matches("file3.txt").count();
    assert!(
        file1_count >= 1 && file2_count >= 1 && file3_count >= 1,
        "Expected all 3 files in tabs, found file1={}, file2={}, file3={}. Screen:\n{}",
        file1_count,
        file2_count,
        file3_count,
        screen_after_opening
    );

    // Switch to file2 using Quick Open buffer mode
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness
        .send_key(KeyCode::Backspace, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("file2").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Verify we're now on file2
    harness.assert_screen_contains("Content 2");

    // Run "Close Tabs To Left" command via Ctrl+P
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("Close Tabs To Right").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Verify only file1 and file2 remain
    let screen = harness.screen_to_string();
    assert!(
        screen.contains("file1.txt"),
        "Expected file1.txt to remain. Screen:\n{}",
        screen
    );
    assert!(
        screen.contains("file2.txt"),
        "Expected file2.txt to remain. Screen:\n{}",
        screen
    );
    assert!(
        !screen.contains("file3.txt"),
        "Expected file3.txt to be closed. Screen:\n{}",
        screen
    );
}

#[test]
fn test_move_tab_left() {
    let (mut harness, temp_dir) = tab_actions_harness();
    let project_root = temp_dir.path().join("project_root");

    let file1 = project_root.join("file1.txt");
    let file2 = project_root.join("file2.txt");
    let file3 = project_root.join("file3.txt");
    std::fs::write(&file1, "Content 1").unwrap();
    std::fs::write(&file2, "Content 2").unwrap();
    std::fs::write(&file3, "Content 3").unwrap();

    // Open all 3 files (file1, file2, file3)
    harness.open_file(&file1).unwrap();
    harness.render().unwrap();
    harness.open_file(&file2).unwrap();
    harness.render().unwrap();
    harness.open_file(&file3).unwrap();
    harness.render().unwrap();

    let split_id = harness.editor().get_active_split();
    let tabs = harness.editor().get_split_tabs(split_id);
    assert_eq!(tabs.len(), 3, "Should have 3 tabs");
    let file1_id = tabs[0];
    let file2_id = tabs[1];
    let file3_id = tabs[2];

    // Run "Move Tab Left" - file3 should move left by one
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("Move Tab Left").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Verify tab order changed: file1, file3, file2
    let tabs_after = harness.editor().get_split_tabs(split_id);
    assert_eq!(tabs_after[0], file1_id, "First tab should be file1");
    assert_eq!(
        tabs_after[1], file3_id,
        "Second tab should be file3 (moved left)"
    );
    assert_eq!(tabs_after[2], file2_id, "Third tab should be file2");

    // Move tab left again - file3 should swap with file1
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("Move Tab Left").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Verify: file3, file1, file2
    let tabs_final = harness.editor().get_split_tabs(split_id);
    assert_eq!(
        tabs_final[0], file3_id,
        "First tab should be file3 (now first)"
    );
    assert_eq!(tabs_final[1], file1_id, "Second tab should be file1");
    assert_eq!(tabs_final[2], file2_id, "Third tab should be file2");

    // Move tab left again - file3 is already at first, should do nothing
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("Move Tab Left").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Verify order unchanged
    let tabs_unchanged = harness.editor().get_split_tabs(split_id);
    assert_eq!(tabs_unchanged[0], file3_id, "First should still be file3");
    assert_eq!(tabs_unchanged[1], file1_id, "Second should still be file1");
    assert_eq!(tabs_unchanged[2], file2_id, "Third should still be file2");
}

#[test]
fn test_move_tab_right() {
    let (mut harness, temp_dir) = tab_actions_harness();
    let project_root = temp_dir.path().join("project_root");

    let file1 = project_root.join("file1.txt");
    let file2 = project_root.join("file2.txt");
    let file3 = project_root.join("file3.txt");
    std::fs::write(&file1, "Content 1").unwrap();
    std::fs::write(&file2, "Content 2").unwrap();
    std::fs::write(&file3, "Content 3").unwrap();

    // Open all 3 files (file1, file2, file3)
    harness.open_file(&file1).unwrap();
    harness.render().unwrap();
    harness.open_file(&file2).unwrap();
    harness.render().unwrap();
    harness.open_file(&file3).unwrap();
    harness.render().unwrap();

    let split_id = harness.editor().get_active_split();
    let tabs = harness.editor().get_split_tabs(split_id);
    assert_eq!(tabs.len(), 3, "Should have 3 tabs");
    let file1_id = tabs[0];
    let file2_id = tabs[1];
    let file3_id = tabs[2];

    // Run "Move Tab Right" - file3 is at last position, should do nothing
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("Move Tab Right").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Verify order unchanged
    let tabs_after = harness.editor().get_split_tabs(split_id);
    assert_eq!(tabs_after[0], file1_id, "First should be file1");
    assert_eq!(tabs_after[1], file2_id, "Second should be file2");
    assert_eq!(tabs_after[2], file3_id, "Third should be file3 (unchanged)");

    // Switch to file1 (first tab)
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness
        .send_key(KeyCode::Backspace, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("file1").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Move tab right - file1 should move to position 2
    // After move: file2, file1, file3
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("Move Tab Right").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    let tabs_after_move1 = harness.editor().get_split_tabs(split_id);
    assert_eq!(tabs_after_move1[0], file2_id, "First should now be file2");
    assert_eq!(
        tabs_after_move1[1], file1_id,
        "Second should be file1 (moved right)"
    );
    assert_eq!(tabs_after_move1[2], file3_id, "Third should be file3");

    // Move tab right again - file1 should move to last position
    // After move: file2, file3, file1
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("Move Tab Right").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    let tabs_after_move2 = harness.editor().get_split_tabs(split_id);
    assert_eq!(tabs_after_move2[0], file2_id, "First should be file2");
    assert_eq!(tabs_after_move2[1], file3_id, "Second should be file3");
    assert_eq!(
        tabs_after_move2[2], file1_id,
        "Third should be file1 (now last)"
    );
}
