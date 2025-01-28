//! File explorer tree.

use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// A file explorer tree.
#[derive(Clone, Debug)]
pub struct Tree {
    arena: im::Vector<Rc<Node>>,
}

impl Tree {
    /// Creates a new `Tree` at a given base path.
    ///
    /// The base path is automatically inferred to be a directory.
    pub fn new(base: Node) -> Tree {
        // create root node @ index zero
        let mut arena = im::Vector::new();
        arena.push_back(Rc::new(base));
        Tree { arena }
    }

    /// Gets the root node.
    pub fn root(&self) -> &Rc<Node> {
        self.get(TreeIndex::ROOT).expect("tree must have root node")
    }

    /// Gets the root node mutably.
    pub fn root_mut(&mut self) -> &mut Rc<Node> {
        self.get_mut(TreeIndex::ROOT)
            .expect("tree must have root node")
    }

    /// Gets a specific node in the tree.
    pub fn get(&self, ix: TreeIndex) -> Option<&Rc<Node>> {
        self.arena.get(ix.0.get() - 1)
    }

    /// Gets a specific node in the tree mutably.
    pub fn get_mut(&mut self, ix: TreeIndex) -> Option<&mut Rc<Node>> {
        self.arena.get_mut(ix.0.get() - 1)
    }

    /// Creates a node in the tree that is a child of an existing node.
    ///
    /// Returns `None` if the path cannot be represented in the tree.
    pub fn create(&mut self, node: Node) -> Option<TreeIndex> {
        let node = Rc::new(node);

        let Ok(discriminator) = node.path.strip_prefix(self.root().path()) else {
            return None;
        };
        let mut ancestors_rev = node
            .path
            .ancestors()
            .take(discriminator.components().count())
            .map(|a| self.root().path().join(a))
            .collect::<Vec<_>>();
        ancestors_rev.reverse();

        let mut cur_ix = TreeIndex::ROOT;
        for ancestor in ancestors_rev {
            // get next node
            match self.get(cur_ix).and_then(|n| n.children.get(&ancestor)) {
                Some(ix) => cur_ix = *ix,
                None => {
                    // if it does not exist, create it
                    let node_ix = if node.path == ancestor {
                        self.push(node.clone())
                    } else {
                        let parent = Rc::new(Node {
                            is_dir: true,
                            ..Node::new(&ancestor)
                        });
                        self.push(parent)
                    };

                    // add ancestor to children
                    let parent = Rc::make_mut(self.get_mut(cur_ix).expect("node exists"));
                    parent.children.insert(ancestor, node_ix);
                    cur_ix = node_ix;
                }
            }
        }

        // run updates
        self.update_node(cur_ix);

        Some(cur_ix)
    }

    /// Creates an unlinked node in the tree.
    fn push(&mut self, node: Rc<Node>) -> TreeIndex {
        let ix = self.arena.len();
        self.arena.push_back(node);
        TreeIndex(NonZeroUsize::new(ix + 1).unwrap())
    }

    /// Updates a node's [`Node::children_open_count`], cascading updating all
    /// other nodes above it.
    fn update_node(&mut self, ix: TreeIndex) {
        let Some(node) = self.get(ix) else {
            return;
        };

        let discriminator = node
            .path
            .strip_prefix(self.root().path())
            .expect("valid subnode");
        let mut ancestors_rev = discriminator
            .ancestors()
            .take(discriminator.components().count())
            .map(|a| self.root().path().join(a))
            .collect::<Vec<_>>();
        ancestors_rev.reverse();

        // works way up the tree
        for i in (0..ancestors_rev.len()).rev() {
            // finds node
            let mut cur_ix = TreeIndex::ROOT;
            for ancestor in ancestors_rev.iter().take(i) {
                // get next node
                match self.get(cur_ix).and_then(|n| n.children.get(ancestor)) {
                    Some(ix) => cur_ix = *ix,
                    None => return,
                }
            }

            // update current node children_open_count
            let mut children_open_count = 0;
            // count all children
            for ix in self.get(cur_ix).expect("node to exist").children.values() {
                let Some(child) = self.get(*ix) else {
                    continue;
                };
                // add one for node's existence
                children_open_count += 1;
                // add all children if node is open
                if child.is_open {
                    children_open_count += child.children_open_count;
                }
            }

            // update node
            let node = Rc::make_mut(self.get_mut(cur_ix).expect("node to exist"));
            node.children_open_count = children_open_count;
        }
    }
}

/// An index into a [`Tree`].
///
/// Represents a [`Node`] in a tree.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TreeIndex(NonZeroUsize);

impl TreeIndex {
    // SAFETY: this number is 1
    pub const ROOT: TreeIndex = unsafe { TreeIndex(NonZeroUsize::new_unchecked(1)) };
}

/// A file explorer node.
#[derive(Clone, Debug)]
pub struct Node {
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_open: bool,
    pub children: im::HashMap<PathBuf, TreeIndex>,
    pub children_open_count: usize,
}

impl Node {
    /// Creates a new `Node` representing the entity @ `path`.
    pub fn new(path: impl Into<PathBuf>) -> Node {
        Node {
            path: path.into(),
            is_dir: false,
            is_open: false,
            children: im::HashMap::new(),
            children_open_count: 0,
        }
    }

    /// The path of the node.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        if self.path == other.path
            && self.is_dir == other.is_dir
            && self.is_open == other.is_open
            && self.children_open_count == other.children_open_count
        {
            // check children
            for (k, v) in self.children.iter() {
                if let Some(other_v) = other.children.get(k) {
                    if v != other_v {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_node() {
        let mut tree = Tree::new(Node {
            is_dir: true,
            is_open: true,
            ..Node::new("/var")
        });

        Rc::make_mut(tree.root_mut()).is_open = true;

        tree.create(Node {
            is_dir: true,
            ..Node::new("/var/opt")
        });
        tree.create(Node::new("/var/opt/hidden"));
        tree.create(Node::new("/var/opt/hidden2"));
        tree.create(Node::new("/var/opt/hidden3"));
        tree.create(Node {
            is_dir: true,
            is_open: true,
            ..Node::new("/var/games")
        });
        tree.create(Node::new("/var/games/battleblock"));
        tree.create(Node::new("/var/games/spelunky"));
        tree.create(Node::new("/var/games/minesweeper"));

        assert_eq!(tree.root().children_open_count, 5);
    }

    #[test]
    fn test_create_node() {
        let mut tree = Tree::new(Node {
            is_dir: true,
            ..Node::new("/var")
        });
        tree.create(Node::new("/var/opt"));
        tree.create(Node::new("/var/games"));
        tree.create(Node::new("/var/games/secret"));

        assert_eq!(
            tree.arena
                .into_iter()
                .map(|c| Rc::unwrap_or_clone(c))
                .collect::<Vec<_>>(),
            vec![
                Node {
                    is_dir: true,
                    children_open_count: 2,
                    children: im::hashmap! {
                        PathBuf::from("/var/opt") => TreeIndex(NonZeroUsize::new(2).unwrap()),
                        PathBuf::from("/var/games") => TreeIndex(NonZeroUsize::new(3).unwrap()),
                    },
                    ..Node::new("/var")
                },
                Node::new("/var/opt"),
                Node {
                    children: im::hashmap! {
                        PathBuf::from("/var/games/secret") => TreeIndex(NonZeroUsize::new(4).unwrap()),
                    },
                    children_open_count: 1,
                    ..Node::new("/var/games")
                },
                Node::new("/var/games/secret"),
            ]
        );
    }
}
