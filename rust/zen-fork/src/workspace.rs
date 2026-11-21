//! Workspaces — Zen's `nsZenWorkspace` + `nsZenWorkspaces` (`gZenWorkspaces`).
//!
//! In Zen, every tab carries a `zen-workspace-id`; a workspace is the named
//! container holding those tabs (`#organizeTabsToWorkspaceSections`), plus a
//! name, icon and contextual-identity container. `Workspaces` models the
//! workspace list: `_workspaceCache` + `activeWorkspace`.

use crate::tab::Tab;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Identifies a workspace (Zen uses a UUID string: `workspace.uuid`).
pub type WorkspaceId = String;

/// A workspace (`nsZenWorkspace`; newer Zen names the list items "spaces").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    /// `workspace.icon`: an emoji or `.svg` path; Zen renders it in the strip
    /// next to the name (`generateMenuItemForWorkspace`).
    pub icon: Option<String>,
    /// `workspace.containerTabId`: the contextual identity this space uses
    /// (`0` = the default container).
    pub container_id: Option<u64>,
    /// `hasCollapsedPinnedTabs`: the space's collapsible-pins section.
    pub pins_collapsed: bool,
}

/// Error for workspace operations (`getWorkspaceFromId` returns null;
/// removal/activation of unknown spaces fails).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkspaceError {
    /// No workspace with that id.
    NotFound(WorkspaceId),
    /// The operation is not permitted in this state.
    NotAllowed(String),
}

static UUID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Zen's `gZenUIManager.generateUuidv4()` — nanos + counter based.
fn generate_uuid() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let c = UUID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("zen-{:016x}-{:04x}", nanos, c)
}

/// The workspace list — `gZenWorkspaces._workspaceCache` + `activeWorkspace`.
#[derive(Clone, Debug, Default)]
pub struct Workspaces {
    spaces: Vec<Workspace>,
    active_id: Option<WorkspaceId>,
}

impl Workspaces {
    /// `new()` — a fresh session without saved spaces starts with one
    /// space named "Space" (`restoreWorkspacesFromSessionStore`).
    pub fn new() -> Self {
        let mut ws = Workspaces::default();
        let id = ws.create("Space", None);
        ws.active_id = Some(id);
        ws
    }

    /// `#createWorkspaceData(name, icon)` — adds a workspace.
    pub fn create(&mut self, name: impl Into<String>, icon: Option<String>) -> WorkspaceId {
        let id = generate_uuid();
        self.spaces.push(Workspace {
            id: id.clone(),
            name: name.into(),
            icon,
            container_id: None,
            pins_collapsed: false,
        });
        id
    }

    /// `saveWorkspace(workspaceData, { insertBeforeId, insertAfterId })`.
    pub fn save(&mut self, workspace: Workspace) {
        if let Some(i) = self.spaces.iter().position(|w| w.id == workspace.id) {
            self.spaces[i] = workspace;
        } else {
            self.spaces.push(workspace);
        }
    }

    /// `reorderWorkspace(id, newPosition)` — moves the workspace in the list.
    pub fn reorder(&mut self, id: &str, new_position: usize) -> Result<(), WorkspaceError> {
        let i = self
            .spaces
            .iter()
            .position(|w| w.id == id)
            .ok_or_else(|| WorkspaceError::NotFound(id.to_string()))?;
        let ws = self.spaces.remove(i);
        self.spaces.insert(new_position.min(self.spaces.len()), ws);
        Ok(())
    }

    /// `removeWorkspace(windowID)` — if the removed workspace was active,
    /// the manager switches to another one (`changeWorkspace`).
    pub fn remove(&mut self, id: &str) -> Result<(), WorkspaceError> {
        let was_active = self.active_id.as_deref() == Some(id);
        let i = self
            .spaces
            .iter()
            .position(|w| w.id == id)
            .ok_or_else(|| WorkspaceError::NotFound(id.to_string()))?;
        self.spaces.remove(i);
        if was_active {
            self.active_id = self.spaces.first().map(|w| w.id.clone());
        }
        Ok(())
    }

    /// `getWorkspaces()`.
    pub fn spaces(&self) -> &[Workspace] {
        &self.spaces
    }

