//! The preference defaults of the Zen shell.
//!
//! These mirror the `prefs/zen/*.yaml` files of the original browser
//! (`prefs/zen/workspaces.yaml`, `compact-mode.yaml`, `glance.yaml`,
//! `split-view.yaml`, `view.yaml`), with the values code reads via
//! `Services.prefs.getBoolPref(...)` where the YAML does not define a
//! product default.

/// Workspace defaults — `prefs/zen/workspaces.yaml` (`zen.workspaces.*`).
pub mod workspaces {
    pub const ACTIVE: &str = ""; // zen.workspaces.active — starts with ""
    pub const CONTINUE_WHERE_LEFT_OFF: bool = false;
    pub const HIDE_DEFAULT_CONTAINER_INDICATOR: bool = true;
    pub const FORCE_CONTAINER_WORKSPACE: bool = false;
    pub const OPEN_NEW_TAB_IF_LAST_UNPINNED_TAB_IS_CLOSED: bool = false;
    pub const SWIPE_ACTIONS: bool = true;
    pub const SWIPE_ACTIONS_DELTA_MULTIPLIER: u64 = 100;
    pub const SWIPE_ACTIONS_EDGE_ACTIONS: bool = true;
    pub const SWITCH_ANIMATION_DURATION_MS: u64 = 250;
    pub const WRAP_AROUND_NAVIGATION: bool = true;
    pub const NATURAL_SCROLL: bool = false;
    pub const SCROLL_MODIFIER_KEY: &str = "ctrl";
    pub const SEPARATE_ESSENTIALS: bool = true;
    pub const DND_SWITCH_PADDING: u64 = 20;
    pub const DEBUG: bool = false;
}

/// Compact mode defaults — `prefs/zen/compact-mode.yaml` (`zen.view.compact.*`).
pub mod compact_mode {
    pub const HIDE_TABBAR: bool = true;
    pub const HIDE_TOOLBAR: bool = false;
    pub const TOOLBAR_FLASH_POPUP: bool = false;
    pub const TOOLBAR_FLASH_POPUP_DURATION_MS: u64 = 800;
    pub const TOOLBAR_HIDE_AFTER_HOVER_MS: u64 = 1000;
    pub const SIDEBAR_KEEP_HOVER_MS: u64 = 150;
    pub const OUTSIDE_WINDOW_EDGE_OFFSET_HORIZONTAL: u64 = 200;
    pub const OUTSIDE_WINDOW_EDGE_OFFSET_VERTICAL: u64 = 100;
    pub const ANIMATE_SIDEBAR: bool = true;
    pub const SHOW_SIDEBAR_AND_TOOLBAR_ON_HOVER: bool = true;
    pub const SHOW_BACKGROUND_TAB_TOAST: bool = true;
    pub const DEBUG: bool = false;
    pub const ENABLE_AT_STARTUP: bool = false;
}

/// Glance defaults — `prefs/zen/glance.yaml` (`zen.glance.*`).
pub mod glance {
    pub const ENABLED: bool = true;
    pub const ENABLE_CONTEXTMENU_SEARCH: bool = true;
    pub const OPEN_ESSENTIAL_EXTERNAL_LINKS: bool = true;
    pub const ACTIVATION_METHOD: &str = "alt"; // ctrl, alt, shift
    pub const ANIMATION_DURATION_MS: u64 = 350;
    pub const DEACTIVATE_DOCSHELL_DURING_ANIMATION: bool = false;
}

/// Split view defaults — `prefs/zen/split-view.yaml` (`zen.splitView.*`).
pub mod split_view {
    pub const ENABLE_TAB_DROP: bool = true;
    pub const ENABLE_TAB_CLICK_SPLIT: bool = true;
    pub const MIN_RESIZE_WIDTH: f64 = 7.0;
    pub const REARRANGE_HOVER_SIZE: f64 = 24.0;
    pub const ENABLE_DRAG_OVER_SPLIT: bool = true;
    pub const DRAG_OVER_SPLIT_DELAY_MS: u64 = 500;
    pub const DRAG_OVER_SPLIT_THRESHOLD: f64 = 40.0;
}

/// View defaults — `prefs/zen/view.yaml` (`zen.view.*`).
pub mod view {
    pub const SIDEBAR_EXPANDED: bool = true;
    pub const SHOW_SCROLL_TO_SELECTED_TAB: bool = true;
}

