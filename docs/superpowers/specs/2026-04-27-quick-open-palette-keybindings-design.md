# Quick Open Palette Keybindings Design

Date: 2026-04-27

## Summary

Add dedicated keybindings for Quick Open palette variants:
- `Ctrl+Tab` → files palette (empty prefix)
- `Shift+Tab` → buffers palette (`#` prefix)

Existing `Ctrl+P` → command palette (`>` prefix) remains unchanged.

## Background

The Quick Open system supports multiple providers via prefix routing:
- `>` → Command palette
- `#` → Buffer switcher
- `:` → Go to line
- (empty) → File finder

Currently, `Ctrl+P` opens with `>` prefix (command palette). Users wanting buffers must type `#` manually, or erase to get files.

## Design

### Actions

Add two new actions in `Action` enum:

```rust
QuickOpenBuffers, // Opens Quick Open with "#" prefix
QuickOpenFiles,  // Opens Quick Open with empty prefix
```

### Handler Logic

In `app/input.rs`, extend the `QuickOpen` match arm:

```rust
Action::QuickOpenBuffers => {
    // Toggle: close if already open
    if let Some(prompt) = &self.prompt {
        if prompt.prompt_type == PromptType::QuickOpen {
            self.cancel_prompt();
            return Ok(());
        }
    }
    // Start with "#" prefix
    self.start_quick_open_with_prefix("#");
}
Action::QuickOpenFiles => {
    if let Some(prompt) = &self.prompt {
        if prompt.prompt_type == PromptType::QuickOpen {
            self.cancel_prompt();
            return Ok(());
        }
    }
    // Start with empty prefix
    self.start_quick_open_with_prefix("");
}
```

### New Method

Add `start_quick_open_with_prefix(prefix: &str)` in `prompt_lifecycle.rs`:

```rust
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

### Keybindings

Add to `keymaps/default.json`:

```json
{
  "key": "Tab",
  "modifiers": ["ctrl"],
  "action": "quick_open_files",
  "when": "global"
},
{
  "key": "Tab",
  "modifiers": ["shift"],
  "action": "quick_open_buffers",
  "when": "global"
}
```

### i18n

Add translation keys:
- `action.quick_open_buffers`
- `action.quick_open_files`

## Files to Modify

1. `crates/fresh-editor/src/input/keybindings.rs` — Add action variants
2. `crates/fresh-editor/src/app/input.rs` — Add action handlers
3. `crates/fresh-editor/src/app/prompt_lifecycle.rs` — Add helper method
4. `crates/fresh-editor/keymaps/default.json` — Add keybindings
5. `crates/fresh-editor/lang/en.toml` (or locale files) — Add translations

## Testing

- Verify `Shift+P` opens buffer switcher
- Verify `Shift+Shift` opens file finder
- Verify existing `Ctrl+P` behavior unchanged
- Test toggle behavior (pressing again closes prompt)