//! Split view — Zen's `nsZenViewSplitter` (`gZenViewSplitter`): groups of
//! up to `MAX_TABS` tabs shown side by side, laid out in a tree of nodes
//! with percentage sizes (`nsSplitNode`).

use crate::tab::TabId;

/// Zen's `nsZenViewSplitter.MAX_TABS`: at most four tabs per split group.
pub const MAX_TABS: usize = 4;

/// Split orientation (`nsSplitNode.direction`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Direction {
    /// Tabs side by side (Zen's `"row"`).
    #[default]
    Row,
    /// Tabs stacked (Zen's `"column"`).
    Column,
}

/// The grid type a group was created with (`splitTabs`' `gridType`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GridType {
    /// "vsep" — tabs in a row separated by a vertical rule.
    #[default]
    Vsep,
    /// "hsep" — tabs in a column separated by horizontal rules.
    Hsep,
    /// "grid" — rows of column pairs.
    Grid,
}

/// A node of the split layout tree (`nsSplitNode`/`nsSplitLeafNode`).
#[derive(Clone, Debug, PartialEq)]
pub struct SplitNode {
    /// `None` for leaf nodes.
    pub direction: Option<Direction>,
    /// Children — empty for leaves.
    pub children: Vec<SplitNode>,
    /// The leaf's tab.
    pub tab: Option<TabId>,
    /// `sizeInParent`: the node's share of its parent, in percent.
    pub size_in_parent: f64,
}

impl SplitNode {
    pub fn leaf(tab: TabId, size_in_parent: f64) -> Self {
        SplitNode { direction: None, children: Vec::new(), tab: Some(tab), size_in_parent }
    }

    pub fn branch(direction: Direction, size_in_parent: f64, children: Vec<SplitNode>) -> Self {
        SplitNode { direction: Some(direction), children, tab: None, size_in_parent }
    }

    pub fn is_leaf(&self) -> bool {
        self.direction.is_none()
    }

    /// All tabs in the subtree, in order (`nsSplitNode.tabs()`).
    pub fn tabs(&self) -> Vec<TabId> {
        if self.is_leaf() {
            return self.tab.map(|t| vec![t]).unwrap_or_default();
        }
        self.children.iter().flat_map(|c| c.tabs()).collect()
    }
}

/// A split group (`splitTabs`' `splitData`).
#[derive(Clone, Debug, PartialEq)]
pub struct SplitGroup {
    /// Zen's `groupId`.
    pub id: u64,
    /// `group.tabs`.
    pub tabs: Vec<TabId>,
    /// `group.gridType`.
    pub grid_type: GridType,
    /// `group.layoutTree`.
    pub layout_tree: SplitNode,
}

/// The split view — `nsZenViewSplitter._data` + `currentView`.
#[derive(Clone, Debug, Default)]
pub struct SplitView {
    groups: Vec<SplitGroup>,
    /// `currentView`: the active group's index, or `None` when inactive.
    current_view: Option<usize>,
    next_group_id: u64,
}

impl SplitView {
    pub fn new() -> Self {
        Self::default()
    }

    /// `splitTabs(tabs, gridType, initialIndex, { activate })`.
    ///
    /// Mirrors Zen: duplicates are removed (Zen's
    /// `.filter((t, i, arr) => arr.indexOf(t) === i)`), and groups are
    /// limited to `MAX_TABS` (`if (tabs.length < 2 || tabs.length > this.MAX_TABS) return`).
    pub fn split_tabs(
        &mut self,
        tabs: Vec<TabId>,
        grid_type: GridType,
        activate: bool,
    ) -> Option<&SplitGroup> {
        // Dedupe, keeping the first occurrence order (Zen's `indexOf` filter).
        let mut ids: Vec<TabId> = Vec::new();
        for t in tabs {
            if !ids.contains(&t) {
                ids.push(t);
            }
        }
        if ids.len() < 2 || ids.len() > MAX_TABS {
            return None;
        }
        self.next_group_id += 1;
        let group = SplitGroup {
            id: self.next_group_id,
            layout_tree: Self::calculate_layout_tree(&ids, grid_type),
            tabs: ids,
            grid_type,
        };
        self.groups.push(group);
        let idx = self.groups.len() - 1;
        if activate {
            self.activate_group(idx);
        }
        self.groups.get(idx)
    }

    /// `activateSplitView(splitData, reset)`.
    pub fn activate_group(&mut self, idx: usize) -> Option<&SplitGroup> {
        if idx >= self.groups.len() {
            return None;
        }
        self.current_view = Some(idx);
        self.groups.get(idx)
    }

    /// `deactivateCurrentSplitView`.
    pub fn deactivate_current(&mut self) {
        self.current_view = None;
    }

    pub fn current_group_idx(&self) -> Option<usize> {
        self.current_view
    }

    pub fn current_group(&self) -> Option<&SplitGroup> {
        self.current_view.and_then(|i| self.groups.get(i))
    }

