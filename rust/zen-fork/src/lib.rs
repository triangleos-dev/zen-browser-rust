//! `zen_fork` — the Zen Browser browser shell rewritten in Rust.
//!
//! Zen Browser (github.com/zen-browser/desktop) is a Firefox fork whose
//! unique logic lives in a handful of places; this crate rewrites those in
//! Rust:
//!
//! | Zen (Mozilla JS/TS over Gecko) | Here |
//! |---|---|
//! | `nsZenWorkspace` / `nsZenWorkspaces` (`src/zen/spaces/`) | [`workspace::Workspaces`] |
//! | Vertical tab strip (`gZenVerticalTabsManager`) | [`vertical_tabs::VerticalTabs`] |
//! | `gZenCompactModeManager` (`src/zen/compact-mode/`) | [`compact_mode::CompactMode`] |
//! | `nsZenGlanceManager` (`src/zen/glance/`) | [`glance::Glance`] |
//! | `nsZenViewSplitter` (`src/zen/split-view/`) | [`split_view::SplitView`] |
//! | `prefs/zen/*.yaml` | [`prefs::Prefs`] |
//! | `Tab` objects (`gBrowser.tabs`) | [`tab::Tab`] |
//!
//! `shell::Shell` ties them together and is the entry point of the rewrite:
//! it owns the tab list, the workspace store, the vertical strip, compact
//! mode, Glance and split view, and a [`prefs::Prefs`] set.
//!
//! See `REWRITE_PLAN.md` at the repository root for the engine choice
//! (Zephium/Servo), the roadmap and the honest limits of this session.

pub mod compact_mode;
pub mod glance;
pub mod prefs;
pub mod shell;
pub mod split_view;
pub mod tab;
pub mod vertical_tabs;
pub mod workspace;

pub use shell::Shell;
