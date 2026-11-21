//! `zen_fork::Shell` — the Zen Browser shell: tabs, workspaces, the
//! vertical tab strip, compact mode, Glance and split view, over the
//! `Tab` list Zen keeps in `gBrowser`.

use crate::compact_mode::{CompactMode, Hoverable};
use crate::glance::{Glance, GlanceState};
use crate::prefs::Prefs;
use crate::split_view::{GridType, SplitGroup, SplitView};
use crate::tab::{GlanceId, Tab, TabId};
use crate::vertical_tabs::VerticalTabs;
use crate::workspace::{Workspace, Workspaces, WorkspaceId};

/// The Zen Browser shell — the state Zen keeps in `gBrowser`,
/// `gZenWorkspaces`, `gZenCompactModeManager`, `gZenGlanceManager` and
/// `gZenViewSplitter`.
#[derive(Clone, Debug)]
pub struct Shell {
    tabs: Vec<Tab>,
    selected: Option<TabId>,
    workspaces: Workspaces,
    vertical: VerticalTabs,
    compact: CompactMode,
    glance: Glance,
    split: SplitView,
    prefs: Prefs,
    next_tab_id: u64,
}

impl Shell {
    /// Zen's startup state: one workspace and one empty tab
    /// (`#initializeEmptyTab` — `gBrowser.addTrustedTab("about:blank", { _forZenEmptyTab: true })`).
    pub fn new() -> Self {
        let mut shell = Shell {
            tabs: Vec::new(),
            selected: None,
            workspaces: Workspaces::new(),
            vertical: VerticalTabs::new(),
            compact: CompactMode::new(),
            glance: Glance::new(),
            split: SplitView::new(),
            prefs: Prefs::default(),
            next_tab_id: 1,
        };
        let id = shell.add_tab("about:blank");
        shell.mark_tab_empty(id);
        shell
    }

    fn new_tab_id(&mut self) -> TabId {
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        id
    }

    /// `gBrowser.addTab` — the tab gets `zen-workspace-id` of the active
    /// workspace (`selectEmptyTab`: `tab.setAttribute("zen-workspace-id", ...)`),
    /// and becomes selected.
    pub fn add_tab(&mut self, url: impl Into<String>) -> TabId {
        let id = self.new_tab_id();
        let workspace_id = self.workspaces.active_id().map(str::to_string);
        self.tabs.push(Tab { id, url: url.into(), workspace_id, ..Tab::default() });
        self.vertical.push_unpinned(id);
        self.selected = Some(id);
        id
    }

    /// Adds a pinned tab to the active workspace (Zen: `gBrowser.pinTab`).
    pub fn add_pinned_tab(&mut self, url: impl Into<String>) -> TabId {
        let id = self.add_tab(url);
        self.set_pinned(id, true);
        id
    }

