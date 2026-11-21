# zen-fork

A Rust rewrite of the Zen Browser browser shell: workspaces, vertical tabs,
compact mode, Glance and split view — over the tab model (`Tab`) Zen keeps
in `gBrowser`.

This is the Rust half of the fork: the model of Zen's unique logic lives
here; see the crate docs in `src/lib.rs` for the module map and
`REWRITE_PLAN.md` at the repository root for the engine choice, roadmap and
honest limits.

## Build and test

```
cargo build   # builds the zen-fork binary + the library
cargo test    # unit tests for every module
```

## The shell, from the command line

The CLI models the shell state over the modules; each invocation builds a
fresh `Shell` (persistence to a real profile is Phase 3 — engine
integration):

```
zen-fork workspace list                     # List workspaces (active marked)
zen-fork workspace new <name>               # Create a workspace
zen-fork workspace switch <name>            # Activate a workspace
zen-fork workspace reorder <id> <position> # Move a workspace in the list
zen-fork workspace remove <id>              # Remove a workspace
zen-fork tab add <url> [--pinned]           # Add a tab to the active workspace
zen-fork tab list                           # List the active workspace's tabs (pinned first)
zen-fork tab select <id>                    # Select a tab
zen-fork tab remove <id>                    # Remove a tab
zen-fork split <id1> <id2> [id3] [id4]      # Split tabs side by side
zen-fork split list                         # List split groups
zen-fork glance open <url> [--parent <id>]  # Preview a URL over a tab (Glance)
zen-fork glance close                       # Close the current Glance
zen-fork glance list                        # Show the current Glance
zen-fork prefs show                         # Show the default prefs
zen-fork demo                               # Scripted end-to-end pass over the shell
```

## Module map

| Zen (Mozilla JS/TS over Gecko) | Here |
|---|---|
| `nsZenWorkspace` / `nsZenWorkspaces` (`src/zen/spaces/`) | `workspace::Workspaces` |
| Vertical tab strip (`gZenVerticalTabsManager`) | `vertical_tabs::VerticalTabs` |
| `gZenCompactModeManager` (`src/zen/compact-mode/`) | `compact_mode::CompactMode` |
| `nsZenGlanceManager` (`src/zen/glance/`) | `glance::Glance` |
| `nsZenViewSplitter` (`src/zen/split-view/`) | `split_view::SplitView` |
| `prefs/zen/*.yaml` | `prefs::Prefs` |
| `Tab` objects (`gBrowser.tabs`) | `tab::Tab` |
| Zen glue over all of the above | `shell::Shell` |
