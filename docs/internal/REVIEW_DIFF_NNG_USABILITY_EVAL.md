# Review Diff — NN/g Usability Evaluation

**Date:** 2026-04-13
**Editor:** `fresh` 0.2.23 (debug build, branch `claude/tui-editor-usability-eval-0LHgo`)
**Environment:** tmux 3.4, 160x45 pane, Linux 4.4.0, terminal with 256-color ANSI
**Artifacts:** Screen captures (ANSI + plain) in `/tmp/eval-workspace/screen_*.txt`,
`/tmp/eval-workspace/pass2/p2_*.txt`, and `/tmp/validate/c_*.txt`.

> **Revision note (pass 3 — manual validation).** Sections 7 and 8 below
> contain the corrected, capture-verified findings after a full
> interactive tmux re-run. Any disagreement between earlier sections and
> §7–§8 should be resolved in favour of §7–§8.

---

## 1. Executive Summary

The Review Diff feature is **functional for a first-pass review** but is held
back by a handful of silent-failure defects and discoverability gaps that
add needless cognitive load. A user who already knows the mode's vocabulary
(`Tab`, `n`, `Enter`, `q`, `s`, `u`, `d`) can complete a standard PR-style
review; a newcomer will bounce off at least two soft failures before learning
the layout.

**Overall usability score: 3.1 / 5 (Fair, not NN/g "Good")**
*(revised down from 3.4 after pass 2 uncovered three additional defects:
terminal-resize corruption, silent empty-state, and `NO_COLOR` non-compliance.)*

- Heuristic wins: highly discoverable entry (`Ctrl+P` shown in status bar),
  a permanent action toolbar, a safe exit affordance (`q Close`), a proper
  confirm-dialog on destructive `d`iscard, clean rename (`R old → new`) and
  pure-additions / pure-deletions / Unicode / emoji rendering, and an
  inline-comment treatment (`» [hunk] …`) that persists across close-reopen.
- Heuristic losses: **no "hunk N of M" indicator**, viewport does not follow
  the cursor in side-by-side view, word-level diff highlighting is absent,
  the fuzzy finder only does subsequence matching (no typo tolerance),
  terminal resize corrupts the layout without auto-recovery, empty states
  are ambiguous, and the in-app Manual / menu bar don't know about the
  feature at all.

**Primary roadblocks (ranked):**

1. **H1 — Silent hunk navigation (visibility of system status).** `n`/`p`
   from the unified diff panel advance the cursor by roughly one line each
   press and give no indication of "which hunk am I on." The user cannot
   tell whether the keystroke succeeded.
2. **H1 — Viewport/cursor desync in side-by-side view.** Status bar always
   reads `Ln 1, Col 1` even after the viewport scrolls 170 lines.
3. **H9 — Terminal resize destroys the layout and does not auto-recover.**
   Confirms BUG-2. Toolbar, menu, and tab row stay hidden until the user
   guesses to press `r` (refresh). Nothing in the UI says so.
4. **H1 — Ambiguous empty state.** Both "not a git repository" and "clean
   repo, no changes" render identically (empty `GIT STATUS` pane, `DIFF`
   header with no filename, `Review Diff: 0 hunks` in the status bar).
   i18n keys `status.not_git_repo` and `panel.no_changes` exist but are
   never displayed.
5. **H5 / accessibility — Whitespace-only changes are invisible.** Trailing
   spaces and collapsed/expanded spaces render identically on both sides,
   only the `-`/`+` marker differs. Also: `NO_COLOR=1` is ignored — the
   editor emits the full 256-color palette regardless.
6. **H9 (error recovery) — Typo tolerance in the command palette is
   weak.** "revw difff" returns `Markdown: Toggle Compose/Preview` as the
   best match — not "Review Diff."
7. **H10 — Feature is absent from the in-app Manual (F1) and from every
   top-level menu.** Users who don't already know `Ctrl+P` → "Review Diff"
   have no discoverable path.

---

## 2. Heuristic Evaluation (NN/g, adapted for TUI diff review)

### H1. Visibility of System Status — **Partial Pass**

| Signal | Present? | Evidence |
|--------|----------|----------|
| Current file name in right-pane header | **Yes** | `DIFF FOR happy.py` (screen_04) |
| Selected row marker in files list | **Yes** | `>M  happy.py` (caret glyph) |
| Total hunk count | **Yes** | Status bar: `Review Diff: 14 hunks` |
| Current-hunk index ("hunk 3 of 10") | **No** | Never displayed |
| Cursor line / column | **Yes (unified view)** | `Ln 21, Col 1` |
| Cursor line / column in side-by-side | **BROKEN** | Stuck at `Ln 1, Col 1` even after 170-line scroll (screen_17) |
| Add/del stats per file | **Partial** | Only in side-by-side: `+10 -10 ~10` |
| Context-sensitive toolbar | **Yes** | Toolbar swaps `↵ Open / r Refresh` for `n Next / p Prev` when focus is on diff (screen_04 vs screen_06) |

