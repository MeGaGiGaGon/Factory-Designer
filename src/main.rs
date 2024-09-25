mod node_input;
mod node_output;
mod node_graph;
mod node;
mod nodes;
mod createable_node;

use eframe::egui;
use eframe::egui::Response;
use eframe::egui::Ui;
use eframe::egui::WidgetText;
use eframe::NativeOptions;
use node_input::NodeInput;
use node_output::NodeOutput;
use nodes::graph_node::GraphNode;

use crate::node::Node;
use crate::node_graph::NodeGraph;
use crate::nodes::adder_node::AdderNode;
use crate::nodes::one_to_n_node::OneToNNode;
use crate::nodes::sink_node::SinkNode;
use crate::nodes::source_node::SourceNode;

/// Selecting text is currently broken under a TSTransform, 
/// so this is a shortcut to prevent it in labels
fn unselectable_label(ui: &mut Ui, text: impl Into<WidgetText>) -> Response {
    ui.add(egui::Label::new(text).selectable(false))
}

#[derive(Default, Clone)]
struct DebugNode;

impl Node for DebugNode {
    fn title(&self) -> &str {
        "Test Node"
    }

    fn body<'a>(
        &'a mut self,
    ) -> (Vec<NodeInput>, Box<(dyn FnOnce(&mut Ui) + 'a)>, Vec<NodeOutput>) {
        (vec![], Box::new(|_| {}), vec![])
    }
}


fn main() -> eframe::Result<()> {
    let mut graph = NodeGraph::new("test");
    graph.selector_panel_enabled = true;
    graph.register_node(DebugNode::default());
    graph.register_node(SourceNode::default());
    graph.register_node(SinkNode::default());
    graph.register_node(AdderNode::default());
    graph.register_node(OneToNNode::default());
    graph.register_node_with_id::<GraphNode>();
    eframe::run_simple_native("app_name", NativeOptions::default(), move |ctx, _frame| {
        graph.show(ctx);
    })
}