/// The full set of Zen shell preferences — a Rust counterpart of the YAML
/// pref files (`Prefs::default()` = the product defaults).
#[derive(Clone, Debug, PartialEq)]
pub struct Prefs {
    // -- Workspaces -------------------------------------------------------
    /// `zen.workspaces.active`: id of the active workspace.
    pub workspaces_active: Option<String>,
    pub workspaces_continue_where_left_off: bool,
    pub workspaces_hide_default_container_indicator: bool,
    pub workspaces_force_container_workspace: bool,
    pub workspaces_open_new_tab_if_last_unpinned_closed: bool,
    pub workspaces_swipe_actions: bool,
    pub workspaces_swipe_actions_delta_multiplier: u64,
    pub workspaces_swipe_actions_edge_actions: bool,
    pub workspaces_switch_animation_duration_ms: u64,
    pub workspaces_wrap_around_navigation: bool,
    pub workspaces_natural_scroll: bool,
    pub workspaces_scroll_modifier_key: String,
    pub workspaces_separate_essentials: bool,
    pub workspaces_dnd_switch_padding: u64,
    pub workspaces_debug: bool,

    // -- Compact mode ------------------------------------------------------
    /// `zen.view.compact.hide-tabbar`: the sidebar is collapsed.
    pub view_compact_hide_tabbar: bool,
    /// `zen.view.compact.hide-toolbar`: the toolbar is collapsed.
    pub view_compact_hide_toolbar: bool,
    pub view_compact_toolbar_flash_popup: bool,
    pub view_compact_toolbar_flash_popup_duration_ms: u64,
    pub view_compact_toolbar_hide_after_hover_ms: u64,
    pub view_compact_sidebar_keep_hover_ms: u64,
    pub view_compact_outside_window_edge_offset_horizontal: u64,
    pub view_compact_outside_window_edge_offset_vertical: u64,
    pub view_compact_animate_sidebar: bool,
    pub view_compact_show_sidebar_and_toolbar_on_hover: bool,
    pub view_compact_show_background_tab_toast: bool,
    pub view_compact_debug: bool,
    pub view_compact_enable_at_startup: bool,

    // -- Glance ----------------------------------------------------------------
    pub glance_enabled: bool,
    pub glance_enable_contextmenu_search: bool,
    pub glance_open_essential_external_links: bool,
    /// `zen.glance.activation-method`: "ctrl", "alt" or "shift".
    pub glance_activation_method: String,
    pub glance_animation_duration_ms: u64,
    pub glance_deactivate_docshell_during_animation: bool,

    // -- Split view --------------------------------------------------------------
    pub splitview_enable_tab_drop: bool,
    pub splitview_enable_tab_click_split: bool,
    pub splitview_min_resize_width: f64,
    pub splitview_rearrange_hover_size: f64,
    pub splitview_enable_drag_over_split: bool,
    pub splitview_drag_over_split_delay_ms: u64,
    pub splitview_drag_over_split_threshold: f64,