**Violation — no hunk index.** With 10 hunks in `monolith.txt`, pressing `n`
four times gave no feedback except a cursor line change from 21 → 29. The
user cannot confirm they arrived at the intended hunk.

### H2. User Control and Freedom — **Pass**

- `q` cleanly closes the Review Diff tab and returns to the previous buffer
  (status: `Tab closed`, screen_27).
- `q` also exits the side-by-side view back to the unified view (screen_18).
- `Escape` closes the command palette without side effects
  (status: `Search cancelled`, screen_26).
- `Ctrl+C` at the editor root is swallowed gracefully (no crash, no prompt,
  editor stays alive — verified PID persisted).

**Gap:** there is no `Undo comment` or reopen-last-closed-review affordance
visible in the toolbar. A user who closes the review tab must re-run
`Ctrl+P` → "Review Diff" to get back in.

### H3. Consistency & Standards — **Partial Pass**

- `+`/`-` prefixes are standard (screen_04).
- Color palette matches convention: dark red `bg 256:52` for removed lines,
  dark green `bg 256:22` for added lines, no intrusive foreground recolor.
- **Deviation from `git diff --unified` standard:** the hunk header is
  rendered as `@@ line_0047 = value_47  # comment for line 47 @@` — the
  post-context line contents — instead of the standard
  `@@ -47,7 +47,7 @@ <function_sig>`. This aids casual readers but breaks
  muscle memory for anyone used to `git diff` / GitHub / `vimdiff`. The
  missing `-start,count +start,count` numbers make it impossible to count
  added/removed lines per hunk at a glance.
- Standard review-mode keys (`s`/`u`/`d`) match `git add -p` / lazygit
  conventions.
- `N` (capital) for "Note" is inconsistent with lowercase action keys
  elsewhere on the same toolbar — users will try lowercase `n` first and
  trigger hunk-nav instead.

### H4. Flexibility & Efficiency of Use — **Partial Pass**

Minimum keystrokes to enter Review Diff from editing:

| Path | Keystrokes | Notes |
|------|------------|-------|
| `Ctrl+P` → `revie` → `Enter` | **7 key events** | Default fuzzy result is "Review Diff" |
| `Ctrl+P` → `rd` → `Enter` | theoretical 4 events | **Not verified** — `rd` did not narrow to Review Diff in testing |

Minimum keystrokes to scroll to the 5th hunk of 10 once the view is open:

| Path | Keystrokes | Observed? |
|------|------------|-----------|
| `Tab` `n n n n` | 5 | Cursor advances, but **viewport does not snap to the hunk header** and there is no index feedback |

**Missing power-user shortcut:** there is no "jump to file N" or
`:<file>` addressing mode inside Review Diff. To reach the 4th file you
must `Tab` to files panel and press `j` three times.

**Good:** Tab toggles focus between files pane and diff pane — verified
twice and well-behaved. `PageDown` scrolls the diff viewport and updates
the cursor line counter correctly in the *unified* view.

### H5. Aesthetic & Minimalist Design — **Pass**

- Two-pane layout (30% files, 70% diff) is balanced at 160 cols.
- Section headers (`▸ Changes` / `▸ Untracked`) are collapsible-style
  indicators that clearly group the file list.
- The toolbar is dense (`s Stage  u Unstage  d Discard │ c Comment  N Note  x Del │ e Export  q Close  ↵ Open  Tab Switch  r Refresh`) — 28 characters of
  actionable hints, separated by `│` glyphs. At 160 cols this fits; at
  narrower widths it will wrap or truncate (see prior BUG-10).
- No superfluous chrome in the diff pane — line-number column is
  intentionally omitted in the unified view, present in side-by-side.

**Cost:** the toolbar *duplicates* information that exists in menus and
documentation, yet omits the two keys users most need (`n`/`p` are only
shown **after** Tab-switching focus to the diff pane). Unified-view users
don't know hunk navigation exists until they've already switched focus.

---

## 3. Friction Points (Flow-by-Flow)

### Flow 1 — Happy Path (modify `happy.py`: 3 adds, 2 dels, 1 hunk)

**Friction level: low.** Time-to-first-read ≈ 7 keystrokes.

- ✅ Review Diff opens in ~300ms on debug build.
- ✅ `@@ -1 +1 @@` header present; coloring correct.
- ✅ Status bar shows `Review Diff: 14 hunks`.
- ⚠ On first open, focus is on **files pane** — pressing `n` or `Enter`
  before Tabbing feels ambiguous. This matches prior BUG-3 findings.