    pub fn is_active(&self) -> bool {
        self.current_view.is_some()
    }

    pub fn groups(&self) -> &[SplitGroup] {
        &self.groups
    }

    /// `group.tabs.includes(tab)` + the tab's index in the group.
    pub fn group_of(&self, tab: TabId) -> Option<(usize, usize)> {
        self.groups
            .iter()
            .enumerate()
            .find_map(|(gi, g)| g.tabs.iter().position(|&t| t == tab).map(|ti| (gi, ti)))
    }

    /// `updateSplitView(tab)` — activates the group a tab is in; if the tab
    /// is in no group, the current split is deactivated.
    pub fn update_split_view(&mut self, tab: TabId) -> bool {
        match self.group_of(tab) {
            Some((gi, _)) => {
                self.activate_group(gi);
                true
            }
            None => {
                self.deactivate_current();
                false
            }
        }
    }

    /// `removeTabFromGroup(tab)` — Zen: groups with fewer than three tabs
    /// are un-split as a whole; otherwise the tab is removed and the layout
    /// is recomputed (`removeNode` + `applyGridLayout`).
    pub fn remove_tab_from_group(&mut self, tab: TabId) -> bool {
        let (gi, ti) = match self.group_of(tab) {
            Some(x) => x,
            None => return false,
        };
        if self.groups[gi].tabs.len() < 3 {
            // Unsplit: drop the group (Zen: `removeGroup`).
            self.groups.remove(gi);
            match self.current_view {
                Some(v) if v == gi => self.current_view = None,
                Some(v) if v > gi => self.current_view = Some(v - 1),
                _ => {}
            }
            return true;
        }
        let group = &mut self.groups[gi];
        group.tabs.remove(ti);
        group.layout_tree = Self::calculate_layout_tree(&group.tabs, group.grid_type);
        true
    }

    /// `addTabToSplit(group, tab)` — steals size: every child shrinks by
    /// `n/(n+1)` and the new leaf takes the rest (Zen's `addTabToSplit`).
    pub fn add_tab_to_group(&mut self, gi: usize, tab: TabId) -> bool {
        let group = &mut self.groups[gi];
        if group.tabs.contains(&tab) || group.tabs.len() >= MAX_TABS {
            return false;
        }
        fn resize(node: &mut SplitNode, reduce: f64) {
            node.size_in_parent *= reduce;
            for c in node.children.iter_mut() {
                resize(c, reduce);
            }
        }
        let reduce = group.tabs.len() as f64 / (group.tabs.len() + 1) as f64;
        resize(&mut group.layout_tree, reduce);
        group.layout_tree.children.push(SplitNode::leaf(tab, (1.0 - reduce) * 100.0));
        group.tabs.push(tab);
        true
    }

