//! The vertical tab strip — Zen's `gZenVerticalTabsManager`: the ordered tab
//! list shown in the sidebar of the active workspace. Pinned tabs are shown
//! first, then the unpinned tabs below them.

use crate::tab::Tab;

/// The vertical tab strip (`gZenVerticalTabsManager`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VerticalTabs {
    /// Pinned tabs (`Tab.pinned`) — the pinned-tabs container is at the top
    /// of the strip (`#organizeTabsToWorkspaceSections`).
    pub pinned: Vec<u64>,
    /// Unpinned tabs (`tabbrowser-arrowscrollbox`).
    pub unpinned: Vec<u64>,
    /// `hasCollapsedPinnedTabs`: the collapsible-pins section is collapsed
    /// (`nsZenCollapsiblePins`).
    pub collapsed_pins: bool,
}

impl VerticalTabs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.pinned.is_empty() && self.unpinned.is_empty()
    }

    pub fn len(&self) -> usize {
        self.pinned.len() + self.unpinned.len()
    }

    /// `pushPinnedTab`-style: appends to the pinned section.
    pub fn push_pinned(&mut self, id: u64) {
        self.pinned.push(id);
    }

    /// Appends an unpinned tab to the strip.
    pub fn push_unpinned(&mut self, id: u64) {
        self.unpinned.push(id);
    }

    /// `Tab.pinned` changes (`TabPinned`/`TabUnpinned` events): the tab moves
    /// between the strip sections.
    pub fn set_pinned(&mut self, id: u64, pinned: bool) {
        self.remove(id);
        if pinned {
            self.pinned.push(id);
        } else {
            self.unpinned.push(id);
        }
    }

    /// Removes the tab from whichever section it is in — strip entry removal
    /// (`gBrowser.removeTab`). Returns the section it was in, if any.
    pub fn remove(&mut self, id: u64) -> Option<bool> {
        if let Some(i) = self.pinned.iter().position(|&t| t == id) {
            self.pinned.remove(i);
            return Some(true);
        }
        if let Some(i) = self.unpinned.iter().position(|&t| t == id) {
            self.unpinned.remove(i);
            return Some(false);
        }
        None
    }

    pub fn contains(&self, id: u64) -> bool {
        self.pinned.contains(&id) || self.unpinned.contains(&id)
    }

    /// Reorders the tab within its section (`moveTabTo` in the strip).
    pub fn reorder(&mut self, id: u64, to: usize) -> bool {
        if let Some(i) = self.pinned.iter().position(|&t| t == id) {
            let item = self.pinned.remove(i);
            self.pinned.insert(to.min(self.pinned.len()), item);
            return true;
        }
        if let Some(i) = self.unpinned.iter().position(|&t| t == id) {
            let item = self.unpinned.remove(i);
            self.unpinned.insert(to.min(self.unpinned.len()), item);
            return true;
        }
        false
    }

    /// Builds the strip of a workspace: pinned tabs first (Zen orders the
    /// pinned section with `insertBefore(..., firstChild)`), then the
    /// unpinned ones — mirrors `#organizeTabsToWorkspaceSections` for one
    /// space. `workspace_id` is the space's `zen-workspace-id`.
    pub fn from_workspace(tabs: &[Tab], workspace_id: Option<&str>) -> Self {
        let mut strip = VerticalTabs::new();
        for tab in tabs {
            if tab.workspace_id.as_deref() != workspace_id {
                continue;
            }
            if tab.pinned {
                strip.pinned.push(tab.id);
            } else {
                strip.unpinned.push(tab.id);
            }
        }
        strip
    }

    /// `nsZenCollapsiblePins.toggle()`.
    pub fn toggle_pins_collapsed(&mut self) {
        self.collapsed_pins = !self.collapsed_pins;
    }

    /// All tab ids: pinned first, then unpinned — the visual strip order.
    pub fn iter(&self) -> impl Iterator<Item = u64> + '_ {
        self.pinned.iter().chain(self.unpinned.iter()).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_remove() {
        let mut s = VerticalTabs::new();
        assert!(s.is_empty());
        s.push_pinned(1);
        s.push_unpinned(2);
        s.push_unpinned(3);
        assert_eq!(s.len(), 3);
        assert!(s.contains(1));
        assert!(!s.contains(99));

        assert_eq!(s.remove(2), Some(false)); // was in unpinned
        assert_eq!(s.remove(1), Some(true)); // was pinned
        assert_eq!(s.remove(1), None);
        assert_eq!(s.pinned, Vec::<u64>::new());
        assert_eq!(s.unpinned, vec![3]);
    }

    #[test]
    fn set_pinned_moves_sections() {
        let mut s = VerticalTabs::new();
        s.push_unpinned(1);
        s.set_pinned(1, true);
        assert_eq!(s.pinned, vec![1]);
        assert!(s.unpinned.is_empty());
        s.set_pinned(1, false);
        assert_eq!(s.unpinned, vec![1]);
        assert!(s.pinned.is_empty());
    }

    #[test]
    fn reorder_within_sections() {
        let mut s = VerticalTabs::new();
        s.push_unpinned(1);
        s.push_unpinned(2);
        s.push_unpinned(3);
        assert!(s.reorder(3, 0));
        assert_eq!(s.unpinned, vec![3, 1, 2]);
        assert!(s.reorder(1, 5)); // clamped to the end
        assert_eq!(s.unpinned, vec![3, 2, 1]);
        assert!(!s.reorder(99, 0));
    }

    #[test]
    fn from_workspace_filters_by_space_and_orders_pinned_first() {
        let mut tabs = vec![
            Tab::new(1, "https://a"), // other space, unpinned
            Tab::new(2, "https://b"), // this space, pinned
            Tab::new(3, "https://c"), // this space, unpinned
            Tab::new(4, "https://d"), // this space, pinned
        ];
        tabs[0].workspace_id = Some("other".into());
        tabs[1].pinned = true;
        tabs[1].workspace_id = Some("ws".into());
        tabs[2].workspace_id = Some("ws".into());
        tabs[3].pinned = true;
        tabs[3].workspace_id = Some("ws".into());

        let s = VerticalTabs::from_workspace(&tabs, Some("ws"));
        assert_eq!(s.pinned, vec![2, 4]); // pinned in strip order
        assert_eq!(s.unpinned, vec![3]);
    }

    #[test]
    fn toggle_pins_collapsed() {
        let mut s = VerticalTabs::new();
        assert!(!s.collapsed_pins);
        s.toggle_pins_collapsed();
        assert!(s.collapsed_pins);
        s.toggle_pins_collapsed();
        assert!(!s.collapsed_pins);
    }

    #[test]
    fn iter_is_pinned_then_unpinned() {
        let mut s = VerticalTabs::new();
        s.push_pinned(1);
        s.push_pinned(2);
        s.push_unpinned(3);
        assert_eq!(s.iter().collect::<Vec<_>>(), vec![1, 2, 3]);
    }
}
