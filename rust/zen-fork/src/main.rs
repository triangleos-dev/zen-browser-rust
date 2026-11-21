//! `zen-fork` — a Rust rewrite of the Zen Browser shell: workspaces,
//! vertical tabs, compact mode, Glance and split view (see the crate docs
//! and `REWRITE_PLAN.md`).

use zen_fork::tab::TabId;
use zen_fork::Shell;

const USAGE: &str = "\
zen-fork — the Zen Browser shell, in Rust

Usage:
  zen-fork workspace list                     List workspaces (the active one is marked)
  zen-fork workspace new <name>               Create a workspace
  zen-fork workspace switch <name>            Activate a workspace by name or id
  zen-fork workspace reorder <id> <position>  Move a workspace in the list
  zen-fork workspace remove <id>              Remove a workspace
  zen-fork tab add <url> [--pinned]           Add a tab to the active workspace
  zen-fork tab list                           List the active workspace's tabs (pinned first)
  zen-fork tab select <id>                    Select a tab
  zen-fork tab remove <id>                    Remove a tab
  zen-fork split <id1> <id2> [id3] [id4]      Split tabs side by side
  zen-fork split list                         List split groups
  zen-fork glance open <url> [--parent <id>]  Preview a URL over a tab (Glance)
  zen-fork glance close                       Close the current Glance
  zen-fork glance list                        Show the current Glance
  zen-fork prefs show                         Show the default prefs
  zen-fork demo                               Run a scripted end-to-end pass over the shell
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut shell = Shell::new();
    match args.first().map(String::as_str) {
        Some("workspace") => cmd_workspace(&mut shell, &args[1..]),
        Some("tab") => cmd_tab(&mut shell, &args[1..]),
        Some("split") => cmd_split(&mut shell, &args[1..]),
        Some("glance") => cmd_glance(&mut shell, &args[1..]),
        Some("prefs") => cmd_prefs(&shell, &args[1..]),
        Some("demo") => cmd_demo(&mut shell),
        Some("help") | None => print!("{}", USAGE),
        Some(other) => println!("unknown command: {}\n\n{}", other, USAGE),
    }
}

fn cmd_workspace(shell: &mut Shell, args: &[String]) {
    match args.first().map(String::as_str) {
        Some("list") | None => {
            for ws in shell.workspaces().spaces() {
                let marker = if shell.workspaces().is_active(&ws.id) { "*" } else { " " };
                println!("{} {:<14} {}", marker, ws.name, ws.id);
            }
        }
        Some("new") => {
            let Some(name) = args.get(1) else {
                println!("usage: zen-fork workspace new <name>");
                return;
            };
            let id = shell.add_workspace(name.clone());
            println!("created workspace {} ({})", name, id);
        }
        Some("switch") => {
            let Some(name) = args.get(1) else {
                println!("usage: zen-fork workspace switch <name>");
                return;
            };
            match shell.workspaces().resolve(name).map(str::to_string) {
                Some(id) => {
                    if shell.switch_workspace(&id) {
                        println!("switched to {} ({})", name, id);
                    } else {
                        println!("cannot switch to {} ({})", name, id);
                    }
                }
                None => println!("no workspace named {}", name),
            }
        }
        Some("reorder") => {
            let (Some(id), Some(pos)) =
                (args.get(1), args.get(2).and_then(|s| s.parse::<usize>().ok()))
            else {
                println!("usage: zen-fork workspace reorder <id> <position>");
                return;
            };
            match shell.workspaces_mut().reorder(id, pos) {
                Ok(()) => println!("reordered {} to position {}", id, pos),
                Err(e) => println!("error: {:?}", e),
            }
        }
        Some("remove") => {
            let Some(id) = args.get(1) else {
                println!("usage: zen-fork workspace remove <id>");
                return;
            };
            match shell.workspaces_mut().remove(id) {
                Ok(()) => println!("removed workspace {}", id),
                Err(e) => println!("error: {:?}", e),
            }
        }
        _ => print!("{}", USAGE),
    }
}

fn cmd_tab(shell: &mut Shell, args: &[String]) {
    match args.first().map(String::as_str) {
        Some("add") => {
            let url = args.get(1).cloned().unwrap_or_else(|| "about:blank".to_string());
            let pinned = args.iter().any(|a| a == "--pinned");
            let id =
                if pinned { shell.add_pinned_tab(url.clone()) } else { shell.add_tab(url.clone()) };
            println!("{}\t{}", id, url);
        }
        Some("list") => {
            // The strip order: pinned first, then unpinned (Zen's vertical
            // strip — `#organizeTabsToWorkspaceSections`).
            for id in shell.vertical().iter() {
                let Some(tab) = shell.tab(id) else { continue };
                if tab.workspace_id.as_deref() != shell.workspaces().active_id() {
                    continue; // the strip holds the active workspace's tabs
                }
                let pinned = if tab.pinned { "[pinned] " } else { "" };
                let sel = if shell.selected() == Some(id) { "*" } else { "" };
                println!("{:>4}\t{}{}\t{}", id, pinned, tab.url, sel);
            }
        }
        Some("select") => {
            let Some(id) = args.get(1).and_then(|s| s.parse::<TabId>().ok()) else {
                println!("usage: zen-fork tab select <id>");
                return;
            };
            if shell.select(id) {
                println!("selected tab {}", id);
            } else {
                println!("no tab {}", id);
            }
        }
        Some("remove") => {
            let Some(id) = args.get(1).and_then(|s| s.parse::<TabId>().ok()) else {
                println!("usage: zen-fork tab remove <id>");
                return;
            };
            if shell.remove_tab(id) {
                println!("removed tab {}", id);
            } else {
                println!("no tab {}", id);
            }
        }
        _ => print!("{}", USAGE),
    }
}

