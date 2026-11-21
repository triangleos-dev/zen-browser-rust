//! The tab model — Zen's `Tab` objects (`gBrowser.tabs`) with the Zen
//! attributes, as Rust struct fields.

use crate::workspace::WorkspaceId;

/// Identifies a tab in the window (Zen uses numeric tab ids).
pub type TabId = u64;

/// Identifies a Glance overlay (Zen's `glance-id` on the child tab).
pub type GlanceId = u64;

/// The Zen-unique tab fields, in the words of the original browser:
///
/// | Zen attribute / field | Meaning |
/// |---|---|
/// | `zen-workspace-id` | the tab belongs to this workspace |
/// | `pinned` | the tab is pinned (`Tab.pinned`) |
/// | `zen-glance-tab` + `glance-id` | the tab is the overlay tab of a Glance |
/// | `zen-empty-tab` | the startup placeholder tab Zen starts with |
/// | `zen-essential` | a tab in the essentials area shared between spaces |
/// | `split-view` | the tab is part of a split view group |
/// | `label` / `favicon` | the page title / icon (`Tab.label`, `Tab.image`) |
///
/// (The rest of Zen's tab behavior — urlbar state, history, titles on
/// navigation — is engine substrate; see `REWRITE_PLAN.md`.)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tab {
    pub id: TabId,
    /// `url` — the tab's address; empty while pending.
    pub url: String,
    /// `label` — the tab's title, empty until the page loads.
    pub title: String,
    /// `favicon` — the tab icon (`Tab.image` in Zen).
    pub favicon: Option<String>,
    /// `pinned` on the Gecko tab.
    pub pinned: bool,
    /// `zen-workspace-id`: the workspace the tab belongs to.
    pub workspace_id: Option<WorkspaceId>,
    /// `zen-glance-tab` + `glance-id`: set when this tab is the overlay of a
    /// Glance (`nsZenGlanceManager.#glances` entries).
    pub glance: Option<GlanceId>,
    /// `zen-empty-tab`: the placeholder tab Zen starts with.
    pub empty: bool,
    /// `zen-essential`: a tab in the essentials area (`TabAddedToEssentials`).
    pub essential: bool,
    /// `split-view`: the tab is part of a split view group (Zen sets this on
    /// tabs in `nsZenViewSplitter._data`).
    pub split_view: bool,
}

impl Tab {
    pub fn new(id: TabId, url: impl Into<String>) -> Self {
        Tab { id, url: url.into(), ..Tab::default() }
    }

    /// Zen's placeholder: `gBrowser.addTrustedTab("about:blank", {
    /// _forZenEmptyTab: true })` + the `zen-empty-tab` attribute.
    pub fn blank(id: TabId) -> Self {
        let mut tab = Tab::new(id, "about:blank");
        tab.empty = true;
        tab
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tab_has_empty_zen_fields() {
        let tab = Tab::new(7, "https://example.com");
        assert_eq!(tab.id, 7);
        assert_eq!(tab.url, "https://example.com");
        assert!(!tab.pinned);
        assert!(tab.workspace_id.is_none());
        assert!(tab.glance.is_none());
        assert!(!tab.empty);
        assert!(!tab.essential);
        assert!(!tab.split_view);
    }

    #[test]
    fn blank_tab_is_empty() {
        let tab = Tab::blank(3);
        assert!(tab.empty);
        assert_eq!(tab.url, "about:blank");
    }
}
