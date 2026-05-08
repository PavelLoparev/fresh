//! Panel registry — maps plugin-allocated `panel_id` to mounted spec
//! and hit-area data for click routing.
//!
//! The registry is the source of truth for "which panels exist, what
//! spec are they currently rendering, and which buffer rows belong
//! to which widget." It does *not* own the virtual buffer the
//! rendered output goes into — the plugin still owns the virtual
//! buffer and passes its `BufferId` at mount time.

use fresh_core::api::WidgetSpec;
use fresh_core::BufferId;
use std::collections::HashMap;

/// Plugin-allocated panel identifier. Unique within a plugin; the
/// editor does not interpret the value.
pub type PanelId = u64;

/// One clickable rectangle within a rendered widget panel.
///
/// The renderer produces one `HitArea` per interactive widget node
/// (`Toggle`, `Button` in v1). Layout containers (`Row`, `Col`,
/// `Spacer`, `HintBar`, `Raw`) emit no hit areas of their own; their
/// children's hit areas bubble up with row/byte offsets adjusted to
/// reflect the final on-screen position.
///
/// Hit-test is `(buffer_row, buffer_col_byte) ∈ rectangle`; the byte
/// range is in UTF-8 bytes within the row's text, matching the
/// coordinate space `mouse_click` already delivers
/// (`HookArgs::MouseClick::buffer_col`).
#[derive(Debug, Clone)]
pub struct HitArea {
    /// Stable widget key from the spec, or empty when the spec did
    /// not assign one.
    pub widget_key: String,
    /// Widget kind discriminator: `"toggle"` or `"button"`.
    pub widget_kind: &'static str,
    /// 0-indexed row within the rendered virtual buffer.
    pub buffer_row: u32,
    /// First UTF-8 byte (inclusive) within the row's text.
    pub byte_start: usize,
    /// Last UTF-8 byte (exclusive) within the row's text.
    pub byte_end: usize,
    /// Event payload to deliver with the `widget_event` hook.
    /// For `"toggle"`: `{ "checked": <new value> }`. For
    /// `"button"`: `{}`.
    pub payload: serde_json::Value,
    /// Event type to deliver with the `widget_event` hook
    /// (`"toggle"` or `"activate"`).
    pub event_type: &'static str,
}

/// Widget instance state retained across spec updates, keyed by
/// the widget's stable `key`. This is the "Spec/instance separation"
/// described in §6 of the design doc — a plugin can rebuild its
/// `WidgetSpec` from scratch on every model change without losing
/// scroll offset, cursor position, expanded keys, or focus, because
/// stateful widgets look up their instance state by `key`.
///
/// In v1 only `List` carries instance state (its scroll offset).
/// `TextInput` cursor and `Tree` expanded-keys live here once those
/// widgets become host-stateful.
#[derive(Debug, Clone, Default)]
pub enum WidgetInstanceState {
    /// Empty/placeholder — never persisted, used as a default.
    #[default]
    None,
    /// `List` instance state: the host-owned scroll offset.
    List { scroll_offset: u32 },
}

/// Per-panel state retained between renders. The reconciler will use
/// the previous spec to compute the minimum mutation when a future
/// `UpdateWidgetPanel` arrives.
#[derive(Debug, Clone)]
pub struct WidgetPanelState {
    /// The virtual buffer this panel renders into.
    pub buffer_id: BufferId,
    /// The currently-mounted spec.
    pub spec: WidgetSpec,
    /// Click rectangles for the rendered output, in declaration
    /// order. Hit-test scans linearly — the small N (one per
    /// interactive widget per panel) doesn't justify a spatial
    /// index.
    pub hits: Vec<HitArea>,
    /// Widget instance state by widget `key`. Survives re-renders —
    /// see `WidgetInstanceState` for what's stored.
    pub instance_states: HashMap<String, WidgetInstanceState>,
}

/// Global registry of mounted widget panels.
#[derive(Debug, Default)]
pub struct WidgetRegistry {
    panels: HashMap<PanelId, WidgetPanelState>,
}

