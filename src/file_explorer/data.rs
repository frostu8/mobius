//! File explorer tree.

use std::{
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

/// A file explorer tree.
#[derive(Clone, Debug)]
pub struct Tree {
    arena: im::Vector<Node>,
}

impl Tree {
    /// Creates a new `Tree` at a given base path.
    pub fn new(base: impl Into<PathBuf>) -> Tree {
        // create root node @ index zero
        let node = Node::new(base);

        let mut arena = im::Vector::new();
        arena.push_back(node);
        Tree { arena }
    }

    /// The root node of the `Tree`.
    pub fn root(&self) -> &Node {
        self.arena.iter().next().expect("tree must have root node")
    }

    /// Mutably gets the root node of the `Tree`.
    pub fn root_mut(&mut self) -> &mut Node {
        self.arena
            .iter_mut()
            .next()
            .expect("tree must have root node")
    }

    /// Gets a specific node in the tree.
    pub fn get(&self, ix: TreeIndex) -> Option<&Node> {
        self.arena.get(ix.0.get())
    }

    /// Gets a specific node in the tree mutably.
    pub fn get_mut(&mut self, ix: TreeIndex) -> Option<&mut Node> {
        self.arena.get_mut(ix.0.get())
    }

    fn update_node(&mut self, ix: TreeIndex) {
        let Some(node) = self.get(ix) else {
            return;
        };

        let discriminator = node
            .path
            .strip_prefix(self.root().path())
            .expect("valid subnode");
        let mut ancestors = discriminator.ancestors().collect::<Vec<_>>();
        ancestors.reverse();

        for i in 0..ancestors.len() {}
    }
}

/// An index into a [`Tree`].
///
/// Represents a [`Node`] in a tree.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct TreeIndex(NonZeroUsize);

/// A file explorer node.
#[derive(Clone, Debug)]
pub struct Node {
    path: PathBuf,
    is_dir: bool,
    is_open: bool,
    children: im::Vector<TreeIndex>,
    children_open_count: usize,
}

impl Node {
    /// Creates a new `Node` representing the entity @ `path`.
    pub fn new(path: impl Into<PathBuf>) -> Node {
        Node {
            path: path.into(),
            is_dir: false,
            is_open: false,
            children: im::Vector::new(),
            children_open_count: 0,
        }
    }

    /// The path of the node.
    pub fn path(&self) -> &Path {
        &self.path
    }
}
