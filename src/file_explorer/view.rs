//! The actual Floem views associated with the file explorer.

use floem::prelude::*;

use super::data::Tree;
use super::list::TreeView;

/// The file explorer view.
pub fn file_explorer_view(tree: RwSignal<Tree>) -> impl IntoView {
    scroll(
        virtual_list(
            VirtualDirection::Vertical,
            VirtualItemSize::Fixed(Box::new(|| 20.0)),
            move || TreeView::new(tree.get()),
            move |item| item.node.path().to_owned(),
            move |item| {
                let padding = item.level as f32 * 12.0;
                label(move || item.file_name().to_owned())
                    .style(move |s| s.height(20.0).padding_left(padding))
            },
        )
        .style(|s| s.flex_col().width_full()),
    )
    .style(|s| s.width(200.0).height(100.pct()).border(1.0))
}
