/**
 * Plugin widget library — declarative UI for Fresh plugins.
 *
 * Plugins describe panel content as a `WidgetSpec` tree. The host owns
 * rendering, theming, and (in later phases) hit-testing, focus, and
 * keymaps. This module provides:
 *
 *   - Type re-exports from the generated `fresh.d.ts` so plugins import
 *     `WidgetSpec` / `HintEntry` from one place.
 *   - Builder helpers (`row`, `col`, `hintBar`, `raw`) that produce the
 *     correct discriminated-union shape.
 *   - A `WidgetPanel` class that wraps the
 *     `mountWidgetPanel` / `updateWidgetPanel` / `unmountWidgetPanel`
 *     IPC trio with mount-once-then-update semantics.
 *   - `parseHintString(s)` — parses the legacy `Tab:section  Esc:close`
 *     string format used by today's plugin i18n bundles into
 *     `HintEntry[]`.
 *
 * See `docs/internal/plugin-widget-library-design.md`.
 *
 * @example
 *   import { WidgetPanel, hintBar, col, raw, parseHintString } from "./lib/widgets.ts";
 *
 *   const panel = new WidgetPanel(bufferId);
 *   panel.set(col(
 *     raw(myExistingEntries),
 *     hintBar(parseHintString(editor.t("panel.help"))),
 *   ));
 *   // …later, on every state change:
 *   panel.set(col(raw(newEntries), hintBar(myHints)));
 *   // …on close:
 *   panel.unmount();
 */

/// <reference path="./fresh.d.ts" />

// `fresh.d.ts` declares HintEntry / WidgetSpec / TextPropertyEntry as
// ambient globals (it is not an ES module). Re-export the relevant
// type names locally so plugin code can write
// `import type { WidgetSpec } from "./lib/widgets.ts"` without dipping
// into the ambient namespace directly.
export type WidgetSpec = globalThis.WidgetSpec;
export type HintEntry = globalThis.HintEntry;
export type ButtonKind = globalThis.ButtonKind;
type TextPropertyEntry = globalThis.TextPropertyEntry;

// =============================================================================
// Builder helpers — preferred over hand-writing `{ kind: "row", ... }`.
// =============================================================================

/** Horizontal layout. Children laid out left-to-right; inline-sized
 * children collapse into a single line. See §3 of the design doc. */
export function row(...children: WidgetSpec[]): WidgetSpec {
  return { kind: "row", children };
}

/** Vertical layout. Children stacked top-to-bottom. */
export function col(...children: WidgetSpec[]): WidgetSpec {
  return { kind: "col", children };
}

/** Keyboard-hint footer. Renders `<keys> <label>` per entry, with the
 * keys portion styled by the `ui.help_key_fg` theme key.
 *
 * Replaces the per-plugin hand-rolled help row. */
export function hintBar(entries: HintEntry[]): WidgetSpec {
  return { kind: "hintBar", entries };
}

/** Imperative-virtual-buffer escape hatch. Wraps an existing
 * `TextPropertyEntry[]` (the same shape `setVirtualBufferContent`
 * already accepts) so a plugin can migrate its panel one widget at a
 * time. */
export function raw(entries: TextPropertyEntry[]): WidgetSpec {
  return { kind: "raw", entries };
}

/** Boolean toggle, rendered as `[v] label` / `[ ] label`.
 * Pass `focused: true` to highlight (the host will own focus once
 * the keymap layer is wired). */
export function toggle(
  checked: boolean,
  label: string,
  options?: { focused?: boolean; key?: string },
): WidgetSpec {
  return {
    kind: "toggle",
    checked,
    label,
    focused: options?.focused ?? false,
    key: options?.key,
  };
}

/** Action button, rendered as `[ Label ]`. `intent` controls visual
 * emphasis: `"normal"` (default) → no override, `"primary"` → bold,
 * `"danger"` → error theme key. */
export function button(
  label: string,
  options?: {
    focused?: boolean;
    intent?: ButtonKind;
    key?: string;
  },
): WidgetSpec {
  return {
    kind: "button",
    label,
    focused: options?.focused ?? false,
    intent: options?.intent ?? "normal",
    key: options?.key,
  };
}

/** Horizontal spacer of fixed column count. In a `Row` it produces
 * `cols` spaces; at the top level or in a `Col` it produces a short
 * blank line. (Flex spacers — `Spacer { flex: true }` filling
 * remaining row width — arrive with the layout engine.) */
export function spacer(cols: number, key?: string): WidgetSpec {
  return { kind: "spacer", cols, key };
}

/** Vertical list of pre-rendered rows with host-managed selection
 * styling, click routing, and **virtual scrolling**. Plugin passes
 * the full dataset of items + a `visibleRows` count; the widget
 * owns scroll offset as instance state (keyed by `key`) and
 * auto-clamps it to keep `selectedIndex` in view. Plugins never
 * compute scroll math.
 *
 * Click on a row fires `widget_event` with `eventType: "select"` and
 * `payload: { index, key }` where `index` is the *absolute* index
 * into `items` (not the visible-window index).
 *
 * `key` is required for any List that should preserve scroll across
 * re-renders. Lists without a key reset to scroll=0 each render. */