    /// `getWorkspaceFromId(id)`.
    pub fn get(&self, id: &str) -> Option<&Workspace> {
        self.spaces.iter().find(|w| w.id == id)
    }

    /// `activeWorkspace` getter.
    pub fn active_id(&self) -> Option<&str> {
        self.active_id.as_deref()
    }

    /// `activeWorkspace` setter — Zen degrades unknown ids to the first
    /// space: `value = spaces[0]?.uuid || ""`.
    pub fn set_active_id(&mut self, id: Option<&str>) -> Option<&str> {
        match id.filter(|i| self.spaces.iter().any(|w| w.id.as_str() == *i)) {
            Some(id) => self.active_id = Some(id.to_string()),
            None => self.active_id = self.spaces.first().map(|w| w.id.clone()),
        }
        self.active_id.as_deref()
    }

    /// `activate(id)`: returns false when the workspace doesn't exist (the
    /// caller falls back with `set_active_id`).
    pub fn activate(&mut self, id: &str) -> bool {
        if self.spaces.iter().any(|w| w.id == id) {
            self.active_id = Some(id.to_string());
            true
        } else {
            false
        }
    }

    /// `isWorkspaceActive(workspace)`.
    pub fn is_active(&self, id: &str) -> bool {
        self.active_id.as_deref() == Some(id)
    }

    /// `activeWorkspace` / `getActiveWorkspace()`.
    pub fn active(&self) -> Option<&Workspace> {
        self.active_id.as_ref().and_then(|id| self.get(id))
    }

    /// Zen's wheel-scroll/shortcut switch (`changeWorkspaceShortcut`): moves
    /// `delta` spaces, wrapping around when `zen.workspaces.wrap-around-navigation`
    /// is on (Zen's `naturalScroll` also flips the sign of the delta; the
    /// sign is the caller's).
    pub fn activate_relative(&mut self, delta: i32, wrap_around: bool) -> bool {
        if self.spaces.is_empty() {
            return false;
        }
        let current = self
            .active_id
            .as_deref()
            .and_then(|id| self.spaces.iter().position(|w| w.id == id));
        let next = match current {
            Some(i) => {
                let n = self.spaces.len() as i32;
                let target = i as i32 + delta;
                let next = if wrap_around {
                    ((target % n) + n) % n
                } else {
                    target.clamp(0, n - 1)
                };
                next as usize
            }
            None => 0,
        };
        let id = self.spaces[next].id.clone();
        self.activate(&id)
    }

    /// `resolveWorkspaceFromCLIString` — matches id or name, case-insensitively.
    pub fn resolve(&self, value: &str) -> Option<&str> {
        self.spaces
            .iter()
            .find(|w| w.id == value || w.name.eq_ignore_ascii_case(value))
            .map(|w| w.id.as_str())
    }

    /// `getWorkspaceIcon` — the space's icon, or the first grapheme of its
    /// name (Zen uppercases it).
    pub fn icon_of(ws: &Workspace) -> String {
        if let Some(icon) = ws.icon.clone().filter(|i| !i.is_empty()) {
            return icon;
        }
        ws.name.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default()
    }

