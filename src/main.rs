mod node_input;
mod node_output;
mod source_node;

use std::any::TypeId;

use dyn_clone::clone_trait_object;
use dyn_clone::DynClone;
use eframe::egui;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::Label;
use eframe::egui::Pos2;
use eframe::egui::Sense;
use eframe::egui::Ui;
use eframe::emath::TSTransform;
use eframe::NativeOptions;
use node_input::NodeInput;
use node_output::NodeOutput;
/// The core is slotmap, with everything being built around it
/// The graph is 3 slotmaps,
/// nodes
/// attachment points
/// links
/// when creating any, a slotmap key is returned for the person to do whatever with
use slotmap::new_key_type;
use slotmap::SlotMap;
use source_node::SourceNode;

new_key_type! {struct NodeKey;}
new_key_type! {struct ConnectorKey;}
new_key_type! {struct InputPointKey;}
new_key_type! {struct LinkKey;}

trait Node: DynClone {
    /// The title to display for the node
    fn title(&self) -> &str;
    fn body(&mut self) -> (Vec<NodeInput>, Box<dyn FnOnce(&mut Ui)>, Vec<NodeOutput>);
    fn register_node(&mut self, node_graph: &mut NodeGraph);
    fn register_input_point<T: 'static>(&mut self, node_graph: &mut NodeGraph, node_key: NodeKey) -> InputPointKey {
        node_graph.add_input_point(node_key, TypeId::of::<T>())
    }
    fn show(&self, ui: &mut Ui) {
        egui::Frame::default()
            .inner_margin(8.0)
            .stroke(ui.ctx().style().visuals.window_stroke)
            .show(ui, |ui| {
                ui.add(Label::new(self.title()).selectable(false));
                ui.separator();
            });
    }
}

clone_trait_object!(Node);

struct InputPointInformation {
    node_key: NodeKey,
    type_id: TypeId,
}

struct NodeGraph {
    nodes: SlotMap<NodeKey, NodeInformation>,
    attachment_points: SlotMap<ConnectorKey, ConnectorInformation>,
    input_points: SlotMap<InputPointKey, InputPointInformation>,
    links: SlotMap<LinkKey, LinkInformation>,
    transform: TSTransform,
    id: Id,
    registered_nodes: Vec<Box<dyn Node>>,
    selector_panel_enabled: bool,
}

impl NodeGraph {
    fn new(id: impl Into<Id>) -> Self {
        Self {
            id: id.into().with("__NodeGraph"),
            nodes: Default::default(),
            attachment_points: Default::default(),
            links: Default::default(),
            transform: Default::default(),
            registered_nodes: Default::default(),
            selector_panel_enabled: Default::default(),
            input_points: Default::default(),
        }
    }

