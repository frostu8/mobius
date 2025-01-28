//! Floem virtual list implementation.

use floem::views::VirtualVector;

use std::ops::Range;
use std::rc::Rc;

use super::data::{Node, Tree, TreeIndex};

/// A virtual list for files
pub struct TreeView {
    tree: Tree,
}

impl TreeView {
    pub fn new(tree: Tree) -> TreeView {
        TreeView { tree }
    }
}

/// A single virtual node.
pub struct NodeView {
    /// The actual node.
    pub node: Rc<Node>,
    /// The level of the node.
    pub level: usize,
}

impl NodeView {
    /// The reduced filename of the node.
    pub fn file_name(&self) -> &str {
        // TODO maybe not unwrap this
        self.node
            .path
            .file_name()
            .and_then(|s| s.to_str())
            .expect("path")
    }
}

impl VirtualVector<NodeView> for TreeView {
    fn total_len(&self) -> usize {
        self.tree.root().children_open_count + 1
    }

    fn slice(&mut self, range: Range<usize>) -> impl Iterator<Item = NodeView> {
        TraverseTree::new(&self.tree)
            .skip(range.start)
            .take(range.len())
    }
}

/// Iterates over all the nodes in a [`Tree`], and their children.
struct TraverseTree<'a> {
    tree: &'a Tree,
    stack: Vec<TraverseEl>,
}

impl<'a> TraverseTree<'a> {
    pub fn new(tree: &'a Tree) -> TraverseTree<'a> {
        TraverseTree {
            stack: vec![TraverseEl {
                ix: TreeIndex::ROOT,
                child_ix: 0,
            }],
            tree,
        }
    }
}

struct TraverseEl {
    ix: TreeIndex,
    child_ix: usize,
}

impl<'a> Iterator for TraverseTree<'a> {
    type Item = NodeView;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // check tos
            if let Some(tos) = self.stack.last_mut() {
                // continue where we left off
                let next = self
                    .tree
                    .get(tos.ix)
                    .expect("valid node")
                    .children
                    .values()
                    .skip(tos.child_ix)
                    .copied()
                    .next();

                if let Some(next_ix) = next {
                    tos.child_ix += 1;
                    let out = NodeView {
                        node: self.tree.get(next_ix).expect("valid node").clone(),
                        level: self.stack.len() - 1,
                    };
                    if out.node.is_open {
                        // iterate over children
                        self.stack.push(TraverseEl {
                            ix: next_ix,
                            child_ix: 0,
                        });
                    }
                    return Some(out);
                } else {
                    // pop stack
                    self.stack.pop();
                }
            } else {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_traversal() {
        let mut tree = Tree::new(Node {
            is_dir: true,
            is_open: true,
            ..Node::new("/var")
        });

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

        let mut out = TraverseTree::new(&tree)
            .map(|s| s.node.path().to_owned())
            .collect::<Vec<_>>();
        out.sort();

        assert_eq!(
            out,
            vec![
                PathBuf::from("/var"),
                PathBuf::from("/var/games"),
                PathBuf::from("/var/games/battleblock"),
                PathBuf::from("/var/games/minesweeper"),
                PathBuf::from("/var/games/spelunky"),
                PathBuf::from("/var/opt"),
            ]
        );
    }
}
