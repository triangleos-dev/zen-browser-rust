//! Compact mode — Zen's `gZenCompactModeManager` (`window.gZenCompactModeManager`):
//! the auto-hiding sidebar and toolbar.

use std::collections::HashSet;

/// Elements that can be hovered in compact mode — Zen's
/// `gZenCompactModeManager.hoverableElements`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Hoverable {
    /// The sidebar (`gNavToolbox`).
    Sidebar,
    /// The top toolbar (`zen-appcontent-navbar-wrapper`).
    Toolbar,
    /// The window buttons.
    WindowButtons,
}

/// The state of compact mode.
#[derive(Clone, Debug)]
pub struct CompactMode {
    /// The persisted preference (`zen-compact-mode` on the document element
    /// and `main-app-wrapper`).
    preference: bool,
    /// `zen.view.compact.hide-tabbar`: the sidebar is collapsed.
    hide_tabbar: bool,
    /// `zen.view.compact.hide-toolbar`: the toolbar is collapsed.
    hide_toolbar: bool,
    /// `zen.view.compact.sidebar-expanded`: the vertical tab strip is expanded.
    sidebar_expanded: bool,
    /// `zen.tabs.vertical.right-side`: the sidebar sits on the right.
    sidebar_is_on_right: bool,
    /// Window buttons sit on the left of the window (macOS-style layout;
    /// Zen uses it in `checkIfIllegalState` via `isWindowsStyledButtons`).
    window_buttons_on_left: bool,
    /// Elements the mouse is currently over (`zen-has-hover`).
    hovering: HashSet<Hoverable>,
    /// `zen-user-show` on the sidebar — the user forced it open
    /// (`toggleSidebar`).
    user_show: bool,
    /// The window is not in a layout where the toolbar is already hidden
    /// (`!gZenVerticalTabsManager.hidesTabsToolbar` — Zen's `shouldBeCompact`).
    hides_tabs_toolbar: bool,
}

impl Default for CompactMode {
    fn default() -> Self {
        CompactMode {
            preference: false, // zen.view.compact.enable-at-startup = false
            hide_tabbar: crate::prefs::compact_mode::HIDE_TABBAR,
            hide_toolbar: crate::prefs::compact_mode::HIDE_TOOLBAR,
            sidebar_expanded: crate::prefs::view::SIDEBAR_EXPANDED,
            sidebar_is_on_right: false,
            window_buttons_on_left: false,
            hovering: HashSet::new(),
            user_show: false,
            hides_tabs_toolbar: false,
        }
    }
}

impl CompactMode {
    pub fn new() -> Self {
        Self::default()
    }

    /// The `zen-compact-mode` preference.
    pub fn preference(&self) -> bool {
        self.preference
    }

    /// Zen's `set preference` — the layout vetoes:
    /// `if (!this.shouldBeCompact) value = false;`
    pub fn set_preference(&mut self, value: bool) -> bool {
        if !self.should_be_compact() {
            self.preference = false;
            return false;
        }
        // Zen: "We dont want the user to be able to spam the button"
        // (the `zen-compact-animating` guard).
        self.preference = value;
        self.preference
    }

    /// Zen's `toggle()` — flips the preference (`cmd_zenCompactModeToggle`).
    pub fn toggle(&mut self) -> bool {
        let value = !self.preference;
        self.set_preference(value)
    }

    /// Zen's `shouldBeCompact` — `!gZenVerticalTabsManager.hidesTabsToolbar`.
    pub fn should_be_compact(&self) -> bool {
        !self.hides_tabs_toolbar
    }

    /// Zen's `canHideSidebar` — `pref hide-tabbar || single toolbar set`.
    pub fn can_hide_sidebar(&self) -> bool {
        self.hide_tabbar || self.hides_tabs_toolbar
    }

    /// Zen's `canHideToolbar` — `pref hide-toolbar && !single toolbar set`.
    pub fn can_hide_toolbar(&self) -> bool {
        self.hide_toolbar && !self.hides_tabs_toolbar
    }

    /// Zen's `checkIfIllegalState`: returns whether the (re-repaired) state
    /// is the "closely illegal" one; repairs it by setting `hide-tabbar`.
    pub fn check_illegal_state(&mut self) -> bool {
        // Due to how we layout the sidebar and toolbar, some states are not
        // allowed mainly because the caption buttons aren't accessible.
        if self.sidebar_expanded {
            // Fast exit when the sidebar is expanded.
            return false;
        }
        let closely_illegal = self.window_buttons_on_left != self.sidebar_is_on_right;
        if closely_illegal && self.can_hide_toolbar() && !self.can_hide_sidebar() {
            // Illegal state: hide the sidebar instead of the toolbar.
            self.hide_tabbar = true;
            return true;
        }
        closely_illegal
    }

    /// Zen's `_setElementExpandAttribute(element, true)` — the element is hovered.
    pub fn enter(&mut self, element: Hoverable) {
        self.hovering.insert(element);
    }