    // -- View ----------------------------------------------------------------------
    pub view_sidebar_expanded: bool,
    pub view_show_scroll_to_selected_tab: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs {
            workspaces_active: None,
            workspaces_continue_where_left_off: workspaces::CONTINUE_WHERE_LEFT_OFF,
            workspaces_hide_default_container_indicator:
                workspaces::HIDE_DEFAULT_CONTAINER_INDICATOR,
            workspaces_force_container_workspace: workspaces::FORCE_CONTAINER_WORKSPACE,
            workspaces_open_new_tab_if_last_unpinned_closed:
                workspaces::OPEN_NEW_TAB_IF_LAST_UNPINNED_TAB_IS_CLOSED,
            workspaces_swipe_actions: workspaces::SWIPE_ACTIONS,
            workspaces_swipe_actions_delta_multiplier: workspaces::SWIPE_ACTIONS_DELTA_MULTIPLIER,
            workspaces_swipe_actions_edge_actions: workspaces::SWIPE_ACTIONS_EDGE_ACTIONS,
            workspaces_switch_animation_duration_ms: workspaces::SWITCH_ANIMATION_DURATION_MS,
            workspaces_wrap_around_navigation: workspaces::WRAP_AROUND_NAVIGATION,
            workspaces_natural_scroll: workspaces::NATURAL_SCROLL,
            workspaces_scroll_modifier_key: workspaces::SCROLL_MODIFIER_KEY.to_string(),
            workspaces_separate_essentials: workspaces::SEPARATE_ESSENTIALS,
            workspaces_dnd_switch_padding: workspaces::DND_SWITCH_PADDING,
            workspaces_debug: workspaces::DEBUG,

            view_compact_hide_tabbar: compact_mode::HIDE_TABBAR,
            view_compact_hide_toolbar: compact_mode::HIDE_TOOLBAR,
            view_compact_toolbar_flash_popup: compact_mode::TOOLBAR_FLASH_POPUP,
            view_compact_toolbar_flash_popup_duration_ms:
                compact_mode::TOOLBAR_FLASH_POPUP_DURATION_MS,
            view_compact_toolbar_hide_after_hover_ms: compact_mode::TOOLBAR_HIDE_AFTER_HOVER_MS,
            view_compact_sidebar_keep_hover_ms: compact_mode::SIDEBAR_KEEP_HOVER_MS,
            view_compact_outside_window_edge_offset_horizontal:
                compact_mode::OUTSIDE_WINDOW_EDGE_OFFSET_HORIZONTAL,
            view_compact_outside_window_edge_offset_vertical:
                compact_mode::OUTSIDE_WINDOW_EDGE_OFFSET_VERTICAL,
            view_compact_animate_sidebar: compact_mode::ANIMATE_SIDEBAR,
            view_compact_show_sidebar_and_toolbar_on_hover:
                compact_mode::SHOW_SIDEBAR_AND_TOOLBAR_ON_HOVER,
            view_compact_show_background_tab_toast: compact_mode::SHOW_BACKGROUND_TAB_TOAST,
            view_compact_debug: compact_mode::DEBUG,
            view_compact_enable_at_startup: compact_mode::ENABLE_AT_STARTUP,

            glance_enabled: glance::ENABLED,
            glance_enable_contextmenu_search: glance::ENABLE_CONTEXTMENU_SEARCH,
            glance_open_essential_external_links: glance::OPEN_ESSENTIAL_EXTERNAL_LINKS,
            glance_activation_method: glance::ACTIVATION_METHOD.to_string(),
            glance_animation_duration_ms: glance::ANIMATION_DURATION_MS,
            glance_deactivate_docshell_during_animation:
                glance::DEACTIVATE_DOCSHELL_DURING_ANIMATION,

            splitview_enable_tab_drop: split_view::ENABLE_TAB_DROP,
            splitview_enable_tab_click_split: split_view::ENABLE_TAB_CLICK_SPLIT,
            splitview_min_resize_width: split_view::MIN_RESIZE_WIDTH,
            splitview_rearrange_hover_size: split_view::REARRANGE_HOVER_SIZE,
            splitview_enable_drag_over_split: split_view::ENABLE_DRAG_OVER_SPLIT,
            splitview_drag_over_split_delay_ms: split_view::DRAG_OVER_SPLIT_DELAY_MS,
            splitview_drag_over_split_threshold: split_view::DRAG_OVER_SPLIT_THRESHOLD,

            view_sidebar_expanded: view::SIDEBAR_EXPANDED,
            view_show_scroll_to_selected_tab: view::SHOW_SCROLL_TO_SELECTED_TAB,
        }
    }
}