- ⚠ Header reads "@@ -1 +1 @@" without range counts (`-1,12 +1,16`).

### Flow 2 — Monolith (1,000-line file, 10 hunks)

**Friction level: high.**

- ✅ Rendering of 10 hunks was instant; `PageDown`×50 completed in ~3s
  total (most of that was `tmux send-keys` overhead) with no visible
  input lag.
- ❌ **H1 violation:** pressing `n` four times moved cursor from line 21
  to line 29 (~+2 per press) — not a hunk-sized jump. No "hunk 5 of 10"
  indicator confirmed arrival at the target. Reproduces BUG-4.
- ❌ Top-of-viewport did not re-anchor to the jumped-to hunk header; we
  scrolled past hunks while thinking we were on hunk 5.
- ⚠ The non-standard `@@ line_0047 = value_47 ... @@` header replaces the
  `@@ -47,3 +47,4 @@` standard, so line-number context must be inferred
  from the content, not read directly.

### Flow 3 — Edge Cases

| State | Outcome |
|-------|---------|
| Whitespace-only change | Diff shows `-hello world` vs `+hello world` with **no visual cue** for the trailing space. Tab chars **are** rendered as `→` glyph (good). Two-space runs are not distinguished from one-space runs. |
| Newly created untracked file | Renders as `@@ -0 +1 @@` with all `+` lines. Layout not broken. ✅ |
| Deleted file | Unified view: `@@ -1 +0 @@` followed by `-` lines. ✅ |
| **Deleted file drill-down** (side-by-side) | **Does NOT hang** — the view opens with OLD content on the left and an empty pane on the right. BUG-5 from the prior combined report appears to be **fixed**. ✅ |

### Flow 4 — Lost User / Error Recovery

| Input | Response |
|-------|----------|
| `Ctrl+P` → "revw difff" | Top result: "Markdown: Toggle Compose/Preview" — Review Diff **not in top 10**. |
| `Ctrl+P` → "reiew" (missing v) | "Review Diff" is top match. ✅ |
| `Ctrl+P` → "rview" (missing e) | "Stop Review Diff" / "Refresh Review Diff" shown — acceptable. |
| Invalid keys in diff panel (`x`, `z`, `Q`, `%`) | Status bar: `Editing disabled in this buffer`. No panic, but a generic message that doesn't tell the user what they *can* do. |
| `Ctrl+C` at editor top level | Swallowed cleanly — editor remains alive. ✅ |

**Finding:** The fuzzy finder is a strict **subsequence** matcher. It does
not tolerate inserted or substituted characters. Standard NN/g guidance
calls for typo-tolerant search (Levenshtein within 1–2 edits) on
rarely-used commands.

---

## 4. Color & Layout Analysis (ANSI-parsed)

ANSI 256-color codes harvested from `capture-pane -e`:

| Role | Code | RGB (approx) | Notes |
|------|------|--------------|-------|
| Removed-line background | `48;5;52` | `#5F0000` | Dark red |
| Added-line background | `48;5;22` | `#005F00` | Dark green |
| Removed-line fg | `38;5;203` | `#FF5F5F` | Salmon — **contrast OK** on black bg |
| Added-line fg | `38;5;40` / `38;5;64` | `#00D700` / `#5F8700` | Bright green |
| Status bar bg | `48;5;226` | `#FFFF00` | Yellow |
| Status bar fg | `38;5;16` | `#000000` | Black |
| Tilde (empty-row) fg | `38;5;59` | dim grey | Unobtrusive |

**Accessibility notes:**

- Red/green-only encoding is the classic deuteranopia trap. The `-`/`+`
  glyph does carry the semantic, but the **background** color does the
  heavy visual lifting. A colorblind user relying on glyphs alone will
  struggle with pure-whitespace diffs (see Flow 3).
- Contrast ratios: `#FF5F5F` on `#5F0000` ≈ 4.0:1 (passes WCAG AA for
  normal text, fails AAA). `#00D700` on `#005F00` ≈ 3.9:1 (AA only).
- Status bar yellow (`#FFFF00`) bg with black text is high contrast
  (>10:1) — ✅.

**Layout / alignment:**

- In side-by-side view, line numbers are right-aligned within a 3-char
  column (`  1`, ` 42`, `171`). For files >999 lines the column expands
  correctly.
- Unified-view diff does **not** show line numbers in the gutter
  (screen_04). This is a deviation from `git diff --unified` with line
  numbers and a measurable source of friction when discussing a specific
  line with a teammate ("which line is that?").
- The `│` vertical separator between panes is rendered as single-column
  box-drawing — clean.

