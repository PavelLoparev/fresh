# Go to Symbol in File

**Date:** 2026-05-01  
**Status:** Approved

## Summary

Add a "Go to Symbol in File" quick-open provider triggered by the `@` prefix (and the `QuickOpenSymbols` action), backed by the LSP `textDocument/documentSymbol` request. Symbols are shown as a hierarchical, indented list. Navigating the list previews the symbol location; confirming selects the full body range.

---

## 1. Data Model

A new `FlatSymbol` struct (in `crates/fresh-editor/src/input/quick_open/providers.rs` or a dedicated `symbols.rs` alongside it):

```rust
pub struct FlatSymbol {
    pub name: String,
    pub kind: lsp_types::SymbolKind,
    pub depth: u8,        // 0 = top-level, capped at 4 for display sanity
    pub start_line: u32,
    pub start_char: u32,
    pub end_line: u32,    // full body range — used for selection on confirm
    pub end_char: u32,
}
```

A `flatten_symbols(symbols: Vec<DocumentSymbol>) -> Vec<FlatSymbol>` helper recurses depth-first through the `DocumentSymbol` tree, incrementing depth at each level and capping at 4. This preserves document order (parent before children).

`QuickOpenContext` gains one new field:

```rust
pub document_symbols: Vec<FlatSymbol>,  // empty until LSP responds
```

The editor builds this from `symbol_cache` when constructing context. It is an empty `Vec` (not `Option`) so providers can always safely iterate it.

---

## 2. LSP Request/Response Pipeline

Five additions to the existing async pattern, following the `FoldingRange` / `InlayHints` precedent:

| Step | Location | Change |
|------|----------|--------|
| 1 | `services/lsp/async_handler.rs` — `LspCommand` enum | Add `DocumentSymbols { request_id: u64, uri: Uri }` |
| 2 | `async_handler.rs` — `run()` dispatch loop | Handle `DocumentSymbols`: call `textDocument/documentSymbol`, send `AsyncMessage::LspDocumentSymbols` |
| 3 | `services/async_bridge.rs` — `AsyncMessage` enum | Add `LspDocumentSymbols { request_id: u64, uri: Uri, symbols: Vec<lsp_types::DocumentSymbol> }` |
| 4 | `app/async_dispatch.rs` | Dispatch `LspDocumentSymbols` → `handle_lsp_document_symbols()` |
| 5 | `app/async_messages.rs` | Implement `handle_lsp_document_symbols()`: flatten tree → store in `editor.symbol_cache: Option<(BufferId, Vec<FlatSymbol>)>` → call `update_quick_open_suggestions()` |

Cache key is `BufferId`. A result belonging to a stale buffer (user switched away) is discarded in the handler before storing.

If the LSP server does not support `documentSymbol` (checked via `manager.capabilities.document_symbols`), the request is skipped entirely.

---

## 3. SymbolProvider

A new `SymbolProvider` struct implementing `QuickOpenProvider` with `prefix() -> "@"`.

**`suggestions(query, ctx)`**

