//! Minimal tree that reflects a searchable filesystem using [Path]
//! and path [Components]
//!
//! Traversal is uni-directional: top -> down only.
//! Nodes contain a [BTreeMap] of their children and nothing else.
//!
//! [PathTreeTrait] should be in scope for proper use.
use std::ffi::OsString;
use std::path::{Component::{self, *}, Components, Path};
use std::collections::BTreeMap;

/// Minimal tree that reflects a searchable filesystem using [Path]
/// and path [Components]
///
/// Traversal is uni-directional: top -> down only.
/// Nodes contain a [BTreeMap] of their children and nothing else.
///
/// [PathTreeTrait] should be in scope for proper use.
#[derive(Debug)]
pub struct PathTree {
    root: bool,
    children: BTreeMap<OsString, Node>,
}

/// The general trait for [PathTree].
///
/// This should be in scope.
pub trait PathTreeTrait {
    /// Retrieves child tree map of this node
    fn children(&self) -> &BTreeMap<OsString, Node>;

    /// Determines whether a path exists at this node
    fn contains(&self, path: &Path) -> bool {
        self.find(path).is_some()
    }

    /// Searches the node for a path from
    fn find(&self, path: &Path) -> Option<TreeNode<'_>> {
        self.find_components(path.components())
    }

    /// Determines whether a path exists at this node
    fn contains_components(&self, components: Components<'_>) -> bool {
        self.find_components(components).is_some()
    }

    /// Searches the node for path components
    fn find_components(&self, components: Components<'_>) -> Option<TreeNode<'_>>;
}

impl PathTree {
    /// Creates a new [PathTree]
    pub fn new_relative() -> Self {
        Self {
            root: false,
            children: BTreeMap::new()
        }
    }

    /// Creates a new [PathTree]
    pub fn new_root() -> Self {
        Self {
            root: true,
            children: BTreeMap::new()
        }
    }

    /// Inserts a path into the tree
    pub fn insert(&mut self, path: &Path) {
        self.insert_components(path.components());
    }

    /// Inserts path components into the tree
    pub fn insert_components(&mut self, components: Components<'_>) {
        insert_os_strings(&mut self.children, components_to_os_strings(components))
    }

}

impl PathTreeTrait for PathTree {
    fn children(&self) -> &BTreeMap<OsString, Node> {
        &self.children
    }

    fn find_components(&self, mut components: Components<'_>) -> Option<TreeNode<'_>> {
        let name = loop {
            match components.next() {
                None => return Some(TreeNode::Base(self)),
                Some(Component::Prefix(prefix)) => break prefix.as_os_str(),
                Some(Component::Normal(os_str)) => break os_str,
                Some(Component::CurDir) => continue,
                Some(Component::ParentDir) => return None,
                Some(Component::RootDir) => if self.root {
                    continue
                } else {
                    return None
                }
            }
        };

        self.children.get(name)
            .map_or(None, |n| n.find_components(components))
    }

}

/// Non-root node of [PathTree]
#[derive(Debug)]
pub struct Node {
    children: BTreeMap<OsString, Node>,
}

impl Node {
    /// Splits a path into components and inserts any that are missing
    pub fn insert(&mut self, path: &Path) {
        self.insert_components(path.components());
    }

    /// Inserts any components that are missing
    pub fn insert_components(&mut self, components: Components<'_>) {
        insert_os_strings(&mut self.children, components_to_os_strings(components))
    }
}

impl PathTreeTrait for Node {
    fn children(&self) -> &BTreeMap<OsString, Node> {
        &self.children
    }

