//! Glance — Zen's `nsZenGlanceManager`: an overlay that previews the target
//! of a link on top of the current tab instead of opening a new one.

use crate::tab::{GlanceId, TabId};
use std::collections::HashMap;

/// The state of one Glance (`nsZenGlanceManager.#glances`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlanceState {
    pub id: GlanceId,
    /// The tab the overlay draws — Zen's `glance-id` on the child tab
    /// (`#createBrowserElement` inserts it into the parent tab).
    pub glance_tab: TabId,
    /// The tab it previews — Zen's `parentTab`.
    pub parent: TabId,
    /// The URL the Glance loads.
    pub url: String,
    /// Zen's `openGlance`/`quickCloseGlance` — whether the overlay is open.
    pub open: bool,
    /// `openDetachedGlance` — the Glance floats free of the tab
    /// (Zen's Little Window).
    pub detached: bool,
}

/// Glance state for the window (`nsZenGlanceManager.#glances`,
/// `#currentGlanceID`).
#[derive(Clone, Debug, Default)]
pub struct Glance {
    glimpses: HashMap<GlanceId, GlanceState>,
    current: Option<GlanceId>,
    next_id: GlanceId,
}

impl Glance {
    pub fn new() -> Self {
        Self::default()
    }

    /// Zen's `openGlance`: while one is open, it re-opens the current Glance
    /// instead of stacking a new one (the `#currentBrowser` check).
    pub fn open(&mut self, parent: TabId, url: String, glance_tab: TabId) -> Option<GlanceId> {
        if let Some(id) = self.current {
            return Some(id);
        }
        Some(self.register(parent, url, glance_tab, false))
    }

    /// Zen's `openDetachedGlance` — the Glance floats free of the tab.
    pub fn open_detached(&mut self, parent: TabId, url: String, glance_tab: TabId) -> GlanceId {
        self.register(parent, url, glance_tab, true)
    }

    fn register(
        &mut self,
        parent: TabId,
        url: String,
        glance_tab: TabId,
        detached: bool,
    ) -> GlanceId {
        self.next_id += 1;
        let id = self.next_id;
        self.glimpses.insert(
            id,
            GlanceState { id, glance_tab, parent, url, open: true, detached },
        );
        self.current = Some(id);
        id
    }

    /// Zen's `closeGlance({ onTabClose })` / `quickCloseGlance`.
    pub fn close(&mut self, id: GlanceId) -> bool {
        let was_current = self.current == Some(id);
        let removed = self.glimpses.remove(&id);
        if was_current {
            self.current = None;
        }
        removed.is_some()
    }

    /// `closeGlance()` on the current Glance.
    pub fn close_current(&mut self) -> bool {
        match self.current {
            Some(id) => self.close(id),
            None => false,
        }
    }

    /// Zen's `closeDetachedGlance`.
    pub fn close_detached(&mut self) -> bool {
        self.close_current()
    }

    pub fn current(&self) -> Option<GlanceId> {
        self.current
    }

    pub fn state(&self, id: GlanceId) -> Option<&GlanceState> {
        self.glimpses.get(&id)
    }

    pub fn current_state(&self) -> Option<&GlanceState> {
        self.current.and_then(|id| self.glimpses.get(&id))
    }

    pub fn parent(&self, id: GlanceId) -> Option<TabId> {
        self.state(id).map(|s| s.parent)
    }

    pub fn glance_tab(&self, id: GlanceId) -> Option<TabId> {
        self.state(id).map(|s| s.glance_tab)
    }

    pub fn is_open(&self, id: GlanceId) -> bool {
        self.state(id).map_or(false, |s| s.open)
    }

    pub fn is_open_current(&self) -> bool {
        self.current.is_some()
    }

    pub fn has_detached(&self) -> bool {
        self.current_state().map_or(false, |s| s.detached)
    }

    pub fn len(&self) -> usize {
        self.glimpses.len()
    }

    pub fn is_empty(&self) -> bool {
        self.glimpses.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_registers_current() {
        let mut g = Glance::new();
        assert!(g.is_empty());
        let id = g.open(5, "https://zen-glance.example".into(), 9).unwrap();
        assert_eq!(g.current(), Some(id));
        let state = g.state(id).unwrap();
        assert_eq!(state.parent, 5);
        assert_eq!(state.glance_tab, 9);
        assert_eq!(state.url, "https://zen-glance.example");
        assert!(state.open);
        assert!(!state.detached);
        assert!(g.is_open(id));
        assert_eq!(g.parent(id), Some(5));
        assert_eq!(g.glance_tab(id), Some(9));
    }

    #[test]
    fn open_while_open_returns_current() {
        let mut g = Glance::new();
        let id = g.open(1, "https://a.example".into(), 2).unwrap();
        let again = g.open(1, "https://b.example".into(), 2).unwrap();
        assert_eq!(again, id); // Zen: re-opens the current Glance
        assert_eq!(g.len(), 1);
    }

    #[test]
    fn close_and_reopen() {
        let mut g = Glance::new();
        let id = g.open(1, "https://a.example".into(), 2).unwrap();
        assert!(g.close(id));
        assert!(!g.close(id)); // already gone
        assert!(g.current().is_none());
        let id2 = g.open(1, "https://a.example".into(), 2).unwrap();
        assert_ne!(id2, id);
    }

    #[test]
    fn close_current() {
        let mut g = Glance::new();
        assert!(!g.close_current());
        let _id = g.open(1, "https://a.example".into(), 2).unwrap();
        assert!(g.close_current());
        assert!(g.current().is_none());
        assert!(!g.is_open_current());
    }

    #[test]
    fn detached() {
        let mut g = Glance::new();
        let id = g.open_detached(1, "https://a.example".into(), 2);
        assert!(g.state(id).unwrap().detached);
        assert!(g.has_detached());
        assert!(g.close_detached());
        assert!(!g.has_detached());
    }
}