- If `ctx.document_symbols` is empty AND `ctx.has_lsp_config` is true → return one disabled entry: `"Loading…"`
- If `ctx.document_symbols` is empty AND `ctx.has_lsp_config` is false → return one disabled entry: `"No language server available"`
- Otherwise: fuzzy-filter entries by name against `query` (empty query = show all), format each as:

  ```
  {indent}[{kind_label}] {name}
  ```

  where `indent` = `" ".repeat(symbol.depth * 2)` and `kind_label` is a short lowercase string derived from `SymbolKind` (e.g. `class`, `method`, `function`, `variable`, `interface`, `enum`, `field`, `module`, `const`, …). Results are returned in document order (preserving parent-before-child hierarchy). When `query` is non-empty, matched entries are shown with their parents above them (parents shown even if they don't match, greyed out) — or simpler: show only matched entries without re-injecting parents.

  **Decision:** show only matched entries (no parent re-injection). Simpler, consistent with how BufferProvider and FileProvider work.

**`on_select(suggestion, query, ctx)`**

Each suggestion carries its `FlatSymbol` data encoded in `Suggestion::value` as `"{start_line}:{start_char}:{end_line}:{end_char}"`. `on_select` parses this and returns a new `QuickOpenResult` variant:

```rust
QuickOpenResult::GotoSymbol {
    start_line: usize,
    start_char: usize,
    end_line: usize,
    end_char: usize,
}
```

`prompt_lifecycle.rs` handles `GotoSymbol`: jump cursor to `start_line:start_char` and set a selection from start to end of the full range.

**Preview**

While the `@` quick open is active, each navigation keystroke (up/down) calls a new `preview_symbol_position(start_line, start_char)` method in `prompt_lifecycle.rs`. This follows the same snapshot/restore pattern as `GotoLineProvider`:
- On first `@` open: save a viewport + cursor snapshot
- On navigation: scroll/move cursor to the symbol's start line without committing
- On Escape: restore snapshot
- On confirm: commit the jump and set the selection range

**Registration**

`SymbolProvider::new()` takes no arguments (stateless). Registered in `editor_init.rs` alongside the other providers:

```rust
quick_open_registry.register(Box::new(SymbolProvider::new()));
```

---

## 4. Action & Command Wiring

| Item | Detail |
|------|--------|
| `Action` variant | `QuickOpenSymbols` — serializes as `"quick_open_symbols"` |
| Trigger | `start_quick_open_with_prefix("@")` + immediately send `LspCommand::DocumentSymbols` for active buffer |
| Default keybinding | `Ctrl+Shift+O` |
| Command name key | `cmd.quick_open_symbols` |
| Command description key | `cmd.quick_open_symbols_desc` |
| Action display name | `"Go to Symbol in File"` (and matching `t!()` key) |
| Locale files | Add keys to **all** locale files under `crates/fresh-editor/locales/` with real translations |

The action appears in the `>` command palette. `Action::QuickOpenSymbols` is grouped with `QuickOpen`, `QuickOpenBuffers`, `QuickOpenFiles` in the serializer and display-name match arms.

---

## 5. Edge Cases & Error Handling

| Scenario | Behavior |
|----------|----------|
| No LSP configured | Show disabled `"No language server available"` entry |
| LSP server doesn't support `documentSymbol` | Skip request; show disabled entry |
| LSP request fails / returns error | `symbol_cache` stays `None`; show `"Loading…"` (user can close and retry) |
| Buffer switches while `@` is open | On `handle_lsp_document_symbols`, discard result if `BufferId` doesn't match active buffer; fire new request for new buffer and show `"Loading…"` |
| Empty symbol list from LSP | Show disabled `"No symbols found"` entry |
| User presses Escape | Restore saved snapshot (cursor + viewport), close prompt |

---

## 6. Tests

**Unit tests** (in `tests/` or inline in the provider module):

1. `flatten_symbols` — verify depth-first ordering, depth capping at 4, correct `start/end` range copying from `DocumentSymbol`
2. `SymbolProvider::suggestions` with empty symbols → `"Loading…"` entry
3. `SymbolProvider::suggestions` with no LSP → `"No language server available"` entry
4. `SymbolProvider::suggestions` with symbols + empty query → all symbols, document order, correct indentation strings
5. `SymbolProvider::suggestions` with symbols + non-empty query → only matching names returned

**E2E test** (`tests/e2e/`):

1. Open a file that has a mock/stub LSP providing a known symbol list
2. Trigger `QuickOpenSymbols` action
3. Assert screen shows `"@"` prefix in prompt and formatted symbol list
4. Navigate down — assert cursor preview jumps to symbol location
5. Press Escape — assert cursor restored to original position
6. Re-open, navigate to a symbol, confirm — assert selection covers full range

---

## Files Changed

| File | Change |
|------|--------|
| `src/services/lsp/async_handler.rs` | Add `LspCommand::DocumentSymbols`; handle in `run()` |
| `src/services/async_bridge.rs` | Add `AsyncMessage::LspDocumentSymbols` |
| `src/app/async_dispatch.rs` | Dispatch `LspDocumentSymbols` |
| `src/app/async_messages.rs` | Implement `handle_lsp_document_symbols()` |
| `src/app/mod.rs` | Add `symbol_cache: Option<(BufferId, Vec<FlatSymbol>)>` field to `Editor` struct |
| `src/input/quick_open/mod.rs` | Add `document_symbols: Vec<FlatSymbol>` to `QuickOpenContext` |
| `src/input/quick_open/providers.rs` | Add `FlatSymbol`, `flatten_symbols()`, `SymbolProvider` |
| `src/input/keybindings.rs` | Add `Action::QuickOpenSymbols` variant + serialization |
| `src/input/commands.rs` | Add command def for `QuickOpenSymbols` |
| `src/app/prompt_lifecycle.rs` | Handle `@` trigger, `preview_symbol_position()`, `GotoSymbol` result |
| `src/app/action_dispatch.rs` | Dispatch `Action::QuickOpenSymbols` |
| `locales/*.json` | Add `cmd.quick_open_symbols`, `cmd.quick_open_symbols_desc`, `action.quick_open_symbols` keys |
| `src/app/editor_init.rs` | Register `SymbolProvider` |
