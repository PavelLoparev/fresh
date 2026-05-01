//! E2E coverage for the `@` Quick Open document symbols provider.
//!
//! Drives a fake LSP server that advertises `documentSymbolProvider: true`
//! and verifies:
//!   1. The `@` quick open populates suggestions formatted as
//!      "{indent}[{kind}] {name}" with children indented under their parents.
//!   2. Confirming a suggestion sets a buffer selection covering the
//!      symbol's full body range.

use crate::common::fake_lsp::FakeLspServer;
use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};

fn make_config_with_fake_lsp(temp_dir: &std::path::Path) -> fresh::config::Config {
    let mut config = fresh::config::Config::default();
    config.lsp.insert(
        "json".to_string(),
        fresh::types::LspLanguageConfig::Multi(vec![fresh::services::lsp::LspServerConfig {
            command: FakeLspServer::document_symbols_script_path(temp_dir)
                .to_string_lossy()
                .to_string(),
            args: vec![],
            enabled: true,
            auto_start: true,
            process_limits: fresh::services::process_limits::ProcessLimits::default(),
            initialization_options: None,
            env: Default::default(),
            language_id_overrides: Default::default(),
            root_markers: Default::default(),
            name: Some("fake-doc-symbols-ls".to_string()),
            only_features: None,
            except_features: None,
        }]),
    );
    config
}

#[test]
#[cfg_attr(target_os = "windows", ignore = "FakeLspServer uses Bash")]
fn test_quick_open_symbols_populates_and_indents() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let _fake_server = FakeLspServer::spawn_with_document_symbols(temp_dir.path())?;

    // The fake server's symbol ranges reach line 60, so write enough lines
    // for the selection ranges in the second test to land inside the buffer.
    let test_file = temp_dir.path().join("test.json");
    let mut content = String::from("{\n");
    for _ in 1..=70 {
        content.push_str("  \"k\": \"v\",\n");
    }
    content.push('}');
    std::fs::write(&test_file, content)?;

    let config = make_config_with_fake_lsp(temp_dir.path());
    let mut harness = EditorTestHarness::create(
        100,
        30,
        crate::common::harness::HarnessOptions::new()
            .with_config(config)
            .with_working_dir(temp_dir.path().to_path_buf()),
    )?;

    harness.open_file(&test_file)?;
    harness.render()?;
    harness.wait_until(|h| h.editor().initialized_lsp_server_count("json") > 0)?;

    harness
        .editor_mut()
        .dispatch_action_for_tests(fresh::input::keybindings::Action::QuickOpenSymbols);
    harness.render()?;

    // After the LSP responds the suggestion list contains the three symbols;
    // children are indented with two spaces per level under their parent.
    harness.wait_until(|h| {
        let s = h.screen_to_string();
        s.contains("[class] Outer")
            && s.contains("  [method] inner_a")
            && s.contains("  [method] inner_b")
            && s.contains("[function] top_level")
    })?;

    harness.send_key(KeyCode::Esc, KeyModifiers::NONE)?;
    Ok(())
}

#[test]
#[cfg_attr(target_os = "windows", ignore = "FakeLspServer uses Bash")]
fn test_quick_open_symbols_confirm_selects_full_range() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let _fake_server = FakeLspServer::spawn_with_document_symbols(temp_dir.path())?;

    let test_file = temp_dir.path().join("test.json");
    let mut content = String::from("{\n");
    for _ in 1..=70 {
        content.push_str("  \"k\": \"v\",\n");
    }
    content.push('}');
    std::fs::write(&test_file, content)?;

    let config = make_config_with_fake_lsp(temp_dir.path());
    let mut harness = EditorTestHarness::create(
        100,
        30,
        crate::common::harness::HarnessOptions::new()
            .with_config(config)
            .with_working_dir(temp_dir.path().to_path_buf()),
    )?;

    harness.open_file(&test_file)?;
    harness.render()?;
    harness.wait_until(|h| h.editor().initialized_lsp_server_count("json") > 0)?;

    harness
        .editor_mut()
        .dispatch_action_for_tests(fresh::input::keybindings::Action::QuickOpenSymbols);
    harness.render()?;

    harness.wait_until(|h| h.screen_to_string().contains("[class] Outer"))?;

    // Confirm the first suggestion ([class] Outer, range lines 1..40 LSP
    // 0-indexed → lines 2..41 in 1-indexed editor coordinates).
    harness.send_key(KeyCode::Enter, KeyModifiers::NONE)?;
    harness.render()?;

    let editor = harness.editor();
    let primary = editor.active_cursors().primary();
    assert!(
        primary.anchor.is_some(),
        "confirming a symbol must produce a selection"
    );

    let state = editor.active_state();
    let selection_start = primary.anchor.unwrap().min(primary.position);
    let selection_end = primary.anchor.unwrap().max(primary.position);

    let start_line = state.buffer.position_to_line_col(selection_start).0;
    let end_line = state.buffer.position_to_line_col(selection_end).0;

    // LSP returned start.line = 1 / end.line = 40 (0-indexed). Compare in the
    // same 0-indexed space the model uses internally.
    assert_eq!(start_line, 1, "selection should start at LSP line 1");
    assert_eq!(end_line, 40, "selection should cover through LSP line 40");

    Ok(())
}