    fn find_components(&self, mut components: Components<'_>) -> Option<TreeNode<'_>> {
        let name = loop {
            match components.next() {
                None => return Some(TreeNode::Node(self)),
                Some(Component::Prefix(prefix)) => break prefix.as_os_str(),
                Some(Component::RootDir) => return None,
                Some(Component::CurDir) => continue,
                Some(Component::ParentDir) => return None,
                Some(Component::Normal(os_str)) => break os_str,
            }
        };

        self.children.get(name)
            .map_or(None, |n| n.find_components(components))
    }
}

/// Wrapper for a result returned by a [PathTree] operation.
///
/// Represents either the base node or one of its children.
#[derive(Debug)]
pub enum TreeNode<'t> {
    Base(&'t PathTree),
    Node(&'t Node)
}

impl<'t> PathTreeTrait for TreeNode<'t> {
    fn contains(&self, path: &Path) -> bool {
        match self {
            TreeNode::Base(base) =>  base.contains(path),
            TreeNode::Node(node) => node.contains(path),
        }
    }

    fn children(&self) -> &BTreeMap<OsString, Node> {
        match self {
            TreeNode::Base(base) => &base.children,
            TreeNode::Node(node) => &node.children,
        }
    }

    fn find(&self, path: &Path) -> Option<TreeNode<'_>> {
        match self {
            TreeNode::Base(base) =>  base.find(path),
            TreeNode::Node(node) => node.find(path),
        }
    }

    fn contains_components(&self, components: Components<'_>) -> bool {
        match self {
            TreeNode::Base(base) =>  base.contains_components(components),
            TreeNode::Node(node) => node.contains_components(components),
        }
    }

    fn find_components(&self, components: Components<'_>) -> Option<TreeNode<'_>> {
        match self {
            TreeNode::Base(base) =>  base.find_components(components),
            TreeNode::Node(node) => node.find_components(components),
        }
    }
}

/*fn components_vec(components: Components<'_>) -> Vec<Component<'_>> {
    let mut v = Vec::new();

    for c in components {
        match c {
            Prefix(_) => v.push(c),
            RootDir => {},
            CurDir => {}
            ParentDir => { v.pop(); }
            Normal(_) => v.push(c),
        }
    }

    v
}*/

fn components_to_os_strings(components: Components<'_>) -> Vec<OsString> {
    let mut v = Vec::new();

    for c in components {
        match c {
            Prefix(p) => v.push(p.as_os_str().to_owned()),
            RootDir => {},
            CurDir => {}
            ParentDir => { v.pop(); }
            Normal(n) => v.push(n.to_os_string()),
        }
    }

    v
}

fn insert_os_strings(treemap: &mut BTreeMap<OsString, Node>, mut os_strings: Vec<OsString>) {
    let (name, children) = match os_strings.len() {
        n if n < 1 => return,
        n if n == 1 => (os_strings.pop().unwrap(), Vec::new()),
        n if n > 1 =>  {
            let children = os_strings.split_off(1);
            let name = os_strings.pop().unwrap();
            (name, children)
        },
        _ => unreachable!()
    };

    let node = treemap
        .entry(name)
        .or_insert(Node { children: BTreeMap::new() });

    insert_os_strings(&mut node.children, children);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root() {
        let mut tree = PathTree::new_root();
        tree.insert(Path::new("/a/1/A"));
        tree.insert(Path::new("/b/2/B"));
        tree.insert(Path::new("/c"));

        assert!(tree.contains(Path::new("/")));

        assert!(tree.contains(Path::new("/b/2/B")));
        assert!(tree.contains(Path::new("b/2/B")));

        assert!(!tree.contains(Path::new("/b/2/3")));

        let dir = tree.find(Path::new("/b")).unwrap();
        assert!(dir.contains(Path::new("2/B")));
        assert!(!dir.contains(Path::new("2/C")));
    }

    #[test]
    fn test_relative() {
        let mut tree = PathTree::new_relative();
        tree.insert(Path::new("a/1/A"));
        tree.insert(Path::new("b/2/B"));
        tree.insert(Path::new("c"));

        assert!(!tree.contains(Path::new("/")));

        assert!(tree.contains(Path::new("b/2/B")));
        assert!(!tree.contains(Path::new("/b/2/B")));

        assert!(!tree.contains(Path::new("b/2/3")));

        let dir = tree.find(Path::new("b")).unwrap();
        assert!(dir.contains(Path::new("2/B")));
    }
}