    /// `Tab.pinned` toggles: the tab moves between the strip sections
    /// (`TabPinned`/`TabUnpinned` events → `#organizeTabsToWorkspaceSections`).
    pub fn set_pinned(&mut self, id: TabId, pinned: bool) -> bool {
        let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) else {
            return false;
        };
        tab.pinned = pinned;
        self.vertical.set_pinned(id, pinned);
        true
    }

    /// `gBrowser.removeTab`.
    pub fn remove_tab(&mut self, id: TabId) -> bool {
        if !self.tabs.iter().any(|t| t.id == id) {
            return false;
        }
        self.vertical.remove(id);
        self.tabs.retain(|t| t.id != id);
        if self.selected == Some(id) {
            self.selected = self.tabs.first().map(|t| t.id);
        }
        true
    }

    /// The `zen-empty-tab` attribute (`Tab.hasAttribute("zen-empty-tab")`).
    pub fn mark_tab_empty(&mut self, id: TabId) -> bool {
        match self.tabs.iter_mut().find(|t| t.id == id) {
            Some(t) => {
                t.empty = true;
                true
            }
            None => false,
        }
    }

    pub fn tab(&self, id: TabId) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == id)
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// The tabs of the active workspace — Zen's filter in
    /// `#organizeTabsToWorkspaceSections`:
    /// `tab.getAttribute("zen-workspace-id") === workspace.uuid`.
    pub fn tabs_in_active_workspace(&self) -> Vec<&Tab> {
        let active = self.workspaces.active_id();
        self.tabs.iter().filter(|t| t.workspace_id.as_deref() == active).collect()
    }

    /// `Tab.label` — the page title once the page loads.
    pub fn set_title(&mut self, id: TabId, title: impl Into<String>) -> bool {
        match self.tabs.iter_mut().find(|t| t.id == id) {
            Some(t) => {
                t.title = title.into();
                true
            }
            None => false,
        }
    }

    /// `TabSelect`: activates the tab; the splitter tracks it
    /// (`nsZenViewSplitter.onTabSelect` → `onLocationChange` → `updateSplitView`).
    pub fn select(&mut self, id: TabId) -> bool {
        if !self.tabs.iter().any(|t| t.id == id) {
            return false;
        }
        self.selected = Some(id);
        self.split.update_split_view(id);
        true
    }

    pub fn selected(&self) -> Option<TabId> {
        self.selected
    }

    /// Zen's `moveTabToWorkspace(tab, workspaceId)` — assigns the tab to a
    /// workspace (the `zen-workspace-id` attribute).
    pub fn move_tab_to_workspace(&mut self, id: TabId, workspace_id: Option<&str>) -> bool {
        let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) else {
            return false;
        };
        tab.workspace_id = workspace_id.map(str::to_string);
        self.rebuild_strip();
        true
    }

    /// Rebuilds the strip for the active workspace — Zen swaps each
    /// `zen-workspace` element's tab sections when switching spaces.
    fn rebuild_strip(&mut self) {
        self.vertical = VerticalTabs::from_workspace(&self.tabs, self.workspaces.active_id());
    }

    pub fn vertical(&self) -> &VerticalTabs {
        &self.vertical
    }

    // -- Workspaces --------------------------------------------------------

    pub fn workspaces(&self) -> &Workspaces {
        &self.workspaces
    }

    pub fn workspaces_mut(&mut self) -> &mut Workspaces {
        &mut self.workspaces
    }

    pub fn active_workspace(&self) -> Option<&Workspace> {
        self.workspaces.active()
    }

    pub fn add_workspace(&mut self, name: impl Into<String>) -> WorkspaceId {
        self.workspaces.create(name, None)
    }

    /// `gZenWorkspaces.changeWorkspaceWithID` / `switchToWorkspace`.
    pub fn switch_workspace(&mut self, id: &str) -> bool {
        if !self.workspaces.activate(id) {
            return false;
        }
        self.rebuild_strip();
        true
    }

    /// Zen's wheel-scroll switch (`changeWorkspaceShortcut`): wraps with
    /// `zen.workspaces.wrap-around-navigation` (`naturalScroll` flips the
    /// delta sign in Zen — callers pass the effective delta).
    pub fn switch_workspace_relative(&mut self, delta: i32) -> bool {
        let wrap = self.prefs.workspaces_wrap_around_navigation;
        if self.workspaces.activate_relative(delta, wrap) {
            self.rebuild_strip();
            true
        } else {
            false
        }
    }

    // -- Glance ------------------------------------------------------------

    /// Zen's `openGlance` — while one is open, re-opens the current one
    /// (`#currentBrowser` check).
    pub fn open_glance(&mut self, parent: TabId, url: impl Into<String>) -> Option<GlanceId> {
        if let Some(id) = self.glance.current() {
            return Some(id);
        }
        let url = url.into();
        let workspace_id = self
            .tab(parent)
            .and_then(|t| t.workspace_id.clone())
            .or_else(|| self.workspaces.active_id().map(str::to_string));
        // `#createBrowserElement`: the child tab is inserted into the parent
        // tab (`zen-glance-tab`), sharing the parent's space.
        let tab_id = self.new_tab_id();
        self.tabs.push(Tab { id: tab_id, url: url.clone(), workspace_id, ..Tab::default() });
        self.vertical.push_unpinned(tab_id);
        let gid = self.glance.open(parent, url, tab_id)?;
        if let Some(t) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
            t.glance = Some(gid);
        }
        Some(gid)
    }

    /// Zen's `closeGlance({ onTabClose })`: closes the current Glance and
    /// removes its overlay tab (`#cleanupGlanceElements`).
    pub fn close_glance(&mut self) -> bool {
        let gid = match self.glance.current() {
            Some(id) => id,
            None => return false,
        };
        let tab = self.glance.state(gid).map(|s| s.glance_tab);
        let ok = self.glance.close_current();
        if let Some(t) = tab {
            self.remove_tab(t);
        }
        ok
    }

    pub fn glance(&self, id: GlanceId) -> Option<&GlanceState> {
        self.glance.state(id)
    }

    pub fn glance_current(&self) -> Option<GlanceId> {
        self.glance.current()
    }

    pub fn glance_parent(&self, id: GlanceId) -> Option<TabId> {
        self.glance.parent(id)
    }

    pub fn is_glance_tab(&self, id: TabId) -> bool {
        self.tab(id).map_or(false, |t| t.glance.is_some())
    }

    /// Zen's `gZenGlanceManager.getTabOrGlanceParent` — the tab a Glance
    /// belongs to (a Glance tab maps to its parent).
    pub fn tab_or_glance_parent(&self, id: TabId) -> Option<TabId> {
        if let Some(t) = self.tab(id) {
            if let Some(gid) = t.glance {
                return self.glance.parent(gid);
            }
        }
        Some(id)
    }

    // -- Split view ---------------------------------------------------------

    /// Zen's `splitTabs(tabs)` — splits the given tabs (default grid type).
    /// Maps Glance tabs to their parents (`gZenGlanceManager.getTabOrGlanceParent`)
    /// and filters empty tabs (`!t.hasAttribute("zen-empty-tab")`).
    pub fn split_tabs(&mut self, tabs: &[TabId]) -> Option<SplitGroup> {
        let mut ids: Vec<TabId> = Vec::new();
        for &t in tabs {
            let parent = self.tab_or_glance_parent(t)?;
            if !ids.contains(&parent) {
                ids.push(parent);
            }
        }
        ids.retain(|&t| self.tab(t).map_or(false, |tab| !tab.empty));
        if ids.len() < 2 {
            return None;
        }
        self.split.split_tabs(ids, GridType::Grid, true).cloned()
    }

    pub fn split(&self) -> &SplitView {
        &self.split
    }

    pub fn split_mut(&mut self) -> &mut SplitView {
        &mut self.split
    }

    pub fn split_group_of(&self, tab: TabId) -> Option<&SplitGroup> {
        let (gi, _) = self.split.group_of(tab)?;
        self.split.groups().get(gi)
    }

    pub fn split_remove_tab(&mut self, tab: TabId) -> bool {
        self.split.remove_tab_from_group(tab)
    }

    // -- Compact mode / prefs ------------------------------------------------

    pub fn compact(&self) -> &CompactMode {
        &self.compact
    }

    pub fn compact_mut(&mut self) -> &mut CompactMode {
        &mut self.compact
    }

    /// Zen's hover events (`_setElementExpandAttribute`).
    pub fn enter_compact_element(&mut self, e: Hoverable) {
        self.compact.enter(e);
    }

    pub fn leave_compact_element(&mut self, e: Hoverable) {
        self.compact.leave(e);
    }

    pub fn prefs(&self) -> &Prefs {
        &self.prefs
    }

    pub fn prefs_mut(&mut self) -> &mut Prefs {
        &mut self.prefs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_one_workspace_and_one_empty_tab() {
        let shell = Shell::new();
        assert_eq!(shell.workspaces().spaces().len(), 1);
        assert_eq!(shell.workspaces().spaces()[0].name, "Space");
        assert!(shell.workspaces().active_id().is_some());
        assert_eq!(shell.tabs().len(), 1);
        assert!(shell.tabs()[0].empty);
        assert_eq!(shell.tab(shell.tabs()[0].id).unwrap().url, "about:blank");
    }

    #[test]
    fn add_tab_belongs_to_active_workspace() {
        let mut shell = Shell::new();
        let id = shell.add_tab("https://example.com");
        let tab = shell.tab(id).unwrap();
        assert_eq!(tab.url, "https://example.com");
        assert_eq!(tab.workspace_id, shell.workspaces().active_id().map(str::to_string));
        assert!(shell.vertical().contains(id));
        assert!(!tab.pinned);
        assert_eq!(shell.selected(), Some(id));
    }

    #[test]
    fn add_pinned_tab_moves_to_pinned_section() {
        let mut shell = Shell::new();
        let id = shell.add_pinned_tab("https://pinned.example");
        assert!(shell.tab(id).unwrap().pinned);
        assert!(shell.vertical().pinned.contains(&id));
        shell.set_pinned(id, false);
        assert!(!shell.vertical().pinned.contains(&id));
        assert!(shell.vertical().unpinned.contains(&id));
    }

    #[test]
    fn remove_tab_removes_from_tabs_and_strip() {
        let mut shell = Shell::new();
        let id = shell.add_tab("https://example.com");
        assert!(shell.remove_tab(id));
        assert!(!shell.vertical().contains(id));
        assert!(shell.tab(id).is_none());
        assert!(!shell.remove_tab(id));
    }

    #[test]
    fn switch_workspace_shows_only_its_tabs() {
        let mut shell = Shell::new();
        let other = shell.add_workspace("Work");
        let ws_tab = shell.add_tab("https://work.example");
        shell.move_tab_to_workspace(ws_tab, Some(&other));
        // The default space has only the empty tab now.
        assert!(!shell.tabs_in_active_workspace().iter().any(|t| t.id == ws_tab));
        assert!(shell.switch_workspace(&other));
        assert!(shell.tabs_in_active_workspace().iter().any(|t| t.id == ws_tab));
        assert_eq!(shell.workspaces().active_id(), Some(other.as_str()));
        assert!(!shell.switch_workspace("no-such-space"));
    }

    #[test]
    fn open_and_close_glance() {
        let mut shell = Shell::new();
        let parent = shell.selected().unwrap();
        let gid = shell.open_glance(parent, "https://zen-glance.example").unwrap();
        assert_eq!(shell.glance_parent(gid), Some(parent));
        assert!(shell.is_glance_tab(shell.glance(gid).unwrap().glance_tab));
        // While open, opening again returns the current Glance.
        assert_eq!(shell.open_glance(parent, "https://other.example"), Some(gid));
        assert!(shell.close_glance());
        assert!(!shell.close_glance());
        assert!(shell.glance_current().is_none());
    }

    #[test]
    fn split_tabs_from_shell() {
        let mut shell = Shell::new();
        let a = shell.add_tab("https://a.example");
        let b = shell.add_tab("https://b.example");
        let group = shell.split_tabs(&[a, b]).unwrap();
        assert_eq!(group.tabs, vec![a, b]);
        assert!(shell.split().is_active());
        assert_eq!(shell.split_group_of(a).map(|g| g.id), Some(group.id));
    }

    #[test]
    fn compact_and_prefs_accessors() {
        let mut shell = Shell::new();
        shell.enter_compact_element(Hoverable::Sidebar);
        assert!(shell.compact().is_hovering(Hoverable::Sidebar));
        assert!(shell.prefs().workspaces_wrap_around_navigation);
        let _ = shell.prefs_mut();
        assert!(shell.active_workspace().is_some());
    }
}
