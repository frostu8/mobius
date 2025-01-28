//! Application driver.

use floem::prelude::*;

use std::path::PathBuf;
use std::rc::Rc;

use crate::file_explorer::{
    data::{Node, Tree},
    view::file_explorer_view,
};

pub fn app_view() -> impl IntoView {
    let project_path = std::env::args_os()
        .skip(1)
        .next()
        .map(|s| PathBuf::from(s))
        .expect("arg1");

    let mut tree = Tree::new(Node {
        is_dir: true,
        is_open: true,
        ..Node::new(project_path.clone())
    });

    Rc::make_mut(tree.root_mut()).is_open = true;

    for entry in walkdir::WalkDir::new(project_path.clone()) {
        if let Ok(entry) = entry {
            tree.create(Node {
                is_dir: entry.file_type().is_dir(),
                is_open: true,
                ..Node::new(entry.into_path())
            });
        }
    }

    let tree = create_rw_signal(tree);

    container(file_explorer_view(tree)).style(|s| {
        s.size(100.pct(), 100.pct())
            .padding_vert(20.0)
            .flex_col()
            .items_center()
    })
}
