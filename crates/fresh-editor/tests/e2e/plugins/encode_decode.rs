//! Test cases for the encode decode plugin
//! Each test:
//! 1. Creates a file with 3 lines (random, test string, random)
//! 2. Selects ONLY the 2nd line
//! 3. Encodes it and verifies FULL buffer (only 2nd line changed)
//! 4. Selects the 2nd line again
//! 5. Decodes back and verifies FULL buffer (all lines back to original)

use crate::common::fixtures::TestFixture;
use crate::common::harness::{copy_plugin, copy_plugin_lib, EditorTestHarness};
use crossterm::event::{KeyCode, KeyModifiers};
use std::time::Duration;

/// Create a harness with the enode_decode plugin loaded.
fn encode_decode_harness() -> (EditorTestHarness, tempfile::TempDir) {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_root = temp_dir.path().join("project_root");
    std::fs::create_dir_all(&project_root).unwrap();

    let plugins_dir = project_root.join("plugins");
    std::fs::create_dir_all(&plugins_dir).unwrap();
    copy_plugin(&plugins_dir, "encode_decode");
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
fn test_base64_roundtrip() {
    let (mut harness, _temp_dir) = encode_decode_harness();

    // Create file with 3 lines: random, test string, random
    let line1 = "1st string";
    let line2 = "Hello World";
    let line3 = "last string";
    let content = format!("{}\n{}\n{}", line1, line2, line3);
    let fixture = TestFixture::new("test.txt", &content).unwrap();
    harness.open_file(&fixture.path).unwrap();
    harness.render().unwrap();

    // Select ONLY the 2nd line
    select_line(&mut harness, 2);
    assert!(
        harness.has_selection(),
        "Should have selection before Base64 encode"
    );

    // Run "String to Base64"
    run_command(&mut harness, "String to Base64");

    // Wait for command to execute
    std::thread::sleep(Duration::from_secs(1));
    harness.render().unwrap();

    // Verify FULL buffer content - only 2nd line should be changed
    let result = harness.get_buffer_content().unwrap();
    let expected_encoded = format!("{}\n{}\n{}", line1, "SGVsbG8gV29ybGQ=", line3);
    assert_eq!(
        result, expected_encoded,
        "After Base64 encode, only 2nd line should change. Got:\n{}",
        result
    );

    // Select the 2nd line again (now encoded)
    select_line(&mut harness, 2);

    // Run "Base64 to String"
    run_command(&mut harness, "Base64 to String");

    // Wait for command to execute
    std::thread::sleep(Duration::from_secs(1));
    harness.render().unwrap();

    // Verify FULL buffer content - all lines should be back to original
    let result = harness.get_buffer_content().unwrap();
    let expected_decoded = format!("{}\n{}\n{}", line1, line2, line3);
    assert_eq!(
        result, expected_decoded,
        "After Base64 decode, all lines should be back to original. Got:\n{}",
        result
    );
}

#[test]
fn test_json_string_roundtrip() {
    let (mut harness, _temp_dir) = encode_decode_harness();

    // Create file with 3 lines: random, test string, random
    let line1 = "1st string";
    let line2 = "Hello \"World\"";
    let line3 = "last string";
    let content = format!("{}\n{}\n{}", line1, line2, line3);
    let fixture = TestFixture::new("test.txt", &content).unwrap();
    harness.open_file(&fixture.path).unwrap();
    harness.render().unwrap();

    // Select ONLY the 2nd line
    select_line(&mut harness, 2);

    // Run "String to JSON String"
    run_command(&mut harness, "String to JSON String");

    // Wait for command to execute
    std::thread::sleep(Duration::from_secs(1));
    harness.render().unwrap();

    // Verify FULL buffer content - only 2nd line should be changed
    let result = harness.get_buffer_content().unwrap();
    let expected_encoded = format!("{}\n{}\n{}", line1, "\"Hello \\\"World\\\"\"", line3);
    assert_eq!(
        result, expected_encoded,
        "After JSON encode, only 2nd line should change. Got:\n{}",
        result
    );

    // Select the 2nd line again (now encoded)
    select_line(&mut harness, 2);

    // Run "JSON String to String"
    run_command(&mut harness, "JSON String to String");

    // Wait for command to execute
    std::thread::sleep(Duration::from_secs(1));
    harness.render().unwrap();

    // Verify FULL buffer content - all lines should be back to original
    let result = harness.get_buffer_content().unwrap();
    let expected_decoded = format!("{}\n{}\n{}", line1, line2, line3);
    assert_eq!(
        result, expected_decoded,
        "After JSON decode, all lines should be back to original. Got:\n{}",
        result
    );
}

#[test]
fn test_uri_encoded_roundtrip() {
    let (mut harness, _temp_dir) = encode_decode_harness();

    // Create file with 3 lines: random, test string, random
    let line1 = "1st string";
    let line2 = "https://example.com/path?query=test value";
    let line3 = "last string";
    let content = format!("{}\n{}\n{}", line1, line2, line3);
    let fixture = TestFixture::new("test.txt", &content).unwrap();
    harness.open_file(&fixture.path).unwrap();
    harness.render().unwrap();

    // Select ONLY the 2nd line
    select_line(&mut harness, 2);

    // Run "String to URI Encoded" (uses encodeURI - preserves reserved chars)
    run_command(&mut harness, "String to URI Encoded");

    // Wait for command to execute
    std::thread::sleep(Duration::from_secs(1));
    harness.render().unwrap();

    // Verify FULL buffer content - only 2nd line should be changed
    // encodeURI does NOT encode: :/?#[]@!$&'()*+,;=
    let result = harness.get_buffer_content().unwrap();
    let expected_encoded = format!(
        "{}\n{}\n{}",
        line1, "https://example.com/path?query=test%20value", line3
    );
    assert_eq!(
        result, expected_encoded,
        "encodeURI should not change this URI. Got:\n{}",
        result
    );

    // Select the 2nd line again
    select_line(&mut harness, 2);

    // Run "URI Encoded to String"
    run_command(&mut harness, "URI Encoded to String");

    // Wait for command to execute
    std::thread::sleep(Duration::from_secs(1));
    harness.render().unwrap();

    // Verify FULL buffer content - all lines should be back to original
    let result = harness.get_buffer_content().unwrap();
    let expected_decoded = format!("{}\n{}\n{}", line1, line2, line3);
    assert_eq!(
        result, expected_decoded,
        "After URI decode, all lines should be back to original. Got:\n{}",
        result
    );
}

#[test]
fn test_uri_component_encoded_roundtrip() {
    let (mut harness, _temp_dir) = encode_decode_harness();

    // Create file with 3 lines: random, test string, random
    let line1 = "1st string";
    let line2 = "query[]=test value";
    let line3 = "last string";
    let content = format!("{}\n{}\n{}", line1, line2, line3);
    let fixture = TestFixture::new("test.txt", &content).unwrap();
    harness.open_file(&fixture.path).unwrap();
    harness.render().unwrap();

    // Select ONLY the 2nd line
    select_line(&mut harness, 2);

    // Run "String to URI Component Encoded" (uses encodeURIComponent - encodes everything)
    run_command(&mut harness, "String to URI Component Encoded");

    // Wait for command to execute
    std::thread::sleep(Duration::from_secs(1));
    harness.render().unwrap();

    // Verify FULL buffer content - only 2nd line should be changed
    let result = harness.get_buffer_content().unwrap();
    // encodeURIComponent does NOT encode: !'()*_~ (but encodes spaces as %20, Chinese chars, etc.)
    let expected_encoded = format!("{}\n{}\n{}", line1, "query%5B%5D%3Dtest%20value", line3);
    assert_eq!(
        result, expected_encoded,
        "After URI Component encode, only 2nd line should change. Got:\n{}",
        result
    );

    // Select the 2nd line again (now encoded)
    select_line(&mut harness, 2);

    // Run "URI Component Encoded to String"
    run_command(&mut harness, "URI Component Encoded to String");

    // Wait for command to execute
    std::thread::sleep(Duration::from_secs(1));
    harness.render().unwrap();

    // Verify FULL buffer content - all lines should be back to original
    let result = harness.get_buffer_content().unwrap();
    let expected_decoded = format!("{}\n{}\n{}", line1, line2, line3);
    assert_eq!(
        result, expected_decoded,
        "After URI Component decode, all lines should be back to original. Got:\n{}",
        result
    );
}

#[test]
fn test_hex_string_roundtrip() {
    let (mut harness, _temp_dir) = encode_decode_harness();

    // Create file with 3 lines: random, test string, random
    let line1 = "1st string";
    let line2 = "[72,101,108,108,111]"; // No spaces
    let line3 = "last string";
    let content = format!("{}\n{}\n{}", line1, line2, line3);
    let fixture = TestFixture::new("test.txt", &content).unwrap();
    harness.open_file(&fixture.path).unwrap();
    harness.render().unwrap();

    // Select ONLY the 2nd line.
    select_line(&mut harness, 2);

    // Run "JSON Byte Array to Hex String"
    run_command(&mut harness, "JSON Byte Array to Hex String");

    // Wait for command to execute.
    std::thread::sleep(Duration::from_secs(1));
    harness.render().unwrap();

    // Verify FULL buffer content - only 2nd line should be changed.
    let result = harness.get_buffer_content().unwrap();
    let expected_encoded = format!("{}\n{}\n{}", line1, "48656c6c6f", line3);
    assert_eq!(
        result, expected_encoded,
        "After hex encode, only 2nd line should change. Got:\n{}",
        result
    );

    // Select the 2nd line again (now encoded).
    select_line(&mut harness, 2);

    // Run "Hex String to JSON Byte Array"
    run_command(&mut harness, "Hex String to JSON Byte Array");

    // Wait for command to execute.
    std::thread::sleep(Duration::from_secs(1));
    harness.render().unwrap();

    // Verify FULL buffer content - all lines should be back to original.
    let result = harness.get_buffer_content().unwrap();
    // JSON might add spaces back.
    let expected_decoded = format!("{}\n{}\n{}", line1, line2, line3);
    assert_eq!(
        result, expected_decoded,
        "After hex decode, all lines should be back to original. Got:\n{}",
        result
    );
}