    /// Registers a node for spawning from the node selection list
    /// Default is required to have a default state to spawn the node in
    /// Note that registering a node multiple times will duplicate it in the display
    fn register_node<N: Node + Default + 'static>(&mut self) {
        self.registered_nodes.push(Box::new(N::default()));
    }

    /// Adds a node to the graph, returning the `NodeKey` unique to it
    /// Note that if you have multiple graphs, using a `NodeKey` from one graph on a different graph is an error
    /// The result is safe, but unspecified, and cannot be detected at runtime
    /// Due to some usages involving creating unspecified numbers of graphs, and for ease of implementation, it is not possible to use a custom key type
    fn add_node(&mut self, node: Box<dyn Node>, position: Pos2) -> NodeKey {
        self.nodes.insert(NodeInformation { node, position })
    }

    /// Adds an input point to the graph for a specific node
    fn add_input_point(&mut self, node_key: NodeKey, type_id: TypeId) -> InputPointKey {
        self.input_points.insert(InputPointInformation { node_key, type_id })
    }

    /// Show the graph using a context
    /// This uses the context's CentralPanel
    fn show(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| self.show_inside(ui));
    }

    /// Show the graph inside a ui
    fn show_inside(&mut self, ui: &mut Ui) {
        let transform =
            TSTransform::from_translation(ui.min_rect().left_top().to_vec2()) * self.transform;

        if self.selector_panel_enabled {
            egui::SidePanel::left(self.id.with("node list")).show_inside(ui, |ui| {
                for (index, node) in self.registered_nodes.clone().into_iter().enumerate() {
                    let rect = ui.add_enabled_ui(false, |ui| node.show(ui)).response.rect;
                    let response = ui.allocate_rect(rect, Sense::drag());
                    if response.dragged() {
                        egui::Area::new(self.id.with("drag display").with(index))
                            .fixed_pos(
                                ui.ctx()
                                    .input(|i| i.pointer.hover_pos())
                                    .unwrap_or_default(),
                            )
                            .show(ui.ctx(), |ui| {
                                ui.add_enabled_ui(false, |ui| node.show(ui));
                            });
                    }
                    if response.drag_stopped() {
                        if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                            self.add_node(node, pos - transform.translation);
                        }
                    }
                }
            });
        }

        let (id, rect) = ui.allocate_space(ui.available_size());
        let response = ui.interact(rect, id, Sense::click_and_drag());
        if response.dragged() {
            self.transform.translation += response.drag_delta()
        }
        if response.secondary_clicked() {}
        let transform =
            TSTransform::from_translation(ui.min_rect().left_top().to_vec2()) * self.transform;
        if let Some(pointer) = ui.ctx().input(|i| i.pointer.hover_pos()) {
            let pointer_in_layer = transform.inverse() * pointer;
            let zoom_delta = ui.ctx().input(|i| i.zoom_delta());
            let pan_delta = ui.ctx().input(|i| i.smooth_scroll_delta);
            // Zoom in on pointer:
            self.transform = self.transform
                * TSTransform::from_translation(pointer_in_layer.to_vec2())
                * TSTransform::from_scaling(zoom_delta)
                * TSTransform::from_translation(-pointer_in_layer.to_vec2());

            // Pan:
            self.transform = TSTransform::from_translation(pan_delta) * self.transform;
        }

        for (node_key, node_information) in self.nodes.iter() {
            let window_layer = ui.layer_id();
            let id = egui::Area::new(id.with(self.id).with(node_key))
                .default_pos(node_information.position)
                .order(egui::Order::Middle)
                .constrain(false)
                .show(ui.ctx(), |ui| {
                    ui.set_clip_rect(transform.inverse() * rect);
                    node_information.node.show(ui);
                })
                .response
                .layer_id;
            ui.ctx().set_transform_layer(id, transform);
            ui.ctx().set_sublayer(window_layer, id);
        }
    }
}

#[derive(Default)]
struct NodeSelector {
    shown: bool,
    position: Pos2,
    available_nodes: Vec<Box<dyn Node>>,
}

struct NodeInformation {
    node: Box<dyn Node>,
    position: Pos2,
}

struct ConnectorInformation {
    connector: Box<dyn Connector2>,
}

struct LinkInformation {
    link: Box<dyn Link>,
}

trait Connector2 {}

struct Connector {
    sinks: u16,
    sources: u16,
}

trait Link {}

#[derive(Default, Clone)]
struct DebugNode;

impl Node for DebugNode {
    fn title(&self) -> &str {
        "Test Node"
    }

    fn body(
        &mut self,
    ) -> (Vec<NodeInput>, Box<(dyn for<'a> FnOnce(&'a mut Ui) + 'static)>, Vec<NodeOutput>) {
        (vec![], Box::new(|_| {}), vec![])
    }
}


fn main() -> eframe::Result<()> {
    let mut graph = NodeGraph::new("test");
    graph.selector_panel_enabled = true;
    graph.register_node::<DebugNode>();
    graph.register_node::<SourceNode>();
    eframe::run_simple_native("app_name", NativeOptions::default(), move |ctx, _frame| {
        graph.show(ctx);
    })
}