fn cmd_split(shell: &mut Shell, args: &[String]) {
    match args.first().map(String::as_str) {
        Some("list") => {
            for g in shell.split().groups() {
                let tabs: Vec<String> = g.tabs.iter().map(|t| t.to_string()).collect();
                println!("split #{}: [{:?}] {}", g.id, g.grid_type, tabs.join(", "));
            }
        }
        Some("remove") => {
            let Some(id) = args.get(1).and_then(|s| s.parse::<TabId>().ok()) else {
                println!("usage: zen-fork split remove <tab>");
                return;
            };
            if shell.split_remove_tab(id) {
                println!("removed tab {} from its split", id);
            } else {
                println!("tab {} is not in a split", id);
            }
        }
        _ if args.len() >= 2 => {
            let ids: Vec<TabId> =
                args[1..].iter().filter_map(|s| s.parse::<TabId>().ok()).collect();
            match shell.split_tabs(&ids) {
                Some(g) => println!("split group #{}: [{:?}] {:?}", g.id, g.grid_type, g.tabs),
                None => println!("cannot split: need 2 to 4 tabs (got {})", ids.len()),
            }
        }
        _ => print!("{}", USAGE),
    }
}

fn cmd_glance(shell: &mut Shell, args: &[String]) {
    match args.first().map(String::as_str) {
        Some("open") => {
            let url = args.get(1).cloned().unwrap_or_else(|| "about:blank".to_string());
            let parent = args
                .iter()
                .position(|a| a == "--parent")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse::<TabId>().ok())
                .or(shell.selected());
            match parent.filter(|p| shell.tab(*p).is_some()) {
                Some(p) => match shell.open_glance(p, url) {
                    Some(gid) => println!("glance {} open over tab {}", gid, p),
                    None => println!("a glance is already open"),
                },
                None => println!("no tab given and no tab selected"),
            }
        }
        Some("close") => {
            if shell.close_glance() {
                println!("glance closed");
            } else {
                println!("no open glance");
            }
        }
        Some("list") => match shell.glance_current() {
            Some(gid) => match shell.glance(gid) {
                Some(s) => println!("glance {} over tab {} — {}", s.id, s.parent, s.url),
                None => println!("glance {} has no state", gid),
            },
            None => println!("no open glance"),
        },
        _ => print!("{}", USAGE),
    }
}

fn cmd_prefs(shell: &Shell, args: &[String]) {
    match args.first().map(String::as_str) {
        Some("show") | None => {
            for (name, value) in shell.prefs().entries() {
                println!("{} = {}", name, value);
            }
        }
        _ => print!("{}", USAGE),
    }
}

/// A scripted end-to-end pass over the shell — the Zen Browser UX models
/// (workspaces, vertical tabs, split view, Glance, prefs) over the `Tab`
/// list. (State persistence to a real profile is Phase 3, engine
/// integration; each invocation of the other commands builds a fresh `Shell`.)
fn cmd_demo(shell: &mut Shell) {
    println!("== Zen shell demo (the Zen Browser UX, modeled in Rust) ==");

    let ws = shell.add_workspace("Work");
    println!();
    println!("-- workspace list --");
    cmd_workspace(shell, &["list".into()]);

    let a = shell.add_tab("https://github.com/zen-browser");
    let b = shell.add_tab("https://zen-browser.app");
    let c = shell.add_pinned_tab("https://example.com");
    println!("pinned tab: {c} https://example.com");
    shell.select(b);
    println!();
    println!("-- tab list (active workspace) --");
    cmd_tab(shell, &["list".into()]);

    println!();
    println!("-- switch to \"Work\" --");
    cmd_workspace(shell, &["switch".into(), ws.clone()]);
    println!("-- tab list (active workspace) --");
    cmd_tab(shell, &["list".into()]);

    println!();
    println!("-- split tabs {a} {b} --");
    let group = shell.split_tabs(&[a, b]);
    println!("split group: {:?}", group.as_ref().map(|g| g.tabs.clone()));
    cmd_split(shell, &["list".into()]);

    println!();
    println!("-- glance --");
    if let Some(p) = shell.selected() {
        match shell.open_glance(p, "https://zen-browser.app/zen-glance") {
            Some(gid) => println!("glance {} open over tab {}", gid, p),
            None => println!("a glance is already open"),
        }
    }
    cmd_glance(shell, &["list".into()]);

    println!();
    println!("-- prefs --");
    cmd_prefs(shell, &[]);
}
