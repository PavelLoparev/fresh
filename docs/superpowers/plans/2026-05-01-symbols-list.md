# Go to Symbol in File — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `@`-prefix quick-open provider that lists LSP document symbols with indented hierarchy, live preview, and full-range selection on confirm.

**Architecture:** Stateless `SymbolProvider` reads a `Vec<FlatSymbol>` injected into `QuickOpenContext`. The editor fires `textDocument/documentSymbol` when `@` is opened, caches the flattened result in `Editor::symbol_cache`, and injects it into context each frame. Preview reuses the existing `goto_line_preview` snapshot/restore mechanism.

**Tech Stack:** Rust, lsp-types crate (`DocumentSymbol`, `SymbolKind`), existing `QuickOpenProvider` trait, `AsyncMessage` / `LspCommand` async pipeline.

---

### Task 1: FlatSymbol model, flatten_symbols, symbol_kind_label

**Files:**
- Modify: `crates/fresh-editor/src/input/quick_open/providers.rs` (add before the `#[cfg(test)]` block)

- [ ] **Step 1: Add FlatSymbol struct and helpers**

  Add this block directly before the `#[cfg(test)]` block in `providers.rs`:

  ```rust
  // ============================================================================
  // Symbol Provider — shared types
  // ============================================================================

  /// A flattened LSP DocumentSymbol entry for display in Quick Open.
  #[derive(Debug, Clone)]
  pub struct FlatSymbol {
      pub name: String,
      pub kind: lsp_types::SymbolKind,
      /// Nesting depth (0 = top-level). Capped at 4 for display.
      pub depth: u8,
      /// LSP 0-indexed start line
      pub start_line: u32,
      /// LSP 0-indexed start character
      pub start_char: u32,
      /// LSP 0-indexed end line (full body range)
      pub end_line: u32,
      /// LSP 0-indexed end character (full body range)
      pub end_char: u32,
  }

  /// Short lowercase label for a SymbolKind (used in "[kind] name" display).
  pub fn symbol_kind_label(kind: lsp_types::SymbolKind) -> &'static str {
      use lsp_types::SymbolKind;
      match kind {
          SymbolKind::FILE => "file",
          SymbolKind::MODULE => "module",
          SymbolKind::NAMESPACE => "namespace",
          SymbolKind::PACKAGE => "package",
          SymbolKind::CLASS => "class",
          SymbolKind::METHOD => "method",
          SymbolKind::PROPERTY => "property",
          SymbolKind::FIELD => "field",
          SymbolKind::CONSTRUCTOR => "constructor",
          SymbolKind::ENUM => "enum",
          SymbolKind::INTERFACE => "interface",
          SymbolKind::FUNCTION => "function",
          SymbolKind::VARIABLE => "variable",
          SymbolKind::CONSTANT => "const",
          SymbolKind::STRING => "string",
          SymbolKind::NUMBER => "number",
          SymbolKind::BOOLEAN => "boolean",
          SymbolKind::ARRAY => "array",
          SymbolKind::OBJECT => "object",
          SymbolKind::KEY => "key",
          SymbolKind::NULL => "null",
          SymbolKind::ENUM_MEMBER => "enum_member",
          SymbolKind::STRUCT => "struct",
          SymbolKind::EVENT => "event",
          SymbolKind::OPERATOR => "operator",
          SymbolKind::TYPE_PARAMETER => "type_param",
          _ => "symbol",
      }
  }

  /// Flatten a recursive DocumentSymbol tree into a depth-first ordered Vec.
  /// Depth is capped at 4 to keep indentation sane.
  pub fn flatten_symbols(symbols: Vec<lsp_types::DocumentSymbol>) -> Vec<FlatSymbol> {
      let mut out = Vec::new();
      flatten_recursive(&symbols, 0, &mut out);
      out
  }

  fn flatten_recursive(
      symbols: &[lsp_types::DocumentSymbol],
      depth: u8,
      out: &mut Vec<FlatSymbol>,
  ) {
      for sym in symbols {
          out.push(FlatSymbol {
              name: sym.name.clone(),
              kind: sym.kind,
              depth: depth.min(4),
              start_line: sym.range.start.line,
              start_char: sym.range.start.character,
              end_line: sym.range.end.line,
              end_char: sym.range.end.character,
          });
          if let Some(children) = &sym.children {
              flatten_recursive(children, depth + 1, out);
          }
      }
  }
  ```

- [ ] **Step 2: Write unit tests for flatten_symbols and symbol_kind_label**

  Add inside the existing `#[cfg(test)] mod tests` block:

  ```rust
  #[test]
  fn test_flatten_symbols_depth_first_order() {
      use lsp_types::{DocumentSymbol, Position, Range, SymbolKind, SymbolTag};
      fn make_sym(name: &str, line: u32, children: Option<Vec<DocumentSymbol>>) -> DocumentSymbol {
          DocumentSymbol {
              name: name.to_string(),
              detail: None,
              kind: SymbolKind::CLASS,
              tags: None,
              deprecated: None,
              range: Range {
                  start: Position { line, character: 0 },
                  end: Position { line: line + 5, character: 0 },
              },
              selection_range: Range {
                  start: Position { line, character: 0 },
                  end: Position { line, character: 5 },
              },
              children,
          }
      }

      let tree = vec![make_sym(
          "MyClass",
          0,
          Some(vec![
              make_sym("method_a", 1, None),
              make_sym("method_b", 3, None),
          ]),
      )];

      let flat = flatten_symbols(tree);
      assert_eq!(flat.len(), 3);
      assert_eq!(flat[0].name, "MyClass");
      assert_eq!(flat[0].depth, 0);
      assert_eq!(flat[1].name, "method_a");
      assert_eq!(flat[1].depth, 1);
      assert_eq!(flat[2].name, "method_b");
      assert_eq!(flat[2].depth, 1);
  }

  #[test]
  fn test_flatten_symbols_caps_depth_at_4() {
      use lsp_types::{DocumentSymbol, Position, Range, SymbolKind};
      fn leaf(name: &str) -> DocumentSymbol {
          DocumentSymbol {
              name: name.to_string(),
              detail: None,
              kind: SymbolKind::VARIABLE,
              tags: None,
              deprecated: None,
              range: Range {
                  start: Position { line: 0, character: 0 },
                  end: Position { line: 1, character: 0 },
              },
              selection_range: Range {
                  start: Position { line: 0, character: 0 },
                  end: Position { line: 0, character: 3 },
              },
              children: None,
          }
      }
      fn wrap(name: &str, child: DocumentSymbol) -> DocumentSymbol {
          let mut p = leaf(name);
          p.children = Some(vec![child]);
          p
      }

      // 6 levels deep
      let tree = vec![wrap("l0", wrap("l1", wrap("l2", wrap("l3", wrap("l4", leaf("l5"))))))];
      let flat = flatten_symbols(tree);
      assert_eq!(flat.len(), 6);
      assert_eq!(flat[5].depth, 4); // capped
  }

  #[test]
  fn test_symbol_kind_label() {
      use lsp_types::SymbolKind;
      assert_eq!(symbol_kind_label(SymbolKind::CLASS), "class");
      assert_eq!(symbol_kind_label(SymbolKind::FUNCTION), "function");
      assert_eq!(symbol_kind_label(SymbolKind::CONSTANT), "const");
  }
  ```