impl WidgetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mount or replace a panel. Returns the previous state if the
    /// panel was already mounted (the dispatcher may use this to
    /// detect re-mounts on the same id).
    ///
    /// `instance_states` carries widget instance state for the
    /// freshly-rendered spec. On the first mount this is whatever
    /// the renderer initialized; on re-mount the dispatcher may
    /// pass through the previous panel's `instance_states` so
    /// scroll offsets and other widget state survive a
    /// `MountWidgetPanel` that replaces an existing panel.
    pub fn mount(
        &mut self,
        panel_id: PanelId,
        buffer_id: BufferId,
        spec: WidgetSpec,
        hits: Vec<HitArea>,
        instance_states: HashMap<String, WidgetInstanceState>,
    ) -> Option<WidgetPanelState> {
        self.panels.insert(
            panel_id,
            WidgetPanelState {
                buffer_id,
                spec,
                hits,
                instance_states,
            },
        )
    }

    /// Replace the spec and hit areas on an already-mounted panel.
    /// Returns `Ok(buffer_id)` to render into, or `Err(())` if no
    /// panel exists for that id (caller should drop the update —
    /// the plugin re-emitted after unmount).
    pub fn update(
        &mut self,
        panel_id: PanelId,
        spec: WidgetSpec,
        hits: Vec<HitArea>,
        instance_states: HashMap<String, WidgetInstanceState>,
    ) -> Result<BufferId, ()> {
        match self.panels.get_mut(&panel_id) {
            Some(state) => {
                state.spec = spec;
                state.hits = hits;
                state.instance_states = instance_states;
                Ok(state.buffer_id)
            }
            None => Err(()),
        }
    }

    /// Read-only access to the instance state for a panel — used by
    /// the dispatcher to thread previous scroll offsets / cursor
    /// positions into the next render so they persist.
    pub fn instance_states(
        &self,
        panel_id: PanelId,
    ) -> Option<&HashMap<String, WidgetInstanceState>> {
        self.panels.get(&panel_id).map(|s| &s.instance_states)
    }

    /// Tear down a panel. Returns the buffer_id the panel was
    /// rendering into, so the caller can clear the buffer if it
    /// owns it.
    pub fn unmount(&mut self, panel_id: PanelId) -> Option<BufferId> {
        self.panels.remove(&panel_id).map(|s| s.buffer_id)
    }

    /// Read-only access to a panel's current state.
    pub fn get(&self, panel_id: PanelId) -> Option<&WidgetPanelState> {
        self.panels.get(&panel_id)
    }

    /// All currently-mounted panel ids — useful for theme-change
    /// re-render passes (every panel re-renders against the new
    /// theme without plugin involvement).
    pub fn panel_ids(&self) -> Vec<PanelId> {
        self.panels.keys().copied().collect()
    }

    /// Hit-test the given buffer-local position against every
    /// currently-mounted panel rendering into `buffer_id`. Returns
    /// the matching panel id and a clone of the hit area on a hit,
    /// `None` otherwise.
    ///
    /// Linear scan: panel count is typically 1 per buffer; per-panel
    /// hit count is small (one per interactive widget). A spatial
    /// index would be over-engineering at this scale.
    pub fn hit_test(
        &self,
        buffer_id: BufferId,
        row: u32,
        col_byte: u32,
    ) -> Option<(PanelId, HitArea)> {
        for (pid, state) in &self.panels {
            if state.buffer_id != buffer_id {
                continue;
            }
            for hit in &state.hits {
                if hit.buffer_row == row
                    && (col_byte as usize) >= hit.byte_start
                    && (col_byte as usize) < hit.byte_end
                {
                    return Some((*pid, hit.clone()));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn empty_spec() -> WidgetSpec {
        WidgetSpec::Col {
            children: vec![],
            key: None,
        }
    }

    fn make_hit(row: u32, byte_start: usize, byte_end: usize, key: &str) -> HitArea {
        HitArea {
            widget_key: key.into(),
            widget_kind: "button",
            buffer_row: row,
            byte_start,
            byte_end,
            payload: json!({}),
            event_type: "activate",
        }
    }

    #[test]
    fn hit_test_finds_widget_inside_range() {
        let mut reg = WidgetRegistry::new();
        reg.mount(
            42,
            BufferId(7),
            empty_spec(),
            vec![make_hit(0, 0, 5, "a"), make_hit(0, 7, 12, "b")],
            HashMap::new(),
        );
        let hit = reg.hit_test(BufferId(7), 0, 8).expect("inside b");
        assert_eq!(hit.0, 42);
        assert_eq!(hit.1.widget_key, "b");
    }

    #[test]
    fn hit_test_returns_none_when_outside_range() {
        let mut reg = WidgetRegistry::new();
        reg.mount(
            1,
            BufferId(0),
            empty_spec(),
            vec![make_hit(0, 0, 5, "a")],
            HashMap::new(),
        );
        assert!(reg.hit_test(BufferId(0), 0, 5).is_none(), "byte_end is exclusive");
        assert!(reg.hit_test(BufferId(0), 0, 100).is_none());
        assert!(reg.hit_test(BufferId(0), 1, 0).is_none(), "wrong row");
        assert!(reg.hit_test(BufferId(99), 0, 0).is_none(), "wrong buffer");
    }

    #[test]
    fn unmount_clears_hits() {
        let mut reg = WidgetRegistry::new();
        reg.mount(
            5,
            BufferId(2),
            empty_spec(),
            vec![make_hit(0, 0, 3, "x")],
            HashMap::new(),
        );
        assert!(reg.hit_test(BufferId(2), 0, 1).is_some());
        reg.unmount(5);
        assert!(reg.hit_test(BufferId(2), 0, 1).is_none());
    }

    #[test]
    fn update_replaces_hits() {
        let mut reg = WidgetRegistry::new();
        reg.mount(
            5,
            BufferId(2),
            empty_spec(),
            vec![make_hit(0, 0, 3, "old")],
            HashMap::new(),
        );
        reg.update(
            5,
            empty_spec(),
            vec![make_hit(1, 4, 9, "new")],
            HashMap::new(),
        )
        .expect("mounted");
        // Old hit gone; new hit visible.
        assert!(reg.hit_test(BufferId(2), 0, 1).is_none());
        let hit = reg.hit_test(BufferId(2), 1, 5).unwrap();
        assert_eq!(hit.1.widget_key, "new");
    }
}
