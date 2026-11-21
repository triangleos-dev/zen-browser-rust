# Zen Browser → Rust Rewrite: plan and status

Zen Browser (github.com/zen-browser/desktop) is a Firefox fork whose
unique logic lives in a handful of places: workspaces, the vertical tab
strip, compact mode, Glance, split view, Zen Mods, the urlbar, window sync,
etc. Its engine is Gecko/C++ (`engine/` after `npm run init`); Zen's own
code is mostly TypeScript/JavaScript over Gecko.

This document is the rewrite plan for a Rust fork. Status is tracked per
module; the Rust scaffold lives in `rust/zen-fork/`.

## What the fork is today

- **GitHub fork:** `triangleos-dev/zen-browser-rust` — created via
  `gh api repos/zen-browser/desktop/forks -f name=zen-browser-rust`
  (authenticated account: `triangleos-dev`). Synced with the source
  (`zen-browser/desktop`, dev HEAD `6acdd79a`).
- **The Rust rewrite** lives on branch `rust-rewrite` of the fork:
  `rust/zen-fork/` + `REWRITE_PLAN.md`, on top of the fork's source tree
  (commit `0fc7a190864a`, tree `b776fd38`). Local working copy of the
  scaffold: `/home/dmytro/zen-scaffold-staging/rust/zen-fork/`.
- **Local copy of the full fork tree:** the repo is ~5.9 GB; `git clone`
  through GitHub's smart-HTTP kept dying mid-transfer on this network and
  a codeload tarball download stalled, so get it by cloning the fork:
  `git clone https://github.com/triangleos-dev/zen-browser-rust.git`
  (or `--branch rust-rewrite` for the rewrite start).

## What "rewritten in Rust" means here

Zen's *unique browser logic* is rewritten in Rust in `rust/zen-fork/`
(crate `zen_fork`), module by module, mirroring the Zen sources:

| Rust module | Zen source | Status |
|---|---|---|
| `workspace.rs` (`Workspaces`) | `nsZenWorkspace` + `nsZenWorkspaces` (`src/zen/spaces/ZenSpaceManager.mjs`, `ZenSpace.mjs`) | ported |
| `vertical_tabs.rs` (`VerticalTabs`) | vertical tab strip (`gZenVerticalTabsManager`; `#organizeTabsToWorkspaceSections`) | ported |
| `compact_mode.rs` (`CompactMode`) | `gZenCompactModeManager` (`src/zen/compact-mode/ZenCompactMode.mjs`) | ported |
| `glance.rs` (`Glance`) | `nsZenGlanceManager` (`src/zen/glance/ZenGlanceManager.mjs`) | ported |
| `split_view.rs` (`SplitView`) | `nsZenViewSplitter` (`src/zen/split-view/ZenViewSplitter.mjs`) | ported |
| `prefs.rs` (`Prefs`) | `prefs/zen/*.yaml` (`workspaces.yaml`, `compact-mode.yaml`, `glance.yaml`, `split-view.yaml`, `view.yaml`) | ported |
| `tab.rs` (`Tab`) | `Tab` objects in `gBrowser` (Zen attributes: `zen-workspace-id`, `pinned`, `zen-glance-tab`, `zen-empty-tab`, `zen-essential`, `split-view`) | ported |
| `shell.rs` (`Shell`) | — (glue over the above) | ported |
| `main.rs` (`zen-fork` CLI) | — (demo surface over the shell) | ported |

The browsing engine itself is a dependency, not rewritten from scratch:

- **Primary: Zephium** (github.com/zephium-browser/Zephium) — a browser
  written in Rust; its Rust core (`crates/zephium-core`, `zephium-app`,
  `zephium-store`, …) owns tabs, storage, the ad blocker and the UI. Its
  product system (`docs/product-system.md`) includes "spaces/workspaces,
  tabs, splits, and navigation" and "horizontal, vertical, compact, and
  power-user browsing arrangements" — exactly the concepts this scaffold
  models. Its welcome flow even imports from Zen. A Zen fork on Zephium =
  Zen's features (workspaces, vertical tabs, compact mode, Glance, split
  view) ported to Zephium's engine and UI.
- **Alternative for Linux: Servo** (github.com/servo/servo — `servoshell`) —
  from-scratch Rust engine; the smallest serious from-scratch Rust browser.
- Other candidates surveyed: Gosub (engine), Falco (MIT from-scratch
  engine), Blitz, etc. (survey notes at the bottom.)

## Phases

1. **Fork the source** — done (GitHub fork + local copy).
2. **Rust scaffold of the shell** — in progress, landing this session:
   `rust/zen-fork/` builds, is unit-tested (`cargo test`) and runnable
   (`zen-fork demo`).
3. **Engine integration** — write the `ZenChrome` interface over the chosen
   engine (Servo's `WebView` or Zephium's window model) and route
   `Shell`/`Tab` operations to it.
4. **Chrome/UI on the engine** — vertical tab strip UI, workspace
   indicator/bar, compact-mode hover behavior, Glance overlay, split-grid
   layout.
5. **Feature parity** — Zen Mods, themes/gradients, essentials, window
   sync, urlbar, keyboard shortcuts, session store, bookmark containers
   (`ZenSpaceBookmarksStorage`), etc.
6. **Verification** — parity matrix Zen vs Rust; behavior-parity tests.

## Honest limits (what this session cannot deliver)

- A from-scratch Rust engine that passes the web-platform-tests suite is
  years of work for a team (Servo/Ladybird). This session delivers the
  fork + the Rust rewrite of Zen's unique logic + a plan.
- Gecko itself is millions of lines of C++ plus JavaScript; "rewrite the
  whole browser" is out of scope in any session.

## Survey notes

- Zen Browser (2026-11 dev): workspaces are "spaces" (`ZenSpaceManager`),
  with names, icons, container ids and per-space tab lists; switch,
  reorder, wrap-around navigation (`zen.workspaces.wrap-around-navigation`),
  natural scroll, swipe-to-switch; the visual is a vertical tab strip with
  pinned tabs first and a collapsible pins section.
- Compact mode: `gZenCompactModeManager` — toggle/hide-tabbar/hide-toolbar,
  hover expansion of sidebar/toolbar/window buttons, illegal-state repair,
  outside-window tracking (Linux lacks global mouse tracking — fixed-duration
  fallback there).
- Glance: `nsZenGlanceManager` — overlay tab drawn inside the parent tab;
  `openGlance` while one is open returns the current glance; detached
  (Little Window) variant; workspace fallback when the toolbar is hidden.
- Split view: `nsZenViewSplitter` — groups of up to 4 tabs, a row/column
  layout tree with percentage sizes, add/remove tab, `updateSplitView` on
  tab select, drag-drop split.
- Prefs: `zen.workspaces.*`, `zen.view.compact.*`, `zen.glance.*`,
  `zen.splitView.*`, `zen.view.*` — ported as `prefs::Prefs` with Zen's
  YAML defaults.

## Next steps (after this session)

1. Finish the scaffold: README + plan committed to the fork (`rust/zen-fork/`).
2. Pick the engine (Zephium vs Servo) and prototype the glue.