- [ ] **Step 3: Run the tests**

  ```bash
  cargo test --package fresh-editor test_flatten_symbols test_symbol_kind_label -- --nocapture
  ```

  Expected: all 3 pass.

- [ ] **Step 4: Commit**

  ```bash
  git add crates/fresh-editor/src/input/quick_open/providers.rs
  git commit -m "feat(symbols): add FlatSymbol model, flatten_symbols, and symbol_kind_label"
  ```

---

### Task 2: QuickOpenResult::GotoSymbol + QuickOpenContext::document_symbols + Editor::symbol_cache

**Files:**
- Modify: `crates/fresh-editor/src/input/quick_open/mod.rs`
- Modify: `crates/fresh-editor/src/app/mod.rs`
- Modify: `crates/fresh-editor/src/app/prompt_lifecycle.rs`

- [ ] **Step 1: Add GotoSymbol to QuickOpenResult**

  In `crates/fresh-editor/src/input/quick_open/mod.rs`, add a new variant to the `QuickOpenResult` enum after `GotoLine`:

  ```rust
  /// Jump to a symbol in the current buffer and select its full range.
  /// All coordinates are LSP 0-indexed.
  GotoSymbol {
      start_line: u32,
      start_char: u32,
      end_line: u32,
      end_char: u32,
  },
  ```

- [ ] **Step 2: Add document_symbols field to QuickOpenContext**

  In the same file, add to the `QuickOpenContext` struct after `relative_line_numbers`:

  ```rust
  /// Document symbols for the active buffer, injected from the LSP symbol cache.
  /// Empty vec when not yet loaded or no LSP available.
  pub document_symbols: Vec<crate::input::quick_open::providers::FlatSymbol>,
  ```

- [ ] **Step 3: Add symbol_cache to Editor struct**

  In `crates/fresh-editor/src/app/mod.rs`, add after the `goto_line_preview` field (~line 867):

  ```rust
  /// Cached document symbols for the most recently requested buffer.
  /// Populated by `handle_lsp_document_symbols` and injected into
  /// `QuickOpenContext` while the `@` quick-open is active.
  symbol_cache: Option<(crate::model::event::BufferId, Vec<crate::input::quick_open::providers::FlatSymbol>)>,

  /// Request ID of the in-flight `textDocument/documentSymbol` request, if any.
  pending_symbol_request_id: Option<u64>,
  ```

- [ ] **Step 4: Initialize new fields in Editor::new / editor_init.rs**

  In `crates/fresh-editor/src/app/editor_init.rs`, find where `goto_line_preview: None` is set in the `Editor { ... }` struct literal and add below it:

  ```rust
  symbol_cache: None,
  pending_symbol_request_id: None,
  ```

- [ ] **Step 5: Update build_quick_open_context to inject document_symbols**

  In `crates/fresh-editor/src/app/prompt_lifecycle.rs`, in `build_quick_open_context()` (~line 185), update the returned `QuickOpenContext { ... }` literal to add:

  ```rust
  document_symbols: {
      let active = self.active_buffer();
      self.symbol_cache
          .as_ref()
          .filter(|(buf_id, _)| *buf_id == active)
          .map(|(_, syms)| syms.clone())
          .unwrap_or_default()
  },
  ```

- [ ] **Step 6: Update make_test_context in providers.rs tests**

  In `crates/fresh-editor/src/input/quick_open/providers.rs`, add `document_symbols: vec![]` to the `QuickOpenContext` literal inside `make_test_context`:

  ```rust
  document_symbols: vec![],
  ```

- [ ] **Step 7: Cargo check**

  ```bash
  cargo check --package fresh-editor 2>&1 | head -30
  ```

  Expected: no errors.

- [ ] **Step 8: Commit**

  ```bash
  git add crates/fresh-editor/src/input/quick_open/mod.rs \
          crates/fresh-editor/src/app/mod.rs \
          crates/fresh-editor/src/app/prompt_lifecycle.rs \
          crates/fresh-editor/src/app/editor_init.rs \
          crates/fresh-editor/src/input/quick_open/providers.rs
  git commit -m "feat(symbols): add GotoSymbol result, document_symbols context, and symbol_cache"
  ```

---

### Task 3: SymbolProvider

**Files:**
- Modify: `crates/fresh-editor/src/input/quick_open/providers.rs`
- Modify: `crates/fresh-editor/src/input/quick_open/mod.rs` (re-export)