---

## 5. Actionable Recommendations

Ordered by effort-to-impact ratio.

### R1 — Show "Hunk N of M" in the status bar (Low effort, High impact)

Add a right-aligned segment: `Hunk 3 / 10`. Update it on every `n`, `p`,
`j`, `k`, or scroll event. This single change fixes the H1 violation in
Flow 2 and gives users navigational confidence.

*Files:* `crates/fresh-editor/plugins/audit_mode.ts` (status message
builder) + an additional `status.hunk_position` key in
`audit_mode.i18n.json`.

### R2 — Snap viewport to the hunk header on `n` / `p` (Medium effort)

Currently `n` only nudges the cursor. Change the handler so that after
moving the cursor to `hunkHeaderRows[idx]`, it forces viewport top =
`hunkHeaderRows[idx] - 2` (two lines of leading context). Also highlight
the hunk header row (reverse video) while the cursor is inside that
hunk. This addresses BUG-4 at its root.

### R3 — Word-level diff highlighting for intra-line changes (Medium effort)

Especially for whitespace-only diffs, byte-level reverse-video on the
*differing* ranges would make `-hello world` vs `+hello world ` (trailing
space) and tab-vs-space changes self-evident. The `diff_nav` plugin
already computes character ranges for `review_export_session`; expose the
same ranges to the renderer.

### R4 — Adopt standard `@@ -start,count +start,count @@` hunk header (Low effort)

Replace the custom "first context line" header with the git-standard one.
Users coming from `git diff`, GitHub, and `vimdiff` will recognize it
immediately (H3: Consistency & Standards). The first context line can be
appended after the closing `@@` as GitHub does:
`@@ -47,7 +47,7 @@ def greet(name):`.

### R5 — Upgrade the command palette to typo-tolerant fuzzy match (Medium effort)

Replace strict subsequence matching in
`crates/fresh-editor/src/input/fuzzy/mod.rs` with a hybrid: subsequence
→ if < N results, fall back to Levenshtein (edit distance ≤ 2) over the
command label. This resolves Flow 4's "revw difff" → Markdown mismatch
and brings the palette in line with VS Code / Zed expectations.

### R6 (Bonus) — Auto-focus the files panel on Review Diff launch (Trivial effort)

Documented in the prior combined report as BUG-3. A one-line fix in
`start_review_diff()` eliminates the silent "first key press does
nothing" trap that every new user hits.

---

## 6. Pass 2 — Extended Scenarios (18 additional flows)

Planned in `docs/internal/REVIEW_DIFF_EXTENDED_SCENARIOS.md` (33 scenarios
catalogued; 18 executed in this pass). Artifacts under
`/tmp/eval-workspace/pass2/p2_*.txt`.

### 6.1 Summary table

