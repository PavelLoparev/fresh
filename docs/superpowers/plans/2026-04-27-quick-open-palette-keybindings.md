# Quick Open Palette Keybindings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Ctrl+Tab and Shift+Tab keybindings for Quick Open with buffer and file modes.

**Architecture:** Add two new actions (QuickOpenBuffers, QuickOpenFiles) that open Quick Open with different prefixes. Extend existing start_quick_open to accept a prefix parameter.

**Tech Stack:** Rust, crossterm key events

---

## Task 1: Add New Action Variants

**Files:**
- Modify: `crates/fresh-editor/src/input/keybindings.rs:463-466`

- [ ] **Step 1: Add action variants**

After line 465 (`QuickOpen,`), add:

```rust
    /// Quick Open - buffers (prefix: "#")
    QuickOpenBuffers,
    /// Quick Open - files (empty prefix)
    QuickOpenFiles,
```

- [ ] **Step 2: Add FromStr parsing**

Find the parse implementation (search for `"quick_open" => QuickOpen`), around line 920. Add:

```rust
            "quick_open_buffers" => QuickOpenBuffers,
            "quick_open_files" => QuickOpenFiles,
```

- [ ] **Step 3: Add Display formatting**

Find where `Action::QuickOpen` is formatted (search for `Action::QuickOpen =>`), around line 2207. Add:

```rust
            Action::QuickOpenBuffers => t!("action.quick_open_buffers"),
            Action::QuickOpenFiles => t!("action.quick_open_files"),
```

- [ ] **Step 4: Commit**

```bash
git add crates/fresh-editor/src/input/keybindings.rs
git commit -m "feat: add QuickOpenBuffers and QuickOpenFiles actions"
```

---

## Task 2: Add Handler in app/input.rs

**Files:**
- Modify: `crates/fresh-editor/src/app/input.rs:571-582`
- Modify: `crates/fresh-editor/src/app/prompt_lifecycle.rs:137-156`

- [ ] **Step 1: Add helper method for prefix-based quick open**

In `prompt_lifecycle.rs` after `start_quick_open`, add:

```rust
    /// Start Quick Open prompt with specified prefix
    pub fn start_quick_open_with_prefix(&mut self, prefix: &str) {
        self.on_editor_focus_lost();
        self.status_message = None;
        self.goto_line_preview = None;

        let mut prompt = Prompt::with_suggestions(String::new(), PromptType::QuickOpen, vec![]);
        prompt.input = prefix.to_string();
        prompt.cursor_pos = prefix.len();
        self.prompt = Some(prompt);

        self.update_quick_open_suggestions(prefix);
    }
```

- [ ] **Step 2: Add action handlers in input.rs**

Find `Action::QuickOpen =>` handler in input.rs (line ~571). Add handlers for new actions after existing QuickOpen handler:

```rust
            Action::QuickOpenBuffers => {
                if let Some(prompt) = &self.prompt {
                    if prompt.prompt_type == PromptType::QuickOpen {
                        self.cancel_prompt();
                        return Ok(());
                    }
                }
                self.start_quick_open_with_prefix("#");
            }
            Action::QuickOpenFiles => {
                if let Some(prompt) = &self.prompt {
                    if prompt.prompt_type == PromptType::QuickOpen {
                        self.cancel_prompt();
                        return Ok(());
                    }
                }
                self.start_quick_open_with_prefix("");
            }
```

- [ ] **Step 3: Commit**

```bash
git add crates/fresh-editor/src/app/input.rs crates/fresh-editor/src/app/prompt_lifecycle.rs
git commit -m "feat: handle QuickOpenBuffers and QuickOpenFiles actions"
```

---

## Task 3: Add Keybindings in default.json

**Files:**
- Modify: `crates/fresh-editor/keymaps/default.json`

- [ ] **Step 1: Add keybindings**

Find the existing QuickOpen binding (lines 4-11). Add new bindings after it:

```json
    {
      "comment": "Quick Open files - find files in project",
      "key": "Tab",
      "modifiers": ["ctrl"],
      "action": "quick_open_files",
      "args": {},
      "when": "global"
    },
    {
      "comment": "Quick Open buffers - switch between open tabs",
      "key": "Tab",
      "modifiers": ["shift"],
      "action": "quick_open_buffers",
      "args": {},
      "when": "global"
    },
```

- [ ] **Step 2: Commit**

```bash
git add crates/fresh-editor/keymaps/default.json
git commit -m "feat: add Ctrl+Tab and Shift+Tab keybindings for Quick Open"
```

---

## Task 4: Add Translations (if needed)

**Files:**
- Check: `crates/fresh-editor/lang/en.toml`

- [ ] **Step 1: Check for existing translation keys**

Search for `action.quick_open` in lang files. If keys need to be added, add:
- `action.quick_open_buffers = "Quick Open: Switch to Buffer"`
- `action.quick_open_files = "Quick Open: Find Files"`

- [ ] **Step 2: Commit (if changes needed)**

```bash
git add crates/fresh-editor/lang/
git commit -m "i18n: add translations for Quick Open buffer/file actions"
```

---

## Task 5: Verification

**Files:**
- Test: `cargo test -p fresh-editor`

- [ ] **Step 1: Run verification commands**

Per AGENTS.md:
```bash
cargo check --all-targets
cargo fmt -- --check
cargo clippy --all-targets
```

- [ ] **Step 2: Run relevant tests**

```bash
cargo test -p fresh-editor -- quick_open
```

- [ ] **Step 3: Commit with verification**

```bash
git commit -m "feat: add Quick Open palette keybindings - implementation complete"
```

---

## Execution

**Plan complete and saved to `docs/superpowers/plans/2026-04-27-quick-open-palette-keybindings.md`. Two execution options:**

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**