- [ ] **Step 1: Add SymbolProvider struct**

  Add directly after the `flatten_recursive` function (before `#[cfg(test)]`):

  ```rust
  // ============================================================================
  // Symbol Provider (prefix: "@")
  // ============================================================================

  /// Provider for "Go to Symbol in File" (LSP textDocument/documentSymbol).
  pub struct SymbolProvider;

  impl SymbolProvider {
      pub fn new() -> Self {
          Self
      }
  }

  impl Default for SymbolProvider {
      fn default() -> Self {
          Self::new()
      }
  }

  impl QuickOpenProvider for SymbolProvider {
      fn prefix(&self) -> &str {
          "@"
      }

      fn suggestions(&self, query: &str, ctx: &QuickOpenContext) -> Vec<Suggestion> {
          if ctx.document_symbols.is_empty() {
              let msg = if ctx.has_lsp_config {
                  t!("symbols.loading").to_string()
              } else {
                  t!("symbols.no_lsp").to_string()
              };
              return vec![Suggestion::disabled(msg)];
          }

          let mut matcher = FuzzyMatcher::new(query);
          ctx.document_symbols
              .iter()
              .filter(|sym| query.is_empty() || matcher.match_target(&sym.name).matched)
              .map(|sym| {
                  let indent = " ".repeat(sym.depth as usize * 2);
                  let kind = symbol_kind_label(sym.kind);
                  let text = format!("{}[{}] {}", indent, kind, sym.name);
                  let value = format!(
                      "{}:{}:{}:{}",
                      sym.start_line, sym.start_char, sym.end_line, sym.end_char
                  );
                  Suggestion::new(text).with_value(value)
              })
              .collect()
      }

      fn on_select(
          &self,
          suggestion: Option<&Suggestion>,
          _query: &str,
          _ctx: &QuickOpenContext,
      ) -> QuickOpenResult {
          let Some(s) = suggestion else {
              return QuickOpenResult::None;
          };
          if s.disabled {
              return QuickOpenResult::None;
          }
          let Some(value) = &s.value else {
              return QuickOpenResult::None;
          };
          // value = "start_line:start_char:end_line:end_char" (LSP 0-indexed)
          let parts: Vec<&str> = value.splitn(4, ':').collect();
          if parts.len() != 4 {
              return QuickOpenResult::None;
          }
          let (Ok(sl), Ok(sc), Ok(el), Ok(ec)) = (
              parts[0].parse::<u32>(),
              parts[1].parse::<u32>(),
              parts[2].parse::<u32>(),
              parts[3].parse::<u32>(),
          ) else {
              return QuickOpenResult::None;
          };
          QuickOpenResult::GotoSymbol {
              start_line: sl,
              start_char: sc,
              end_line: el,
              end_char: ec,
          }
      }

      fn as_any(&self) -> &dyn std::any::Any {
          self
      }
  }
  ```

- [ ] **Step 2: Re-export SymbolProvider from mod.rs**

  In `crates/fresh-editor/src/input/quick_open/mod.rs`, update the existing re-export line:

  ```rust
  pub use providers::{BufferProvider, CommandProvider, FileProvider, GotoLineProvider, SymbolProvider};
  ```

- [ ] **Step 3: Write unit tests for SymbolProvider**

  Add inside the `#[cfg(test)] mod tests` block in `providers.rs`:

  ```rust
  fn make_symbol_context(syms: Vec<FlatSymbol>, has_lsp: bool) -> QuickOpenContext {
      let mut ctx = make_test_context("/tmp");
      ctx.document_symbols = syms;
      ctx.has_lsp_config = has_lsp;
      ctx
  }

  fn make_flat(name: &str, kind: lsp_types::SymbolKind, depth: u8, sl: u32, sc: u32, el: u32, ec: u32) -> FlatSymbol {
      FlatSymbol { name: name.to_string(), kind, depth, start_line: sl, start_char: sc, end_line: el, end_char: ec }
  }

  #[test]
  fn test_symbol_provider_no_lsp() {
      let provider = SymbolProvider::new();
      let ctx = make_symbol_context(vec![], false);
      let s = provider.suggestions("", &ctx);
      assert_eq!(s.len(), 1);
      assert!(s[0].disabled);
  }

  #[test]
  fn test_symbol_provider_loading() {
      let provider = SymbolProvider::new();
      let ctx = make_symbol_context(vec![], true);
      let s = provider.suggestions("", &ctx);
      assert_eq!(s.len(), 1);
      assert!(s[0].disabled);
  }

  #[test]
  fn test_symbol_provider_empty_query_all_symbols() {
      use lsp_types::SymbolKind;
      let provider = SymbolProvider::new();
      let syms = vec![
          make_flat("MyClass", SymbolKind::CLASS, 0, 0, 0, 10, 0),
          make_flat("my_method", SymbolKind::METHOD, 1, 2, 4, 8, 5),
      ];
      let ctx = make_symbol_context(syms, true);
      let s = provider.suggestions("", &ctx);
      assert_eq!(s.len(), 2);
      assert!(s[0].text.contains("[class] MyClass"));
      assert!(s[1].text.starts_with("  [method]")); // 2-space indent
  }

  #[test]
  fn test_symbol_provider_filters_by_name() {
      use lsp_types::SymbolKind;
      let provider = SymbolProvider::new();
      let syms = vec![
          make_flat("MyClass", SymbolKind::CLASS, 0, 0, 0, 10, 0),
          make_flat("other_fn", SymbolKind::FUNCTION, 0, 11, 0, 15, 0),
      ];
      let ctx = make_symbol_context(syms, true);
      let s = provider.suggestions("my", &ctx);
      assert_eq!(s.len(), 1);
      assert!(s[0].text.contains("MyClass"));
  }

  #[test]
  fn test_symbol_provider_on_select_parses_value() {
      use lsp_types::SymbolKind;
      let provider = SymbolProvider::new();
      let syms = vec![make_flat("foo", SymbolKind::FUNCTION, 0, 5, 2, 15, 0)];
      let ctx = make_symbol_context(syms, true);
      let suggestions = provider.suggestions("", &ctx);
      let result = provider.on_select(suggestions.first(), "", &ctx);
      match result {
          QuickOpenResult::GotoSymbol { start_line, start_char, end_line, end_char } => {
              assert_eq!(start_line, 5);
              assert_eq!(start_char, 2);
              assert_eq!(end_line, 15);
              assert_eq!(end_char, 0);
          }
          other => panic!("expected GotoSymbol, got {:?}", other),
      }
  }
  ```