impl Prefs {
    /// All prefs as `(name, value)` pairs, mirroring the YAML order — the
    /// shape `zen-fork prefs show` prints.
    pub fn entries(&self) -> Vec<(&'static str, String)> {
        vec![
            ("zen.workspaces.active", self.workspaces_active.clone().unwrap_or_default()),
            ("zen.workspaces.continue-where-left-off",
                self.workspaces_continue_where_left_off.to_string()),
            ("zen.workspaces.hide-default-container-indicator",
                self.workspaces_hide_default_container_indicator.to_string()),
            ("zen.workspaces.force-container-workspace",
                self.workspaces_force_container_workspace.to_string()),
            ("zen.workspaces.open-new-tab-if-last-unpinned-tab-is-closed",
                self.workspaces_open_new_tab_if_last_unpinned_closed.to_string()),
            ("zen.workspaces.swipe-actions", self.workspaces_swipe_actions.to_string()),
            ("zen.workspaces.swipe-actions.delta-multiplier",
                self.workspaces_swipe_actions_delta_multiplier.to_string()),
            ("zen.workspaces.swipe-actions.edge-actions",
                self.workspaces_swipe_actions_edge_actions.to_string()),
            ("zen.workspaces.switch-animation-duration",
                self.workspaces_switch_animation_duration_ms.to_string()),
            ("zen.workspaces.wrap-around-navigation",
                self.workspaces_wrap_around_navigation.to_string()),
            ("zen.workspaces.natural-scroll",
                self.workspaces_natural_scroll.to_string()),
            ("zen.workspaces.scroll-modifier-key",
                self.workspaces_scroll_modifier_key.clone()),
            ("zen.workspaces.separate-essentials",
                self.workspaces_separate_essentials.to_string()),
            ("zen.workspaces.dnd-switch-padding",
                self.workspaces_dnd_switch_padding.to_string()),
            ("zen.workspaces.debug", self.workspaces_debug.to_string()),
            ("zen.view.compact.hide-tabbar", self.view_compact_hide_tabbar.to_string()),
            ("zen.view.compact.hide-toolbar", self.view_compact_hide_toolbar.to_string()),
            ("zen.view.compact.toolbar-flash-popup",
                self.view_compact_toolbar_flash_popup.to_string()),
            ("zen.view.compact.toolbar-flash-popup.duration",
                self.view_compact_toolbar_flash_popup_duration_ms.to_string()),
            ("zen.view.compact.toolbar-hide-after-hover.duration",
                self.view_compact_toolbar_hide_after_hover_ms.to_string()),
            ("zen.view.compact.sidebar-keep-hover.duration",
                self.view_compact_sidebar_keep_hover_ms.to_string()),
            ("zen.view.compact.outside-window-edge-offset.horizontal",
                self.view_compact_outside_window_edge_offset_horizontal.to_string()),
            ("zen.view.compact.outside-window-edge-offset.vertical",
                self.view_compact_outside_window_edge_offset_vertical.to_string()),
            ("zen.view.compact.animate-sidebar", self.view_compact_animate_sidebar.to_string()),
            ("zen.view.compact.show-sidebar-and-toolbar-on-hover",
                self.view_compact_show_sidebar_and_toolbar_on_hover.to_string()),
            ("zen.view.compact.show-background-tab-toast",
                self.view_compact_show_background_tab_toast.to_string()),
            ("zen.view.compact.debug", self.view_compact_debug.to_string()),
            ("zen.view.compact.enable-at-startup",
                self.view_compact_enable_at_startup.to_string()),
            ("zen.glance.enabled", self.glance_enabled.to_string()),
            ("zen.glance.enable-contextmenu-search",
                self.glance_enable_contextmenu_search.to_string()),
            ("zen.glance.open-essential-external-links",
                self.glance_open_essential_external_links.to_string()),
            ("zen.glance.activation-method", self.glance_activation_method.clone()),
            ("zen.glance.animation-duration",
                self.glance_animation_duration_ms.to_string()),
            ("zen.glance.deactivate-docshell-during-animation",
                self.glance_deactivate_docshell_during_animation.to_string()),
            ("zen.splitView.enable-tab-drop",
                self.splitview_enable_tab_drop.to_string()),
            ("zen.splitView.enable-tab-click-split",
                self.splitview_enable_tab_click_split.to_string()),
            ("zen.splitView.min-resize-width",
                format!("{}", self.splitview_min_resize_width)),
            ("zen.splitView.rearrange-hover-size",
                format!("{}", self.splitview_rearrange_hover_size)),
            ("zen.splitView.enable-drag-over-split",
                self.splitview_enable_drag_over_split.to_string()),
            ("zen.splitView.drag-over-split-delayMC",
                self.splitview_drag_over_split_delay_ms.to_string()),
            ("zen.splitView.drag-over-split-threshold",
                format!("{}", self.splitview_drag_over_split_threshold)),
            ("zen.view.sidebar-expanded", self.view_sidebar_expanded.to_string()),
            ("zen.view.show-scroll-to-selected-tab",
                self.view_show_scroll_to_selected_tab.to_string()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_zen_pref_files() {
        let p = Prefs::default();
        assert!(p.workspaces_wrap_around_navigation);
        assert!(!p.workspaces_natural_scroll);
        assert_eq!(p.workspaces_scroll_modifier_key, "ctrl");
        assert!(p.workspaces_swipe_actions);
        assert!(p.view_compact_hide_tabbar);
        assert!(!p.view_compact_hide_toolbar);
        assert_eq!(p.view_compact_toolbar_flash_popup_duration_ms, 800);
        assert_eq!(p.glance_animation_duration_ms, 350);
        assert!(p.splitview_enable_tab_drop);
        assert_eq!(p.splitview_min_resize_width, 7.0);
        assert_eq!(p.splitview_rearrange_hover_size, 24.0);
        assert!(p.view_sidebar_expanded);
    }

    #[test]
    fn entries_are_complete() {
        let p = Prefs::default();
        let names: Vec<&str> = p.entries().iter().map(|(n, _)| *n).collect();
        for expected in [
            "zen.workspaces.active",
            "zen.workspaces.wrap-around-navigation",
            "zen.view.compact.hide-tabbar",
            "zen.view.compact.hide-toolbar",
            "zen.glance.enabled",
            "zen.splitView.min-resize-width",
            "zen.view.sidebar-expanded",
        ] {
            assert!(names.contains(&expected), "missing pref {expected}");
        }
    }
}