| ID | Scenario | Verdict | Heuristic | Key evidence |
|----|----------|---------|-----------|--------------|
| S1  | Unicode / emoji in diff | **Pass** | H3/H5 | `p2_06_uni.txt` — Japanese, Cyrillic, emoji (🦀🚀✨) all align in both panes. |
| S2  | 2,000-char single line | **Partial** | H5 | `p2_36_hscroll.txt` — viewport does not scroll on `Right`; only `End` jumps to column 2001. No overflow/continuation glyph. |
| S4  | File with no trailing newline | **Fail** | H3 | `p2_09_nonl.txt` — the standard `\ No newline at end of file` marker is **absent**; user cannot tell the newline was stripped. |
| S6  | Pure-additions diff | **Pass** | H3 | `p2_04_pure_del.txt` context — `@@ -1 +1 @@` with `+` lines only, clean. |
| S7  | Pure-deletions diff | **Pass** | H3 | `p2_05_pure_del.txt` — `-` lines only, no alignment issues. |
| S8  | Review Diff in non-git dir | **Fail** | H1/H9 | `p2_41_nogit.txt` — empty panes, `0 hunks`, **no error message** despite `status.not_git_repo` existing in i18n. |
| S9  | Clean repo, zero changes | **Fail** | H1 | `p2_42_clean.txt` — indistinguishable from S8. `panel.no_changes` string is never rendered. |
| S10 | `git mv` rename detection | **Pass** | H3 | `p2_02_review_open.txt` — shows `R  rename_me.txt → renamed.txt` (arrow glyph included). |
| S13 | Merge conflict file (`UU`) | **Partial** | H9 | `p2_43_conflict.txt` — status `U`, listed in *both* `Staged` and `Changes`, diff pane shows `(no diff available)`. No hint `<<<<<<` markers exist, no 3-way view. |
| S14 | 80 × 24 terminal | **Fail** | H5 | `p2_29_resize_80.txt` — the menu bar, tab row, and toolbar all disappear; only the two panes remain. Every discoverable key hint is gone. |
| S16 | Resize from 160×45 → 80×24 → 160×45 | **Fail** | H9 | `p2_30_resize_restore.txt` — layout does **not** auto-recover to the new width. A manual `r` press fixes it (`p2_31_after_r.txt`), but nothing tells the user that. Confirms BUG-2. |
| S18 | Stage → review | **Pass** | H1/H3 | `p2_02_review_open.txt` — a dedicated `▸ Staged` section appears above `▸ Changes`; staged file (`happy.py`) is grouped correctly. |
| S19 | Discard confirmation | **Pass** | H9 | `p2_14_discard_prompt.txt` — modal "Discard changes in file / Cancel" with `This cannot be undone.` warning. Proper error-prevention. |
| S20 | Add comment → inline render | **Pass** | H1 | `p2_16_after_comment.txt` — shows `» [hunk] Nit: consider using a constant` directly under the hunk header. Regression-fix for prior BUG-6 (comments on hunks, at least, now render). |
| S22 | Export review to markdown | **Pass** | H4 | `p2_17_export.txt` + `s_main/.review/session.md` — timestamped, includes file/hunk counts and grouped comments. |
| S26 | `n` vs `N` key collision | **Partial** | H3 | `p2_22_lower_n.txt` vs `p2_23_upper_N.txt` — `n` moves cursor ~1 line (no hunk-jump feedback); `N` opens the "Note:" prompt. Distinct behaviors but easy to mis-fire. |
| S27 | Menu-bar discoverability | **Fail** | H6 | `p2_10_menu.txt`, `p2_11_edit_menu.txt`, `p2_12_go_menu.txt` — `File`, `Edit`, and `Go` menus have **no** "Review Diff" entry. Only `Command Palette…` under `Go`, which delegates back to `Ctrl+P`. |
| S28 | F1 / in-app Manual | **Fail** | H10 | `p2_25_help_search.txt` — `Ctrl+F` "review diff" → `No matches found for 'review diff'` in the Fresh Manual. |
| S29 | Close + reopen persistence | **Pass** | H2 | `p2_21_pure_add_reopen.txt` — the inline comment from S20 survives close-and-reopen. Session file on disk is the source of truth. |
| S30 | `NO_COLOR=1` env var | **Fail** | Accessibility | `p2_39_nocolor_ansi.txt` — full 256-color ANSI palette (`[48;5;52m`, `[48;5;22m`, `[38;5;203m`, …) emitted despite the standard `NO_COLOR` signal. |

### 6.2 New friction points discovered in pass 2

- **F-PASS2-A (H9).** Resize-then-wait is silently broken. The fix path
  (`r`) exists but is never surfaced; a user with a tiling WM who
  switches monitors will see "the toolbar vanished" and assume the
  feature is broken.
- **F-PASS2-B (H1).** The empty `Review Diff: 0 hunks` screen is used
  for three disjoint conditions — not a repo, no changes, and "review
  failed silently." Each needs its own status line.