- [ ] **Step 4: Add locale keys for loading/no_lsp messages**

  In every file under `crates/fresh-editor/locales/` add these two keys (use the same pattern as nearby keys):

  ```json
  "symbols.loading": "Loading symbols…",
  "symbols.no_lsp": "No language server available",
  "symbols.not_found": "No symbols found",
  ```

- [ ] **Step 5: Run provider tests**

  ```bash
  cargo test --package fresh-editor test_symbol_provider -- --nocapture
  ```

  Expected: 5 tests pass.

- [ ] **Step 6: Commit**

  ```bash
  git add crates/fresh-editor/src/input/quick_open/providers.rs \
          crates/fresh-editor/src/input/quick_open/mod.rs \
          crates/fresh-editor/locales/
  git commit -m "feat(symbols): add SymbolProvider with fuzzy filtering and indented display"
  ```

---

### Task 4: LSP pipeline

**Files:**
- Modify: `crates/fresh-editor/src/services/lsp/async_handler.rs`
- Modify: `crates/fresh-editor/src/services/async_bridge.rs`
- Modify: `crates/fresh-editor/src/services/lsp/manager.rs`
- Modify: `crates/fresh-editor/src/app/async_dispatch.rs`
- Modify: `crates/fresh-editor/src/app/async_messages.rs`
- Modify: `crates/fresh-editor/src/app/lsp_requests.rs`

- [ ] **Step 1: Add LspCommand::DocumentSymbols variant**

  In `async_handler.rs`, add after `LspCommand::FoldingRange { ... }` (~line 649):

  ```rust
  /// Request document symbols for a file
  DocumentSymbols { request_id: u64, uri: Uri },
  ```

- [ ] **Step 2: Add handle_document_symbols method to LspServerState**

  In `async_handler.rs`, add after `handle_folding_ranges` (~line 2303):

  ```rust
  async fn handle_document_symbols(
      &self,
      request_id: u64,
      uri: Uri,
      pending: &PendingRequests,
  ) -> Result<(), String> {
      use lsp_types::{DocumentSymbolParams, TextDocumentIdentifier};

      tracing::trace!("LSP: document symbols request for {}", uri.as_str());

      let params = DocumentSymbolParams {
          text_document: TextDocumentIdentifier { uri: uri.clone() },
          work_done_progress_params: Default::default(),
          partial_result_params: Default::default(),
      };

      match self
          .send_request_sequential::<_, Option<lsp_types::DocumentSymbolResponse>>(
              "textDocument/documentSymbol",
              Some(params),
              pending,
          )
          .await
      {
          Ok(response) => {
              let symbols = match response {
                  Some(lsp_types::DocumentSymbolResponse::Nested(syms)) => syms,
                  Some(lsp_types::DocumentSymbolResponse::Flat(_)) | None => vec![],
              };
              let _ = self.async_tx.send(AsyncMessage::LspDocumentSymbols {
                  request_id,
                  uri: uri.as_str().to_string(),
                  symbols,
              });
              Ok(())
          }
          Err(e) => {
              tracing::debug!("Document symbols request failed: {}", e);
              let _ = self.async_tx.send(AsyncMessage::LspDocumentSymbols {
                  request_id,
                  uri: uri.as_str().to_string(),
                  symbols: vec![],
              });
              Err(e)
          }
      }
  }
  ```

- [ ] **Step 3: Handle LspCommand::DocumentSymbols in the run() dispatch loop**

  In `async_handler.rs`, add after the `LspCommand::FoldingRange` arm (~line 3262):

  ```rust
  LspCommand::DocumentSymbols { request_id, uri } => {
      if initialized {
          tracing::info!("Processing DocumentSymbols request for {}", uri.as_str());
          spawn_request!(state, pending, |s, p| s
              .handle_document_symbols(request_id, uri, &p)
              .await);
      } else {
          tracing::trace!("LSP not initialized, cannot get document symbols");
          let _ = state.async_tx.send(AsyncMessage::LspDocumentSymbols {
              request_id,
              uri: uri.as_str().to_string(),
              symbols: Vec::new(),
          });
      }
  }
  ```

- [ ] **Step 4: Also handle in replay_pending_commands (after the FoldingRange arm ~line 881)**

  ```rust
  LspCommand::DocumentSymbols { request_id, uri } => {
      let _ = s.handle_document_symbols(request_id, uri, &p).await;
  }
  ```

- [ ] **Step 5: Add document_symbols() method to LspHandle**

  In `async_handler.rs`, add after `folding_ranges()` (~line 4598):

  ```rust
  /// Request document symbols for a file
  pub fn document_symbols(&self, request_id: u64, uri: Uri) -> Result<(), String> {
      self.command_tx
          .try_send(LspCommand::DocumentSymbols { request_id, uri })
          .map_err(|_| "Failed to send document_symbols command".to_string())
  }
  ```

- [ ] **Step 6: Add AsyncMessage::LspDocumentSymbols**

  In `async_bridge.rs`, add after `LspFoldingRanges { ... }` (~line 150):

  ```rust
  /// LSP document symbols response (textDocument/documentSymbol)
  LspDocumentSymbols {
      request_id: u64,
      uri: String,
      symbols: Vec<lsp_types::DocumentSymbol>,
  },
  ```

- [ ] **Step 7: Add document_symbols_supported() to LspManager**

  In `manager.rs`, add after `folding_ranges_supported()` (~line 481):

  ```rust
  /// Check if any eligible server for the language supports document symbols.
  pub fn document_symbols_supported(&self, language: &str) -> bool {
      self.get_handles(language).iter().any(|sh| {
          sh.feature_filter.allows(LspFeature::DocumentSymbols)
              && sh.capabilities.document_symbols
      })
  }
  ```