    /// The tabs of a workspace — Zen filters `gBrowser.tabs` by
    /// `zen-workspace-id` (`#clearAnyZombieTabs`, `#organizeTabsToWorkspaceSections`).
    pub fn tabs_in<'a>(&self, workspace_id: &str, tabs: &'a [Tab]) -> Vec<&'a Tab> {
        tabs.iter()
            .filter(|t| t.workspace_id.as_deref() == Some(workspace_id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_with_default_space() {
        let ws = Workspaces::new();
        assert_eq!(ws.spaces().len(), 1);
        assert_eq!(ws.spaces()[0].name, "Space");
        assert!(ws.active_id().is_some());
        assert!(ws.is_active(&ws.spaces()[0].id));
    }

    #[test]
    fn create_and_get() {
        let mut ws = Workspaces::new();
        let id = ws.create("Work", Some("💼".to_string()));
        assert_eq!(ws.get(&id).map(|w| w.name.as_str()), Some("Work"));
        assert_eq!(ws.get(&id).and_then(|w| w.icon.clone()), Some("💼".to_string()));
        assert_eq!(ws.spaces().len(), 2);
        // Creating doesn't change the active workspace.
        assert_eq!(ws.active_id(), Some(ws.spaces()[0].id.as_str()));
    }

    #[test]
    fn activate_and_set_active_id() {
        let mut ws = Workspaces::new();
        let id = ws.spaces()[0].id.clone();
        assert!(ws.activate(&id));
        assert!(!ws.activate("does-not-exist"));
        // Unknown/empty ids fall back to the first space (Zen's setter).
        assert_eq!(ws.set_active_id(None), Some(id.as_str()));
        assert_eq!(ws.set_active_id(Some("nope")), Some(id.as_str()));
        assert!(ws.is_active(&id));
    }

    #[test]
    fn resolve_by_name_and_id() {
        let mut ws = Workspaces::new();
        let id = ws.create("Work", None);
        assert_eq!(ws.resolve(&id), Some(id.as_str()));
        assert_eq!(ws.resolve("work"), Some(id.as_str())); // case-insensitive
        assert_eq!(ws.resolve("WORK"), Some(id.as_str()));
        assert_eq!(ws.resolve("space"), Some(ws.spaces()[0].id.as_str()));
        assert_eq!(ws.resolve("nothing"), None);
    }

    #[test]
    fn reorder() {
        let mut ws = Workspaces::new();
        let a = ws.spaces()[0].id.clone();
        let b = ws.create("B", None);
        ws.reorder(&a, 1).unwrap();
        assert_eq!(ws.spaces()[0].id, b);
        assert_eq!(ws.spaces()[1].id, a);
        assert!(ws.reorder("nope", 0).is_err());
    }

    #[test]
    fn remove() {
        let mut ws = Workspaces::new();
        let a = ws.spaces()[0].id.clone();
        let b = ws.create("B", None);
        assert!(ws.is_active(&a));

        // Removing the active space switches to another one.
        ws.remove(&a).unwrap();
        assert_eq!(ws.active_id(), Some(b.as_str()));

        // Removing a non-active space keeps the active one.
        let c = ws.create("C", None);
        ws.remove(&c).unwrap();
        assert_eq!(ws.active_id(), Some(b.as_str()));

        // Unknown ids fail.
        assert!(ws.remove("nope").is_err());

        // Removing the last space clears the active id.
        ws.remove(&b).unwrap();
        assert!(ws.active_id().is_none());
    }

    #[test]
    fn activate_relative() {
        let mut ws = Workspaces::new();
        let a = ws.spaces()[0].id.clone();
        let b = ws.create("B", None);
        assert!(ws.is_active(&a));

        // +1 moves to b.
        assert!(ws.activate_relative(1, true));
        assert!(ws.is_active(&b));
        // +1 wraps to a.
        assert!(ws.activate_relative(1, true));
        assert!(ws.is_active(&a));
        // -1 from a wraps to b.
        assert!(ws.activate_relative(-1, true));
        assert!(ws.is_active(&b));
        // Without wrap, -1 from a clamps to a.
        assert!(ws.activate_relative(-1, false));
        assert!(ws.is_active(&a));
        // delta 0 keeps the active space.
        assert!(ws.activate_relative(0, false));
        assert!(ws.is_active(&a));
    }

    #[test]
    fn icon_of() {
        let mut ws = Workspaces::new();
        let id = ws.create("Work", None);
        let ws_ref = ws.get(&id).unwrap();
        assert_eq!(Workspaces::icon_of(ws_ref), "W");
        let id2 = ws.create("🚀 Launch", Some("./icons/rocket.svg".into()));
        let ws_ref2 = ws.get(&id2).unwrap();
        assert_eq!(Workspaces::icon_of(ws_ref2), "./icons/rocket.svg");
    }

    #[test]
    fn tabs_in_filters_by_workspace() {
        let ws = Workspaces::new();
        let id = ws.spaces()[0].id.clone();
        let mut tabs = vec![Tab::new(1, "https://a"), Tab::new(2, "https://b")];
        tabs[1].workspace_id = Some(id.clone());
        let ids: Vec<u64> = ws.tabs_in(&id, &tabs).iter().map(|t| t.id).collect();
        assert_eq!(ids, vec![2]);
        assert!(ws.tabs_in("other", &tabs).is_empty());
    }
}
