use std::cell::RefCell;

use eframe::egui::Ui;

use crate::{createable_node::CreatableNode, node_graph::NodeGraph, node_input::NodeInput, node_output::NodeOutput, Node};

#[derive(Clone)]
pub struct GraphNode<'a, 'b> {
    graph: RefCell<NodeGraph<'a, 'b>>,
}

impl<'a: 'b, 'b: 'c, 'c> CreatableNode<'c> for GraphNode<'a, 'b> {
    fn new_with_id(id: eframe::egui::Id) -> Box<(dyn Node + 'c)> 
    {
        Box::new(Self { graph: NodeGraph::new(id.with("graph node node")).enable_selector_panel().into() })
    }
}

impl<'a: 'b, 'b> Node for GraphNode<'a, 'b> {
    fn title(&self) -> &str {
        "Graph Node"
    }

    fn body<'c>(
        &'c mut self,
    ) -> (std::vec::Vec<NodeInput>, Box<(dyn FnOnce(&mut Ui) + 'c)>, std::vec::Vec<NodeOutput>) { 
        (vec![], Box::new(|ui| {
            self.graph.borrow_mut().show_inside(ui)
        }), vec![])
    }
}