- [ ] **Step 8: Dispatch LspDocumentSymbols in async_dispatch.rs**

  In `async_dispatch.rs`, add after the `AsyncMessage::LspFoldingRanges { ... }` arm (~line 261):

  ```rust
  AsyncMessage::LspDocumentSymbols {
      request_id,
      uri,
      symbols,
  } => {
      self.handle_lsp_document_symbols(request_id, uri, symbols);
  }
  ```

- [ ] **Step 9: Implement handle_lsp_document_symbols in async_messages.rs**

  In `async_messages.rs`, add after `handle_lsp_folding_ranges` (~line 392):

  ```rust
  pub(super) fn handle_lsp_document_symbols(
      &mut self,
      request_id: u64,
      _uri: String,
      symbols: Vec<lsp_types::DocumentSymbol>,
  ) {
      // Discard if this isn't the request we're waiting for
      if self.pending_symbol_request_id != Some(request_id) {
          return;
      }
      self.pending_symbol_request_id = None;

      let active = self.active_buffer();
      let flat = crate::input::quick_open::providers::flatten_symbols(symbols);
      self.symbol_cache = Some((active, flat));

      // Refresh the Quick Open suggestions list if still open on "@" prefix
      if let Some(prompt) = &self.prompt {
          if prompt.prompt_type == crate::view::prompt::PromptType::QuickOpen
              && prompt.input.starts_with('@')
          {
              let input = prompt.input.clone();
              self.update_quick_open_suggestions(&input);
          }
      }
  }
  ```

- [ ] **Step 10: Add request_document_symbols_for_active_buffer in lsp_requests.rs**

  In `lsp_requests.rs`, add after `request_folding_ranges_for_buffer` (~line 3224):

  ```rust
  /// Fire a textDocument/documentSymbol request for the active buffer.
  /// Stores the request_id in `pending_symbol_request_id` so the response
  /// can be matched. No-op if LSP is not configured or doesn't support the feature.
  pub(crate) fn request_document_symbols_for_active_buffer(&mut self) {
      let buffer_id = self.active_buffer();

      let Some(metadata) = self.buffer_metadata.get(&buffer_id) else {
          return;
      };
      if !metadata.lsp_enabled {
          return;
      }
      let Some(uri) = metadata.file_uri().cloned() else {
          return;
      };
      let file_path = metadata.file_path().cloned();

      let Some(language) = self.buffers.get(&buffer_id).map(|s| s.language.clone()) else {
          return;
      };

      let Some(lsp) = self.lsp.as_mut() else {
          return;
      };

      if !lsp.document_symbols_supported(&language) {
          return;
      }

      use crate::services::lsp::manager::LspSpawnResult;
      if lsp.try_spawn(&language, file_path.as_deref()) != LspSpawnResult::Spawned {
          return;
      }

      let Some(sh) = lsp.handle_for_feature_mut(&language, crate::types::LspFeature::DocumentSymbols) else {
          return;
      };
      let handle = &mut sh.handle;

      let request_id = self.next_lsp_request_id;
      self.next_lsp_request_id += 1;

      match handle.document_symbols(request_id, uri.as_uri().clone()) {
          Ok(()) => {
              self.pending_symbol_request_id = Some(request_id);
          }
          Err(e) => {
              tracing::debug!("Failed to request document symbols: {}", e);
          }
      }
  }
  ```

- [ ] **Step 11: Cargo check**

  ```bash
  cargo check --package fresh-editor 2>&1 | head -30
  ```

  Expected: no errors.

- [ ] **Step 12: Commit**

  ```bash
  git add crates/fresh-editor/src/services/lsp/async_handler.rs \
          crates/fresh-editor/src/services/async_bridge.rs \
          crates/fresh-editor/src/services/lsp/manager.rs \
          crates/fresh-editor/src/app/async_dispatch.rs \
          crates/fresh-editor/src/app/async_messages.rs \
          crates/fresh-editor/src/app/lsp_requests.rs
  git commit -m "feat(symbols): add LSP document symbols request/response pipeline"
  ```

---

### Task 5: Action wiring, command, locales

**Files:**
- Modify: `crates/fresh-editor/src/input/keybindings.rs`
- Modify: `crates/fresh-editor/src/input/commands.rs`
- Modify: `crates/fresh-editor/locales/*.json` (all 14 files)
- Modify: `crates/fresh-editor/src/app/input.rs`

- [ ] **Step 1: Add QuickOpenSymbols to Action enum**

  In `keybindings.rs`, add after `QuickOpenFiles` (~line 480):

  ```rust
  /// Quick Open - symbols in current file (prefix: "@")
  QuickOpenSymbols,
  ```

- [ ] **Step 2: Add string serialization**

  In `keybindings.rs`, in the `from_str` / deserialization match (~line 968), add after `"quick_open_files" => QuickOpenFiles,`:

  ```rust
  "quick_open_symbols" => QuickOpenSymbols,
  ```

- [ ] **Step 3: Add to is_terminal_ui_action**

  In `keybindings.rs`, in the `is_terminal_ui_action` match (~line 1640), add `Action::QuickOpenSymbols` alongside the other QuickOpen variants.

- [ ] **Step 4: Add display name**

  In `keybindings.rs`, in `to_display_name()` (~line 2277), add after `Action::QuickOpenFiles => t!("action.quick_open_files"),`:

  ```rust
  Action::QuickOpenSymbols => t!("action.quick_open_symbols"),
  ```

- [ ] **Step 5: Add default keybinding**

  In `keybindings.rs`, in the default bindings section, add a `Ctrl+Shift+O` binding for `Normal` context:

  ```rust
  // Ctrl+Shift+O — Go to Symbol in File
  KeyBinding {
      key: KeyEvent::new(KeyCode::Char('O'), KeyModifiers::CONTROL | KeyModifiers::SHIFT),
      context: KeyContext::Normal,
      action: Action::QuickOpenSymbols,
  },
  ```

  (Find the nearby Ctrl+Shift+P or similar binding location for exact placement.)

