//! Test cases for the text transform plugin
//! Each test:
//! 1. Creates a file with 3 lines (random, test string, random)
//! 2. Selects ONLY the 2nd line
//! 3. Transforms it and verifies FULL buffer (only 2nd line changed)

use crate::common::fixtures::TestFixture;
use crate::common::harness::{copy_plugin, copy_plugin_lib, EditorTestHarness};
use crossterm::event::{KeyCode, KeyModifiers};
use std::time::Duration;

/// Create a harness with the text_transform plugin loaded.
fn text_transform_harness() -> (EditorTestHarness, tempfile::TempDir) {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_root = temp_dir.path().join("project_root");
    std::fs::create_dir_all(&project_root).unwrap();

    let plugins_dir = project_root.join("plugins");
    std::fs::create_dir_all(&plugins_dir).unwrap();
    copy_plugin(&plugins_dir, "text_transform");
    copy_plugin_lib(&plugins_dir);

    let mut harness =
        EditorTestHarness::with_config_and_working_dir(80, 24, Default::default(), project_root)
            .unwrap();

    // Wait for the plugin to be fully loaded
    for _ in 0..10 {
        let _ = harness.render();
        std::thread::sleep(Duration::from_millis(50));
    }

    (harness, temp_dir)
}

/// Helper to move to a specific line (1-based) and select it
fn select_line(harness: &mut EditorTestHarness, line_num: usize) {
    // Move to start of buffer
    let _ = harness.send_key(KeyCode::Home, KeyModifiers::CONTROL);
    harness.render().unwrap();

    // Move down to the target line
    for _ in 1..line_num {
        let _ = harness.send_key(KeyCode::Down, KeyModifiers::NONE);
    }

    // Select the entire line using Shift+End
    let _ = harness.send_key(KeyCode::End, KeyModifiers::SHIFT);
}

/// Helper to run a command via the command palette
fn run_command(harness: &mut EditorTestHarness, command: &str) {
    // Open command palette
    let _ = harness.send_key(KeyCode::Char('p'), KeyModifiers::CONTROL);
    let _ = harness.render();

    // Type the command name
    let _ = harness.type_text(command);
    let _ = harness.render();

    // Press Enter to execute
    let _ = harness.send_key(KeyCode::Enter, KeyModifiers::NONE);
    let _ = harness.render();
}

#[test]
fn test_transform_to_kebab_case() {
    let (mut harness, _temp_dir) = text_transform_harness();

    // Create file with 3 lines
    let line1 = "1st string";
    let line2 = "Title Case, camelCase, kebab-case, snake_case, PascalCase";
    let line3 = "last string";
    let content = format!("{}\n{}\n{}", line1, line2, line3);
    let fixture = TestFixture::new("test.txt", &content).unwrap();
    harness.open_file(&fixture.path).unwrap();
    harness.render().unwrap();

    // Select ONLY the 2nd line
    select_line(&mut harness, 2);
    assert!(
        harness.has_selection(),
        "Should have selection before string processing"
    );

    // Run Command
    run_command(&mut harness, "Transform to kebab-case");

    // Wait for command to execute
    std::thread::sleep(Duration::from_secs(1));
    harness.render().unwrap();

    // Verify FULL buffer content - only 2nd line should be changed
    let result = harness.get_buffer_content().unwrap();
    let expected_encoded = format!(
        "{}\n{}\n{}",
        line1, "title case, camel-case, kebab-case, snake-case, pascal-case", line3
    );
    assert_eq!(
        result, expected_encoded,
        "After text propcessing, only 2nd line should change. Got:\n{}",
        result
    );
}

#[test]
fn test_transform_to_title_case() {
    let (mut harness, _temp_dir) = text_transform_harness();

    // Create file with 3 lines
    let line1 = "1st string";
    let line2 = "Title Case, camelCase, kebab-case, snake_case, PascalCase";
    let line3 = "last string";
    let content = format!("{}\n{}\n{}", line1, line2, line3);
    let fixture = TestFixture::new("test.txt", &content).unwrap();
    harness.open_file(&fixture.path).unwrap();
    harness.render().unwrap();

    // Select ONLY the 2nd line
    select_line(&mut harness, 2);
    assert!(
        harness.has_selection(),
        "Should have selection before string processing"
    );

    // Run Command
    run_command(&mut harness, "Transform to Title Case");

    // Wait for command to execute
    std::thread::sleep(Duration::from_secs(1));
    harness.render().unwrap();

    // Verify FULL buffer content - only 2nd line should be changed
    let result = harness.get_buffer_content().unwrap();
    let expected_encoded = format!(
        "{}\n{}\n{}",
        line1, "Title Case, Camelcase, Kebab-Case, Snake_Case, Pascalcase", line3
    );
    assert_eq!(
        result, expected_encoded,
        "After text propcessing, only 2nd line should change. Got:\n{}",
        result
    );
}