- **F-PASS2-C (Accessibility).** `NO_COLOR` is a well-known standard
  (https://no-color.org). Ignoring it is a hard accessibility fail for
  users on monochrome terminals, screen readers with ANSI-stripping,
  and deuteranopic users.
- **F-PASS2-D (H10).** Running `F1` → search for "review diff" returns
  zero results. The Manual must at minimum document the feature name,
  the launcher (`Ctrl+P → Review Diff`), and the key bindings shown in
  the toolbar.
- **F-PASS2-E (H3).** Files whose trailing newline is stripped look
  like plain line edits because the `\ No newline at end of file`
  marker from `git diff` is dropped. Reviewers will not catch the
  newline regression — high risk for shell scripts and fixture files.
- **F-PASS2-F (H9).** Merge-conflict files (`UU`) appear twice in the
  file list (once under `Staged`, once under `Changes`) and the diff
  pane reads `(no diff available)`. The user is told "there's
  something wrong" but given no affordance to resolve it inline.

### 6.3 Additional recommendations (delta from §5)

- **R7 — Render empty-state messages.** Wire the existing
  `status.not_git_repo` and `panel.no_changes` i18n keys into the
  files pane. For non-git: show a centred message "Not a git
  repository — run `git init` to enable Review Diff." For clean
  repos: "No changes to review. Modify a file and press `r` to
  refresh."
- **R8 — Auto-recover layout on terminal resize.** Hook the `resize`
  event to invalidate cached pane widths and recompute the toolbar /
  tab row. If a full recompute is expensive, at least repaint
  chrome-only on every resize.
- **R9 — Honor `NO_COLOR`.** At the ANSI writer (see
  `crates/fresh-editor/src/view/color_support.rs`), short-circuit
  256-color emission when `std::env::var_os("NO_COLOR").is_some()`.
  Fall back to plain text with `-` / `+` carrying all meaning.
- **R10 — Preserve the `\ No newline at end of file` marker.** The
  underlying `git diff` output contains it; the plugin's parser must
  forward it to the diff buffer. Render as a dim-foreground,
  italicised line so it doesn't disrupt the hunk shape.
- **R11 — Discoverability.** Add `Review Diff…` to the `Go` menu (or
  a new `Review` submenu). Add a "Review Diff" section to the Fresh
  Manual (`docs/manual.md` or the plugin README) covering the
  toolbar keys, side-by-side drill-down, and export formats.
- **R12 — De-duplicate merge-conflict files in the files pane.** Show
  `UU file` once (in its own "Conflicts" section if needed), and make
  `Enter` open the file in a regular editor buffer pre-navigated to
  the first `<<<<<<<` marker.

---

## Appendix A — Reproduction Commands

```bash
# 1. Build (debug)
cargo build

# 2. Prepare test repo (whitespace + new + deleted + monolith)
cd /tmp && mkdir eval && cd eval && git init -q
# ... (see body for full setup; ran from /tmp/eval-workspace/testrepo)

# 3. tmux session with ANSI capture
tmux new-session -d -s tui-test -x 160 -y 45
tmux send-keys -t tui-test "cd /tmp/eval-workspace/testrepo && \
  /home/user/fresh/target/debug/fresh" C-m
sleep 1

# 4. Open Review Diff
tmux send-keys -t tui-test C-p; sleep 0.5
tmux send-keys -t tui-test -l "review diff"; sleep 0.5
tmux send-keys -t tui-test Enter; sleep 1

# 5. Capture with colors
tmux capture-pane -t tui-test -p -e > current_screen.txt
```

## Appendix B — Screen Capture Index

All artifacts under `/tmp/eval-workspace/`:

| File | Scenario |
|------|----------|
| `screen_04_review_ansi.txt` | Happy-path unified diff, ANSI |
| `screen_07_nextHunk.txt` | `n` in unified diff — no visible jump |
| `screen_13_whitespace_ansi.txt` | Whitespace-only diff |
| `screen_16_drilldown.txt` | Side-by-side for `monolith.txt` |
| `screen_17_sxs_next.txt` | `n` in side-by-side — viewport moves but `Ln` stays 1 |
| `screen_20_delete_drill.txt` | Deleted-file drill-down (no hang — BUG-5 fixed) |
| `screen_22_typo.txt` | Palette with "revw difff" — wrong top match |
| `screen_28_ctrlc.txt` | Ctrl+C at root — editor survives |

### Pass-2 captures (`/tmp/eval-workspace/pass2/`)

| File | Scenario |
|------|----------|
| `p2_02_review_open.txt` | Staged section + rename display (S10, S18) |
| `p2_06_uni.txt` / `_ansi.txt` | Unicode / emoji diff (S1) |
| `p2_09_nonl.txt` | File missing trailing newline (S4) |
| `p2_10_menu.txt` / `p2_11_edit_menu.txt` / `p2_12_go_menu.txt` | Menu-bar walk (S27) |
| `p2_14_discard_prompt.txt` | Discard confirmation dialog (S19) |
| `p2_16_after_comment.txt` | Inline comment rendering (S20) |
| `p2_17_export.txt` + `.review/session.md` | Markdown export (S22) |
| `p2_21_pure_add_reopen.txt` | Comment persists across close/reopen (S29) |
| `p2_22_lower_n.txt` / `p2_23_upper_N.txt` | `n` vs `N` key collision (S26) |
| `p2_25_help_search.txt` | F1 Manual — "review diff" not found (S28) |
| `p2_29_resize_80.txt` | Chrome disappears at 80×24 (S14) |
| `p2_30_resize_restore.txt` / `p2_31_after_r.txt` | Resize corruption + manual recovery (S16) |
| `p2_35_long_sxs.txt` / `p2_36_hscroll.txt` / `p2_37_end.txt` | Long-line horizontal scroll (S2) |
| `p2_39_nocolor.txt` / `_ansi.txt` | `NO_COLOR=1` ignored (S30) |
| `p2_41_nogit.txt` | Review Diff outside a git repo (S8) |
| `p2_42_clean.txt` | Review Diff on a clean repo (S9) |
| `p2_43_conflict.txt` | Merge-conflict (`UU`) file (S13) |

Full scenario catalogue (33 scenarios, 18 executed here, 15 deferred) is
in `docs/internal/REVIEW_DIFF_EXTENDED_SCENARIOS.md`.

---

## 7. Pass 3 — Manual Interactive Validation (supersedes earlier claims)

Pass 3 re-ran every defect claim interactively under `tmux`, capturing
state **before and after** each keystroke (with generous sleeps to avoid
dropped events) on a purpose-built repo with five 9-line hunks.
Artifacts: `/tmp/validate/c_*.txt`.

### 7.1 Retracted claims

These items were in the earlier priority lists but the pass-3 captures
*disprove* them. They should **not** be acted on as bugs.

| Earlier claim | Capture-verified reality |
|---------------|--------------------------|
| "`n` / `p` advance the cursor only ~1 line — hunk navigation is broken." | `n` jumps exactly between hunk headers. Measured progression on a 9-line/hunk file: `Ln 1 → 2 → 11 → 20 → 29 → 38`. Past the last hunk, cursor clamps (`38 → 38 → 38`). `c_03`–`c_09`. |
| "Viewport does not scroll when `n` / `p` are pressed in the unified pane." | Viewport scrolls to follow the cursor. After 10 `n` presses the top of the viewport moved from `L006` to `L107`; after 5 `p` presses back to `L071`. `c_15`, `c_16`, `c_17`. |
| "No current-hunk highlight exists." | The current hunk header is drawn with bg `256:17` (dark blue) while other hunk headers stay on the default black bg. `c_11_viewport_ansi.txt`. |
| "`NO_COLOR=1` is ignored — the editor still emits 256-color ANSI." | `NO_COLOR` **is** honored. With `NO_COLOR=1`, only `[0m` and `[4m` are emitted; without it, 25 distinct 256-color codes appear. `c_24_nocolor.txt` vs `c_25_color.txt`. The pass-2 counter-example was captured after a shell-prompt corruption that never set the env var. |
| "`r` (refresh) recovers the layout after a terminal resize." | Only partial. After shrink 160×45 → 80×24 → 160×45, `r` does *not* restore the menu / toolbar / tab row; close-and-reopen of the review tab also fails. Only restarting the editor recovers. See §7.2 item 1. |

### 7.2 Validated defects (the real priority list)

Each row is capture-proven.

#### P0 — Ship-blockers

| # | Defect | Evidence |
|---|--------|----------|
| 1 | **Terminal resize is unrecoverable.** Shrink 160×45 → 80×24 → grow back leaves menu, toolbar, and tab row hidden. `r` refresh doesn't fix it; resize-bump doesn't fix it; close-and-reopen of the review tab leaves stale rendering on the right pane. Only killing and relaunching the editor recovers. | `c_40_at_80.txt`, `c_41_back.txt`, `c_42_after_r.txt`, `c_44_bump.txt`, `c_46_reopen.txt` |
| 2 | **Side-by-side `n` / `p` move the viewport but do NOT update the status-bar `Ln` / `Col`.** Arrow keys update `Ln` correctly; `n` / `p` leave it stale. | `c_18_sxs.txt` (Ln 1) → `c_21_sxs_n3.txt` (viewport at `L054` but `Ln 8`) → `c_19_sxs_down1.txt` (Down → `Ln 7`, immediate update) |
| 3 | **No "Hunk N of M" indicator anywhere.** Status bar shows only the *total* count (`Review Diff: 35 hunks`); never the current index. | `c_03`–`c_09` show the counter unchanged across every hunk jump |
| 4 | **Empty state is ambiguous.** Non-git directory and clean git repo render *byte-identically*: empty `GIT STATUS` pane, `DIFF` header with no filename, `Review Diff: 0 hunks`. The i18n keys `status.not_git_repo` and `panel.no_changes` exist but are never displayed. | `c_22_nogit.txt`, `c_23_clean.txt` |

#### P1 — High-impact UX gaps

| # | Defect | Evidence |
|---|--------|----------|
| 5 | **Unified diff pane has no per-keyword syntax highlighting.** Side-by-side does (`def` → fg 207, `return` → fg 51); unified pane uses one foreground color per `+` / `-` line. | `c_33_unified_syntax.txt`: `[1m[38;5;51mdef add(a: int, b: int) -> int:[0m` (single color for the whole keyword-rich line). `c_34_sxs_syntax.txt`: `[38;5;207mdef`, `[38;5;51mreturn`, per-token colors. |
| 6 | **`n` / `p` do nothing when the files pane is focused.** Pressing `n` there neither advances the file selection nor moves the diff cursor. | `c_36_n_filespane.txt` — still on `a.py`, diff still at hunk 1. |
| 7 | **`n` / `p` do not cross file boundaries.** From the last hunk of `a.py`, further `n` presses don't jump to the first hunk of `b.py`; cursor clamps just past the last hunk header of the current file. | `c_37_n_pastend.txt` — Ln 20, still on `a.py`. |
| 8 | **Hunk-nav keys (`n` / `p`) only appear in the toolbar after the diff pane is focused.** A user who never `Tab`s into the diff pane never learns hunk navigation exists. | Compare `c_01_review.txt` (no `n` / `p` in toolbar) vs `c_13_diff_start.txt` (toolbar shows `n Next  p Prev`). |
| 9 | **Whitespace-only changes have no per-character highlight.** Trailing-space and double-space edits look identical on the `-` and `+` lines; only the leading marker differs. | `screen_13_whitespace_ansi.txt` (pass 1) — full-line bg, no intra-line spans. |

#### P2 — Standards & discoverability

| # | Defect | Evidence |
|---|--------|----------|
| 10 | **Non-standard hunk header.** Renders as `@@ L006 @@` (a context-line preview) instead of git-standard `@@ -X,Y +X,Y @@ <signature>`. Breaks muscle memory from `git diff` / GitHub / `vimdiff`, and prevents counting added/removed lines per hunk at a glance. | `c_15_start.txt` |
| 11 | **Review Diff is in zero top-level menus.** Walked every menu (File / Edit / View / Selection / Go / LSP / Help). Only `Go → Command Palette…` exists — which delegates back to `Ctrl+P`. | `c_26_menu_file.txt`–`c_32_menu_help.txt` |
| 12 | **F1 in-app Manual lacks the feature.** `Ctrl+F` "review diff" in the Manual → `No matches found`. | `screen_25_help_search.txt` (pass 1) |
| 13 | **Fuzzy palette is subsequence-only — no typo tolerance.** "revw difff" returns `Markdown: Toggle Compose/Preview` instead of `Review Diff`. | `screen_22_typo.txt` (pass 1) |
| 14 | **`\ No newline at end of file` marker is dropped.** Diff for a file stripped of its trailing newline shows only the normal `+modified` line with no marker — reviewers will miss newline regressions in shell scripts / fixtures. | `p2_09_nonl.txt` (pass 2) |
| 15 | **Merge-conflict files (`UU`) appear in both `Staged` and `Changes` sections** with `(no diff available)` and no resolution affordance. | `p2_43_conflict.txt` (pass 2) |

#### P3 — Polish & edge cases

| # | Defect | Evidence |
|---|--------|----------|
| 16 | **`N` / `n` key collision.** Lowercase `n` = next hunk; capital `N` = open Note prompt. Distinct but easy to mis-fire; no other toolbar key pairs rely on case-sensitivity. | `p2_22_lower_n.txt` vs `p2_23_upper_N.txt` |
| 17 | **At 80 × 24, menu / tab row / toolbar all vanish.** No graceful degradation to a single-glyph legend. | `c_40_at_80.txt` |
| 18 | **No overflow indicator** when a diff line is truncated at pane width. `End` scrolls horizontally, but nothing signals that the line continues. | `screen_04_review_plain.txt`, `p2_37_end.txt` |
| 19 | **Files list is sorted alphabetically, not naturally.** `many/f10.txt` precedes `many/f2.txt`. | `p2_02_review_open.txt` |
| 20 | **No line numbers in the unified diff gutter.** Side-by-side has them; unified does not. Makes "which line is that?" conversations awkward. | `c_15_start.txt` |
| 21 | **No "reopen last review" command.** After `q`, every re-entry requires the 4-keystroke palette round-trip. | No entry found in palette listing (`screen_03b_search.txt`). |

### 7.3 Behaviours verified working (do not "fix")

- `n` / `p` hunk navigation in the unified diff pane.
- Viewport auto-scrolls to follow cursor in unified view.
- Current hunk header is highlighted (dark-blue bg).
- `NO_COLOR=1` is honored.
- Cursor position is preserved across `Tab` between panes.
- Inline comments render as `» [hunk] …` and persist across close-reopen.
- `d` discard shows a proper confirmation dialog.
- Rename detection renders cleanly as `R old → new`.
- Unicode and emoji align correctly in both panes.
- `q` cleanly closes the review and the editor survives.

## 8. Sprint Bundle (final, validated)

- **Sprint A (stabilise):** §7.2 items 1–4. Unblocks the feature for
  real-world PR workflows; every item here is a silent-failure today.
- **Sprint B (standards & a11y):** items 5, 9, 10, 14. Small diffs,
  cumulative big win.
- **Sprint C (navigation ergonomics):** items 6, 7, 8. Close the
  `n` / `p` coverage holes — files pane, cross-file, toolbar visibility.
- **Sprint D (discoverability & polish):** items 11, 12, 13, 15,
  16–21.