- [ ] **Step 6: Add command definition**

  In `commands.rs`, add after the `QuickOpenFiles` CommandDef (~line 263):

  ```rust
  CommandDef {
      name_key: "cmd.quick_open_symbols",
      desc_key: "cmd.quick_open_symbols_desc",
      action: || Action::QuickOpenSymbols,
      contexts: &[],
      custom_contexts: &[],
  },
  ```

- [ ] **Step 7: Add locale keys to all 14 locale files**

  In each of `cs.json`, `de.json`, `en.json`, `es.json`, `fr.json`, `it.json`, `ja.json`, `ko.json`, `pt-BR.json`, `ru.json`, `th.json`, `uk.json`, `vi.json`, `zh-CN.json`:

  Add alongside the nearby `quick_open_files` keys:
  ```json
  "action.quick_open_symbols": "Go to Symbol in File",
  "cmd.quick_open_symbols": "Go to Symbol in File",
  "cmd.quick_open_symbols_desc": "Go to a symbol in the current file",
  ```

- [ ] **Step 8: Dispatch the action in input.rs**

  In `app/input.rs`, add after the `Action::QuickOpenFiles` arm (~line 979):

  ```rust
  Action::QuickOpenSymbols => {
      if let Some(prompt) = &self.prompt {
          if prompt.prompt_type == PromptType::QuickOpen {
              self.cancel_prompt();
              return Ok(());
          }
      }
      self.start_quick_open_with_prefix("@");
  }
  ```

- [ ] **Step 9: Cargo check**

  ```bash
  cargo check --package fresh-editor 2>&1 | head -30
  ```

  Expected: no errors.

- [ ] **Step 10: Commit**

  ```bash
  git add crates/fresh-editor/src/input/keybindings.rs \
          crates/fresh-editor/src/input/commands.rs \
          crates/fresh-editor/locales/ \
          crates/fresh-editor/src/app/input.rs
  git commit -m "feat(symbols): add QuickOpenSymbols action, command, keybinding, and locales"
  ```

---

### Task 6: Quick open trigger, preview, and confirm

**Files:**
- Modify: `crates/fresh-editor/src/app/prompt_lifecycle.rs`
- Modify: `crates/fresh-editor/src/view/prompt_input.rs`
- Modify: `crates/fresh-editor/src/app/input_dispatch.rs`
- Modify: `crates/fresh-editor/src/app/prompt_actions.rs`

- [ ] **Step 1: Fire LSP request when "@" quick open opens**

  In `prompt_lifecycle.rs`, in `start_quick_open_with_prefix()` (~line 152), add after `self.update_quick_open_suggestions(prefix);`:

  ```rust
  if prefix == "@" {
      self.symbol_cache = None;
      self.request_document_symbols_for_active_buffer();
  }
  ```

- [ ] **Step 2: Add preview_symbol_position method**

  In `prompt_lifecycle.rs`, add after `apply_goto_line_preview` (~line 268):

  ```rust
  /// Preview a symbol's location while navigating the "@" quick open list.
  /// Uses the same goto_line_preview snapshot/restore mechanism as GotoLineProvider.
  /// `line` and `col` are LSP 0-indexed; converted to 1-indexed for goto_line_col.
  pub(super) fn preview_symbol_position(&mut self, line: u32, col: u32) {
      self.save_goto_line_preview_snapshot();
      self.goto_line_col(line as usize + 1, Some(col as usize + 1));
      let new_position = self.active_cursors().primary().position;
      if let Some(snap) = self.goto_line_preview.as_mut() {
          snap.last_jump_position = new_position;
      }
  }
  ```

- [ ] **Step 3: Emit PromptSelectionChanged for QuickOpen in prompt_input.rs**

  In `view/prompt_input.rs`, in the `KeyCode::Up` branch where `selected_suggestion` is updated, add alongside the existing Plugin emit (~line 168):

  ```rust
  if matches!(
      self.prompt_type,
      crate::view::prompt::PromptType::QuickOpen
  ) {
      ctx.defer(DeferredAction::PromptSelectionChanged {
          selected_index: new_selected,
      });
  }
  ```

  Do the same in the `KeyCode::Down` branch (~line 212).

- [ ] **Step 4: Handle QuickOpen selection change for symbol preview in input_dispatch.rs**

  In `input_dispatch.rs`, inside the `DeferredAction::PromptSelectionChanged { selected_index }` arm (~line 308), add a new branch for QuickOpen after the existing Plugin branch:

  ```rust
  if let Some(prompt) = &self.prompt {
      let prompt_type = prompt.prompt_type.clone();
      let input = prompt.input.clone();
      let suggestion_value = prompt
          .suggestions
          .get(selected_index)
          .and_then(|s| s.value.clone());

      match prompt_type {
          crate::view::prompt::PromptType::Plugin { custom_type } => {
              self.plugin_manager.run_hook(
                  "prompt_selection_changed",
                  crate::services::plugins::hooks::HookArgs::PromptSelectionChanged {
                      prompt_type: custom_type,
                      selected_index,
                  },
              );
          }
          crate::view::prompt::PromptType::QuickOpen if input.starts_with('@') => {
              if let Some(value) = suggestion_value {
                  let parts: Vec<&str> = value.splitn(4, ':').collect();
                  if parts.len() == 4 {
                      if let (Ok(line), Ok(col)) =
                          (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                      {
                          self.preview_symbol_position(line, col);
                      }
                  }
              }
          }
          _ => {}
      }
  }
  ```

  (Replace the existing body of `PromptSelectionChanged` with this unified match.)

