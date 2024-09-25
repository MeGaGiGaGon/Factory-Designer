use eframe::egui::Id;

use crate::node::Node;

/// Used as an alternate method of adding nodes to the graph 
/// for nodes that require a unique Id to work
pub trait CreatableNode<'a> {
    fn new_with_id(id: Id) -> Box<dyn Node + 'a>;
}