#[test]
fn test_transform_to_snake_case() {
    let (mut harness, _temp_dir) = text_transform_harness();

    // Create file with 3 lines
    let line1 = "1st string";
    let line2 = "Title Case, camelCase, kebab-case, snake_case, PascalCase";
    let line3 = "last string";
    let content = format!("{}\n{}\n{}", line1, line2, line3);
    let fixture = TestFixture::new("test.txt", &content).unwrap();
    harness.open_file(&fixture.path).unwrap();
    harness.render().unwrap();

    // Select ONLY the 2nd line
    select_line(&mut harness, 2);
    assert!(
        harness.has_selection(),
        "Should have selection before string processing"
    );

    // Run Command
    run_command(&mut harness, "Transform to snake_case");

    // Wait for command to execute
    std::thread::sleep(Duration::from_secs(1));
    harness.render().unwrap();

    // Verify FULL buffer content - only 2nd line should be changed
    let result = harness.get_buffer_content().unwrap();
    let expected_encoded = format!(
        "{}\n{}\n{}",
        line1, "title case, camel_case, kebab-case, snake_case, pascal_case", line3
    );
    assert_eq!(
        result, expected_encoded,
        "After text propcessing, only 2nd line should change. Got:\n{}",
        result
    );
}

#[test]
fn test_transform_to_camel_case() {
    let (mut harness, _temp_dir) = text_transform_harness();

    // Create file with 3 lines
    let line1 = "1st string";
    let line2 = "Title Case, camelCase, kebab-case, snake_case, PascalCase";
    let line3 = "last string";
    let content = format!("{}\n{}\n{}", line1, line2, line3);
    let fixture = TestFixture::new("test.txt", &content).unwrap();
    harness.open_file(&fixture.path).unwrap();
    harness.render().unwrap();

    // Select ONLY the 2nd line
    select_line(&mut harness, 2);
    assert!(
        harness.has_selection(),
        "Should have selection before string processing"
    );

    // Run Command
    run_command(&mut harness, "Transform to camelCase");

    // Wait for command to execute
    std::thread::sleep(Duration::from_secs(1));
    harness.render().unwrap();

    // Verify FULL buffer content - only 2nd line should be changed
    let result = harness.get_buffer_content().unwrap();
    let expected_encoded = format!(
        "{}\n{}\n{}",
        line1, "titleCase,CamelCase,KebabCase,SnakeCase,PascalCase", line3
    );
    assert_eq!(
        result, expected_encoded,
        "After text propcessing, only 2nd line should change. Got:\n{}",
        result
    );
}

#[test]
fn test_transform_to_pascal_case() {
    let (mut harness, _temp_dir) = text_transform_harness();

    // Create file with 3 lines
    let line1 = "1st string";
    let line2 = "Title Case, camelCase, kebab-case, snake_case, PascalCase";
    let line3 = "last string";
    let content = format!("{}\n{}\n{}", line1, line2, line3);
    let fixture = TestFixture::new("test.txt", &content).unwrap();
    harness.open_file(&fixture.path).unwrap();
    harness.render().unwrap();

    // Select ONLY the 2nd line
    select_line(&mut harness, 2);
    assert!(
        harness.has_selection(),
        "Should have selection before string processing"
    );

    // Run Command
    run_command(&mut harness, "Transform to PascalCase");

    // Wait for command to execute
    std::thread::sleep(Duration::from_secs(1));
    harness.render().unwrap();

    // Verify FULL buffer content - only 2nd line should be changed
    let result = harness.get_buffer_content().unwrap();
    let expected_encoded = format!(
        "{}\n{}\n{}",
        line1, "TitleCase,CamelCase,KebabCase,SnakeCase,PascalCase", line3
    );
    assert_eq!(
        result, expected_encoded,
        "After text propcessing, only 2nd line should change. Got:\n{}",
        result
    );
}