- [ ] **Step 5: Handle GotoSymbol result in prompt_actions.rs**

  In `prompt_actions.rs`, in `execute_quick_open_result()`, add a `GotoSymbol` arm in the `match &result` block that checks the preview (~line 1476), alongside `GotoLine`:

  ```rust
  QuickOpenResult::GotoSymbol { .. } => {
      // Commit: discard snapshot without restoring (cursor is already previewing the target)
      self.goto_line_preview = None;
  }
  ```

  Then in the second `match result` (~line 1486), add after the `GotoLine` arm:

  ```rust
  QuickOpenResult::GotoSymbol { start_line, start_char, end_line, end_char } => {
      // start_line etc. are LSP 0-indexed; select_range takes 1-indexed
      self.select_range(
          start_line as usize + 1,
          Some(start_char as usize + 1),
          end_line as usize + 1,
          Some(end_char as usize + 1),
      );
      PromptResult::Done
  }
  ```

- [ ] **Step 6: Cargo check**

  ```bash
  cargo check --package fresh-editor 2>&1 | head -30
  ```

  Expected: no errors.

- [ ] **Step 7: Commit**

  ```bash
  git add crates/fresh-editor/src/app/prompt_lifecycle.rs \
          crates/fresh-editor/src/view/prompt_input.rs \
          crates/fresh-editor/src/app/input_dispatch.rs \
          crates/fresh-editor/src/app/prompt_actions.rs
  git commit -m "feat(symbols): wire up trigger, preview on navigation, and GotoSymbol confirm"
  ```

---

### Task 7: Register SymbolProvider + cancel cleanup

**Files:**
- Modify: `crates/fresh-editor/src/app/editor_init.rs`
- Modify: `crates/fresh-editor/src/app/prompt_lifecycle.rs`

- [ ] **Step 1: Register SymbolProvider in editor_init.rs**

  In `editor_init.rs`, where the other providers are registered, add:

  ```rust
  quick_open_registry.register(Box::new(SymbolProvider::new()));
  ```

  Add the import at the top of the file if needed:
  ```rust
  use crate::input::quick_open::SymbolProvider;
  ```

- [ ] **Step 2: Clear symbol_cache on QuickOpen cancel**

  In `prompt_lifecycle.rs`, in the `cancel_prompt` / `PromptType::QuickOpen` cancel branch (~line 734), add after `self.restore_goto_line_preview_snapshot();`:

  ```rust
  // Clear symbol cache so a stale result isn't shown on next open
  self.symbol_cache = None;
  self.pending_symbol_request_id = None;
  ```

- [ ] **Step 3: Full cargo check**

  ```bash
  cargo check --all-targets 2>&1 | head -30
  ```

  Expected: no errors.

- [ ] **Step 4: Run all tests**

  ```bash
  cargo test --package fresh-editor 2>&1 | tail -20
  ```

  Expected: all tests pass.

- [ ] **Step 5: Commit**

  ```bash
  git add crates/fresh-editor/src/app/editor_init.rs \
          crates/fresh-editor/src/app/prompt_lifecycle.rs
  git commit -m "feat(symbols): register SymbolProvider and clean up cache on cancel"
  ```

---

### Task 8: FakeLspServer extension + E2E test

**Files:**
- Modify: `crates/fresh-editor/tests/common/fake_lsp.rs`
- Create: `crates/fresh-editor/tests/e2e/lsp_document_symbols.rs`
- Modify: `crates/fresh-editor/tests/e2e_tests.rs` (or wherever e2e modules are declared)

- [ ] **Step 1: Add spawn_with_document_symbols to FakeLspServer**

  In `tests/common/fake_lsp.rs`, add after `spawn_with_inlay_hints`:

  ```rust
  /// Spawn a fake LSP server that responds to textDocument/documentSymbol.
  /// Returns two symbols: a top-level class `MyClass` (lines 0–9) containing
  /// a method `my_method` (lines 2–5).
  pub fn spawn_with_document_symbols(dir: &std::path::Path) -> anyhow::Result<Self> {
      let (stop_tx, stop_rx) = mpsc::channel();

      let script = r#"#!/bin/bash
  read_message() {
      local content_length=0
      while IFS= read -r line; do
          line="${line%$'\r'}"
          [ -z "$line" ] && break
          case "$line" in
              Content-Length:*) content_length="${line#Content-Length: }" ;;
          esac
      done
      [ "$content_length" -gt 0 ] 2>/dev/null && dd bs=1 count="$content_length" 2>/dev/null
  }
  send_message() {
      local msg="$1"
      printf "Content-Length: %d\r\n\r\n%s" "${#msg}" "$msg"
  }
  while true; do
      msg=$(read_message)
      [ -z "$msg" ] && break
      method=$(echo "$msg" | grep -o '"method":"[^"]*"' | cut -d'"' -f4)
      msg_id=$(echo "$msg" | grep -o '"id":[0-9]*' | cut -d':' -f2)
      case "$method" in
          "initialize")
              send_message '{"jsonrpc":"2.0","id":'$msg_id',"result":{"capabilities":{"textDocumentSync":1,"documentSymbolProvider":true}}}'
              ;;
          "initialized"|"textDocument/didOpen"|"textDocument/didChange"|"textDocument/didSave")
              ;;
          "textDocument/documentSymbol")
              send_message '{"jsonrpc":"2.0","id":'$msg_id',"result":[{"name":"MyClass","kind":5,"range":{"start":{"line":0,"character":0},"end":{"line":9,"character":1}},"selectionRange":{"start":{"line":0,"character":6},"end":{"line":0,"character":13}},"children":[{"name":"my_method","kind":6,"range":{"start":{"line":2,"character":4},"end":{"line":5,"character":5}},"selectionRange":{"start":{"line":2,"character":8},"end":{"line":2,"character":17}},"children":[]}]}]}'
              ;;
          "shutdown")
              send_message '{"jsonrpc":"2.0","id":'$msg_id',"result":null}'
              break
              ;;
      esac
  done
  "#;

      let script_path = dir.join("fake_lsp_document_symbols.sh");
      std::fs::write(&script_path, script)?;
      #[cfg(unix)]
      {
          use std::os::unix::fs::PermissionsExt;
          let mut perms = std::fs::metadata(&script_path)?.permissions();
          perms.set_mode(0o755);
          std::fs::set_permissions(&script_path, perms)?;
      }

      let handle = Some(thread::spawn(move || { let _ = stop_rx.recv(); }));
      Ok(Self { handle, stop_tx })
  }

  pub fn document_symbols_script_path(dir: &std::path::Path) -> std::path::PathBuf {
      dir.join("fake_lsp_document_symbols.sh")
  }
  ```