export function list(options: {
  items: TextPropertyEntry[];
  itemKeys?: string[];
  selectedIndex?: number;
  visibleRows: number;
  key?: string;
}): WidgetSpec {
  return {
    kind: "list",
    items: options.items,
    itemKeys: options.itemKeys ?? [],
    selectedIndex: options.selectedIndex ?? -1,
    visibleRows: options.visibleRows,
    key: options.key,
  };
}

/** Single-line text input, rendered as `[value]` (or
 * `Label: [value]` if `label` is provided), with a one-cell cursor
 * highlight at `cursorByte` when ≥ 0. v1 is render-only — the
 * plugin still owns the value string and cursor, and the existing
 * `mode_text_input` + bound-key path handles editing. The widget
 * provides theme-keyed focus styling and removes the per-plugin
 * `buildFieldDisplay` / `addCursorOverlay` byte-offset arithmetic. */
export function textInput(
  value: string,
  options?: {
    cursorByte?: number;
    focused?: boolean;
    label?: string;
    placeholder?: string;
    maxVisibleChars?: number;
    key?: string;
  },
): WidgetSpec {
  return {
    kind: "textInput",
    value,
    cursorByte: options?.cursorByte ?? -1,
    focused: options?.focused ?? false,
    label: options?.label,
    placeholder: options?.placeholder,
    maxVisibleChars: options?.maxVisibleChars ?? 0,
    key: options?.key,
  };
}

// =============================================================================
// HintEntry parsing — for the legacy `Tab:section  Esc:close` format
// shipped in existing plugin i18n bundles.
// =============================================================================

/** Parse a hint string of the form `<keys>:<label>  <keys>:<label> ...`.
 *
 * The separator between entries defaults to two-or-more spaces (matching
 * what existing i18n bundles use). The separator between keys and label
 * within an entry is a colon.
 *
 * Empty input yields an empty array. Entries without a colon are
 * preserved with empty label. */
export function parseHintString(
  s: string,
  options?: { entrySep?: RegExp; keyLabelSep?: string },
): HintEntry[] {
  if (!s) return [];
  const entrySep = options?.entrySep ?? /\s{2,}/;
  const keyLabelSep = options?.keyLabelSep ?? ":";
  const parts = s.split(entrySep).filter((p) => p.length > 0);
  return parts.map((part) => {
    const idx = part.indexOf(keyLabelSep);
    if (idx < 0) {
      return { keys: part, label: "" };
    }
    return {
      keys: part.slice(0, idx).trim(),
      label: part.slice(idx + keyLabelSep.length).trim(),
    };
  });
}

// =============================================================================
// WidgetPanel — mount-once-update-many wrapper around the IPC trio.
// =============================================================================

/** A handle to a mounted widget panel. Construct one per virtual
 * buffer that should host widget-rendered content; call `set(spec)`
 * on every render; call `unmount()` when the buffer is closed.
 *
 * The first `set()` issues `mountWidgetPanel`; subsequent calls
 * issue `updateWidgetPanel`. Idempotent re-mount is guaranteed by the
 * host (see `WidgetRegistry::mount`). */
export class WidgetPanel {
  private mounted = false;
  private readonly panelId: number;
  private readonly bufferId: number;

  constructor(bufferId: number, panelId?: number) {
    this.bufferId = bufferId;
    this.panelId = panelId ?? allocatePanelId();
  }

  /** Returns the plugin-allocated panel id, useful for routing
   * widget events back through `editor.on("widget_event", ...)`. */
  id(): number {
    return this.panelId;
  }

  /** Render or re-render the panel against the given spec.
   * Cheap to call on every state change; the host reconciles. */
  set(spec: WidgetSpec): boolean {
    // deno-lint-ignore no-explicit-any
    const editor = (globalThis as any).editor;
    if (!this.mounted) {
      this.mounted = true;
      return editor.mountWidgetPanel(this.panelId, this.bufferId, spec);
    }
    return editor.updateWidgetPanel(this.panelId, spec);
  }

  /** Tear down the panel. The plugin retains ownership of the
   * underlying virtual buffer. Subsequent `set()` calls re-mount. */
  unmount(): boolean {
    if (!this.mounted) return true;
    this.mounted = false;
    // deno-lint-ignore no-explicit-any
    const editor = (globalThis as any).editor;
    return editor.unmountWidgetPanel(this.panelId);
  }
}

// =============================================================================
// Panel-id allocation. Plugin-side counter; need only be unique per
// plugin instance (the host doesn't interpret the value).
// =============================================================================

let nextPanelId = 1;
function allocatePanelId(): number {
  // Bias high so plugin-allocated ids don't collide with the
  // editor's internal panel-id space if it ever uses small ints.
  const id = nextPanelId++;
  return 0x1000_0000 + id;
}