    /// Zen's `_setElementExpandAttribute(element, false)`.
    pub fn leave(&mut self, element: Hoverable) {
        self.hovering.remove(&element);
    }

    /// Whether the element is hovered (the `zen-has-hover` attribute).
    pub fn is_hovering(&self, element: Hoverable) -> bool {
        self.hovering.contains(&element)
    }

    /// Zen's `toggleSidebar()` — user-forced show (`zen-user-show`).
    pub fn toggle_sidebar(&mut self) {
        self.user_show = !self.user_show;
    }

    pub fn user_show(&self) -> bool {
        self.user_show
    }

    /// Zen's `_clearAllHoverStates` — on window state changes.
    pub fn clear_hover_states(&mut self) {
        self.hovering.clear();
    }

    /// Zen's `_collapseTrackedElement` — the window was deactivated.
    pub fn on_window_deactivated(&mut self) {
        self.clear_hover_states();
    }

    // -- environment setters (Zen reads these from prefs/layout) ----------
    pub fn set_sidebar_is_on_right(&mut self, right: bool) {
        self.sidebar_is_on_right = right;
    }

    pub fn set_window_buttons_on_left(&mut self, left: bool) {
        self.window_buttons_on_left = left;
    }

    pub fn set_hides_tabs_toolbar(&mut self, hides: bool) {
        self.hides_tabs_toolbar = hides;
    }

    pub fn set_sidebar_expanded(&mut self, expanded: bool) {
        self.sidebar_expanded = expanded;
    }

    pub fn set_hide_tabbar(&mut self, value: bool) {
        self.hide_tabbar = value;
    }

    pub fn set_hide_toolbar(&mut self, value: bool) {
        self.hide_toolbar = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_flips_preference() {
        let mut cm = CompactMode::new();
        assert!(!cm.preference());
        assert!(cm.set_preference(true));
        assert!(cm.preference());
        assert!(!cm.set_preference(false));
        assert!(!cm.preference());
        assert_eq!(cm.toggle(), true);
        assert_eq!(cm.toggle(), false);
    }

    #[test]
    fn layout_vetoes_preference() {
        let mut cm = CompactMode::new();
        cm.set_hides_tabs_toolbar(true);
        assert!(!cm.should_be_compact());
        assert!(!cm.set_preference(true)); // veto
        assert!(!cm.preference());
        assert!(cm.can_hide_sidebar()); // hide-tabbar(true) || single toolbar
        assert!(!cm.can_hide_toolbar()); // hide-toolbar(false)
    }

    #[test]
    fn check_illegal_state_repairs_when_possible() {
        let mut cm = CompactMode::new();
        cm.set_sidebar_expanded(false);
        cm.set_window_buttons_on_left(true);
        cm.set_sidebar_is_on_right(false); // illegal state
        cm.set_hide_tabbar(false); // can_hide_sidebar() = false
        cm.set_hide_toolbar(true); // can_hide_toolbar() = true
        // Illegal: repair sets hide-tabbar.
        assert!(cm.check_illegal_state());
        assert!(cm.can_hide_sidebar()); // repaired
        assert!(cm.hide_tabbar);
    }

    #[test]
    fn check_illegal_state_true_when_not_repairable() {
        let mut cm = CompactMode::new();
        cm.set_sidebar_expanded(false);
        cm.set_window_buttons_on_left(true);
        cm.set_sidebar_is_on_right(false); // illegal state
        cm.set_hide_tabbar(false);
        cm.set_hide_toolbar(false); // can_hide_toolbar() = false
        assert!(cm.check_illegal_state());
        assert!(!cm.hide_tabbar); // nothing to repair
    }

    #[test]
    fn check_illegal_state_false_when_expanded() {
        let mut cm = CompactMode::new();
        cm.set_sidebar_expanded(true);
        cm.set_window_buttons_on_left(true);
        cm.set_sidebar_is_on_right(true);
        assert!(!cm.check_illegal_state());
    }

    #[test]
    fn hover_state_machine() {
        let mut cm = CompactMode::new();
        assert!(!cm.is_hovering(Hoverable::Sidebar));
        cm.enter(Hoverable::Sidebar);
        assert!(cm.is_hovering(Hoverable::Sidebar));
        assert!(!cm.is_hovering(Hoverable::Toolbar));
        cm.enter(Hoverable::Toolbar);
        cm.leave(Hoverable::Sidebar);
        assert!(!cm.is_hovering(Hoverable::Sidebar));
        assert!(cm.is_hovering(Hoverable::Toolbar));
        // Window deactivate clears hover.
        cm.on_window_deactivated();
        assert!(!cm.is_hovering(Hoverable::Toolbar));
    }

    #[test]
    fn user_show_toggles() {
        let mut cm = CompactMode::new();
        assert!(!cm.user_show());
        cm.toggle_sidebar();
        assert!(cm.user_show());
        cm.toggle_sidebar();
        assert!(!cm.user_show());
    }
}