- [ ] **Step 2: Create the E2E test file**

  Create `crates/fresh-editor/tests/e2e/lsp_document_symbols.rs`:

  ```rust
  use crate::common::fake_lsp::FakeLspServer;
  use crate::common::harness::EditorTestHarness;
  use crossterm::event::{KeyCode, KeyModifiers};

  #[test]
  #[cfg_attr(target_os = "windows", ignore = "FakeLspServer uses Bash")]
  fn test_quick_open_symbols_shows_list() -> anyhow::Result<()> {
      let temp_dir = tempfile::tempdir()?;
      let _fake_server = FakeLspServer::spawn_with_document_symbols(temp_dir.path())?;

      let test_file = temp_dir.path().join("test.rs");
      // 10 lines so symbol ranges are valid
      std::fs::write(&test_file, "class MyClass {\n\n    fn my_method() {\n        ()\n    }\n\n}\n\n\n\n")?;

      let mut config = fresh::config::Config::default();
      config.lsp.insert(
          "rust".to_string(),
          fresh::types::LspLanguageConfig::Multi(vec![fresh::services::lsp::LspServerConfig {
              command: FakeLspServer::document_symbols_script_path(temp_dir.path())
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
              name: Some("fake-symbols-ls".to_string()),
              only_features: None,
              except_features: None,
          }]),
      );

      let mut harness = EditorTestHarness::create(
          80,
          24,
          crate::common::harness::HarnessOptions::new()
              .with_config(config)
              .with_working_dir(temp_dir.path().to_path_buf()),
      )?;

      harness.open_file(&test_file)?;
      harness.render()?;

      // Wait for LSP to initialize
      harness.wait_until(|h| h.editor().initialized_lsp_server_count("rust") > 0)?;

      // Trigger QuickOpenSymbols via action
      harness.send_action(fresh::input::keybindings::Action::QuickOpenSymbols)?;
      harness.render()?;

      // Wait for symbols to load (LSP responds asynchronously)
      harness.wait_until(|h| {
          h.screen_contains("[class]") || h.screen_contains("[method]")
      })?;

      harness.render()?;
      harness.assert_screen_contains("[class] MyClass");
      harness.assert_screen_contains("[method] my_method");

      // Navigate down to my_method (it's the second item)
      harness.send_key(KeyCode::Down, KeyModifiers::NONE)?;
      harness.render()?;

      // Confirm: should select the full range of my_method
      harness.send_key(KeyCode::Enter, KeyModifiers::NONE)?;
      harness.render()?;

      // Cursor should be at line 3 (1-indexed), which is my_method's start line (LSP line 2 + 1)
      harness.assert_cursor_line(3);

      Ok(())
  }

  #[test]
  #[cfg_attr(target_os = "windows", ignore = "FakeLspServer uses Bash")]
  fn test_quick_open_symbols_escape_restores_cursor() -> anyhow::Result<()> {
      let temp_dir = tempfile::tempdir()?;
      let _fake_server = FakeLspServer::spawn_with_document_symbols(temp_dir.path())?;

      let test_file = temp_dir.path().join("test2.rs");
      std::fs::write(&test_file, "class MyClass {\n\n    fn my_method() {}\n}\n")?;

      let mut config = fresh::config::Config::default();
      config.lsp.insert(
          "rust".to_string(),
          fresh::types::LspLanguageConfig::Multi(vec![fresh::services::lsp::LspServerConfig {
              command: FakeLspServer::document_symbols_script_path(temp_dir.path())
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
              name: Some("fake-symbols-ls2".to_string()),
              only_features: None,
              except_features: None,
          }]),
      );

      let mut harness = EditorTestHarness::create(
          80,
          24,
          crate::common::harness::HarnessOptions::new()
              .with_config(config)
              .with_working_dir(temp_dir.path().to_path_buf()),
      )?;

      harness.open_file(&test_file)?;
      harness.render()?;
      harness.wait_until(|h| h.editor().initialized_lsp_server_count("rust") > 0)?;

      // Record starting cursor position (line 1)
      let start_line = harness.cursor_line();

      // Open symbols, wait for list
      harness.send_action(fresh::input::keybindings::Action::QuickOpenSymbols)?;
      harness.render()?;
      harness.wait_until(|h| h.screen_contains("[class]"))?;

      // Navigate down — cursor should preview my_method location
      harness.send_key(KeyCode::Down, KeyModifiers::NONE)?;
      harness.render()?;

      // Escape — cursor should return to original position
      harness.send_key(KeyCode::Esc, KeyModifiers::NONE)?;
      harness.render()?;

      assert_eq!(harness.cursor_line(), start_line, "cursor should be restored after Escape");

      Ok(())
  }
  ```

- [ ] **Step 3: Register the test module**

  In the e2e test entry point (check `tests/e2e_tests.rs` or a similar file that `mod`-includes other e2e files), add:

  ```rust
  mod lsp_document_symbols;
  ```

- [ ] **Step 4: Run the E2E tests**

  ```bash
  cargo test --package fresh-editor --test e2e_tests lsp_document_symbols -- --nocapture
  ```

  Expected: both tests pass.

- [ ] **Step 5: Run the full test suite**

  ```bash
  cargo test --package fresh-editor 2>&1 | tail -20
  ```

  Expected: no regressions.

- [ ] **Step 6: Commit**

  ```bash
  git add crates/fresh-editor/tests/common/fake_lsp.rs \
          crates/fresh-editor/tests/e2e/lsp_document_symbols.rs \
          crates/fresh-editor/tests/e2e_tests.rs
  git commit -m "test(symbols): add e2e tests for Go to Symbol in File"
  ```