    /// Zen's `calculateLayoutTree(tabs, gridType)`.
    pub fn calculate_layout_tree(tabs: &[TabId], grid_type: GridType) -> SplitNode {
        match grid_type {
            GridType::Vsep => {
                let n = tabs.len().max(1);
                SplitNode::branch(
                    Direction::Row,
                    100.0,
                    tabs.iter().map(|&t| SplitNode::leaf(t, 100.0 / n as f64)).collect(),
                )
            }
            GridType::Hsep => {
                let n = tabs.len().max(1);
                SplitNode::branch(
                    Direction::Column,
                    100.0,
                    tabs.iter().map(|&t| SplitNode::leaf(t, 100.0 / n as f64)).collect(),
                )
            }
            GridType::Grid if tabs.len() == 2 => SplitNode::branch(
                Direction::Row,
                100.0,
                tabs.iter().map(|&t| SplitNode::leaf(t, 50.0)).collect(),
            ),
            GridType::Grid => {
                // A row of column pairs: pairs of two tabs stacked vertically.
                let n = tabs.len();
                let row_width = 100.0 / n.div_ceil(2) as f64;
                let mut columns = Vec::new();
                let mut i = 0;
                while i < n {
                    let mut kids = vec![SplitNode::leaf(tabs[i], 50.0)];
                    if i + 1 < n {
                        kids.push(SplitNode::leaf(tabs[i + 1], 50.0));
                    } else {
                        kids[0].size_in_parent = 100.0; // odd tail column: full width
                    }
                    columns.push(SplitNode::branch(Direction::Column, row_width, kids));
                    i += 2;
                }
                SplitNode::branch(Direction::Row, 100.0, columns)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_two_tabs_vsep() {
        let mut sv = SplitView::new();
        assert!(sv.groups().is_empty());
        let g = sv.split_tabs(vec![1, 2], GridType::Vsep, true).unwrap().clone();
        assert_eq!(g.id, 1);
        assert_eq!(g.tabs, vec![1, 2]);
        assert_eq!(g.grid_type, GridType::Vsep);
        assert!(sv.is_active());
        assert_eq!(sv.current_group_idx(), Some(0));
        // Layout: row of two leaves at 50/50.
        assert_eq!(g.layout_tree.direction, Some(Direction::Row));
        let kids = &g.layout_tree.children;
        assert_eq!(kids.len(), 2);
        assert_eq!(kids[0].tab, Some(1));
        assert_eq!(kids[0].size_in_parent, 50.0);
        assert_eq!(kids[1].tab, Some(2));
        assert_eq!(kids[1].size_in_parent, 50.0);
    }

    #[test]
    fn split_rejects_invalid_counts() {
        let mut sv = SplitView::new();
        assert!(sv.split_tabs(vec![1], GridType::Vsep, true).is_none());
        assert!(sv.split_tabs(vec![1, 2, 3, 4, 5], GridType::Vsep, true).is_none());
        assert!(sv.split_tabs(vec![1, 1], GridType::Vsep, true).is_none()); // dedupe → 1 tab
        // Zen's dedupe keeps the first occurrences: [1, 1, 2] → [1, 2].
        let g = sv.split_tabs(vec![1, 1, 2], GridType::Vsep, true).unwrap();
        assert_eq!(g.tabs, vec![1, 2]);
        assert_eq!(sv.groups().len(), 1);
        assert!(sv.is_active());
    }

    #[test]
    fn split_four_tabs_grid() {
        let mut sv = SplitView::new();
        let g = sv.split_tabs(vec![1, 2, 3, 4], GridType::Grid, true).unwrap();
        assert_eq!(g.tabs, vec![1, 2, 3, 4]);
        let root = &g.layout_tree;
        assert_eq!(root.direction, Some(Direction::Row));
        assert_eq!(root.children.len(), 2); // two columns
        assert_eq!(root.children[0].children.len(), 2);
        assert_eq!(root.children[1].children.len(), 2);
        assert_eq!(g.layout_tree.tabs(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn layout_trees_match_zen_shapes() {
        // vsep: row of leaves, equal sizes.
        let t = SplitView::calculate_layout_tree(&[1, 2, 3], GridType::Vsep);
        assert_eq!(t.direction, Some(Direction::Row));
        assert_eq!(t.children.len(), 3);
        assert!((t.children[0].size_in_parent - 100.0 / 3.0).abs() < 1e-9);
        // hsep: column of leaves.
        let t = SplitView::calculate_layout_tree(&[1, 2], GridType::Hsep);
        assert_eq!(t.direction, Some(Direction::Column));
        assert_eq!(t.children.len(), 2);
        assert!((t.children[0].size_in_parent - 50.0).abs() < 1e-9);
    }

    #[test]
    fn activate_and_deactivate_groups() {
        let mut sv = SplitView::new();
        sv.split_tabs(vec![1, 2], GridType::Vsep, true);
        let g2 = sv.split_tabs(vec![3, 4], GridType::Vsep, true).unwrap().clone();
        assert_eq!(sv.current_group_idx(), Some(1));
        assert_eq!(sv.current_group().map(|g| g.id), Some(g2.id));
        sv.deactivate_current();
        assert!(!sv.is_active());
    }

    #[test]
    fn update_split_view() {
        let mut sv = SplitView::new();
        sv.split_tabs(vec![1, 2], GridType::Vsep, true);
        assert!(sv.update_split_view(1)); // activates the group
        assert!(!sv.update_split_view(3)); // tab not in a split → deactivate
        assert!(!sv.is_active());
    }

    #[test]
    fn remove_tab_from_group_unsplits_two_tabs() {
        let mut sv = SplitView::new();
        sv.split_tabs(vec![1, 2], GridType::Vsep, true);
        assert!(sv.remove_tab_from_group(1));
        assert!(sv.groups().is_empty());
        assert!(!sv.is_active());
        assert!(!sv.remove_tab_from_group(1)); // already gone
    }

    #[test]
    fn remove_tab_from_group_keeps_three() {
        let mut sv = SplitView::new();
        sv.split_tabs(vec![1, 2, 3], GridType::Vsep, true);
        assert!(sv.remove_tab_from_group(1));
        let g = sv.groups().first().unwrap();
        assert_eq!(g.tabs, vec![2, 3]);
        assert!(sv.is_active());
    }

    #[test]
    fn add_tab_to_group() {
        let mut sv = SplitView::new();
        sv.split_tabs(vec![1, 2], GridType::Vsep, true);
        assert!(sv.add_tab_to_group(0, 3));
        assert_eq!(sv.groups()[0].tabs, vec![1, 2, 3]);
        // Sizes sum to ~100.
        let root = &sv.groups()[0].layout_tree;
        let sum: f64 = root.children.iter().map(|c| c.size_in_parent).sum();
        assert!((sum - 100.0).abs() < 1e-6, "sum = {sum}");
        // No duplicates; capped at MAX_TABS.
        assert!(!sv.add_tab_to_group(0, 3));
        assert!(sv.add_tab_to_group(0, 4));
        assert!(!sv.add_tab_to_group(0, 5));
        assert_eq!(sv.groups()[0].tabs.len(), MAX_TABS);
    }
}
