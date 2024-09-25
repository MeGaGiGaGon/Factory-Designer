use std::any::TypeId;

use eframe::egui;
use eframe::egui::Color32;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::LayerId;
use eframe::egui::Pos2;
use eframe::egui::Rect;
use eframe::egui::Sense;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::emath::TSTransform;
use slotmap::new_key_type;
use slotmap::SlotMap;

use crate::Node;

new_key_type! {pub struct NodeKey;}
new_key_type! {pub struct ConnectorKey;}
new_key_type! {pub struct InputPointKey;}
new_key_type! {pub struct LinkKey;}

struct NodeInformation<'a> {
    node: Box<dyn Node + 'a>,
    position: Pos2,
}

pub struct NodeGraph<'a, 'b> {
    nodes: SlotMap<NodeKey, NodeInformation<'b>>,
    // attachment_points: SlotMap<ConnectorKey, ConnectorInformation>,
    // input_points: SlotMap<InputPointKey, InputPointInformation>,
    // links: SlotMap<LinkKey, LinkInformation>,
    transform: TSTransform,
    id: Id,
    registered_nodes: Vec<Box<dyn Node + 'a>>,
    pub selector_panel_enabled: bool,
    link_drag_info: Option<(NodeKey, TypeId, Pos2, bool, usize)>,
    next_frame_link_dropped: bool,
    links: Vec<((NodeKey, usize), (NodeKey, usize))>,
}

impl<'a: 'b, 'b> NodeGraph<'a, 'b> {
    pub fn new(id: impl Into<Id>) -> Self {
        Self {
            id: id.into().with("__NodeGraph"),
            nodes: Default::default(),
            // attachment_points: Default::default(),
            // links: Default::default(),
            transform: Default::default(),
            registered_nodes: Default::default(),
            selector_panel_enabled: Default::default(),
            link_drag_info: Default::default(),
            next_frame_link_dropped: Default::default(),
            links: Default::default(),
            // input_points: Default::default(),
        }
    }

    /// Registers a node for spawning from the node selection list
    /// The node given is what will be rendered in the list
    /// and what will be placed when dragging in from the list
    /// Note that registering a node multiple times will duplicate it in the display
    pub fn register_node<'c>(&mut self, node: impl Node + 'c + 'a) {
        self.registered_nodes.push(Box::new(node));
    }

    /// Adds a node to the graph, returning the `NodeKey` unique to it
    /// Note that if you have multiple graphs, using a `NodeKey` from one graph on a different graph is an error
    /// The result is safe, but unspecified, and cannot be detected at runtime
    /// Due to some usages involving creating unspecified numbers of graphs, and for ease of implementation, it is not possible to use a custom key type
    pub fn add_node<'c: 'b>(&mut self, node: Box<dyn Node + 'c>, position: Pos2) -> NodeKey {
        self.nodes.insert(NodeInformation { node, position })
    }

    /// Show the graph using a context
    /// This uses the context's CentralPanel
    pub fn show(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| self.show_inside(ui));
    }

    /// Show the graph inside a ui
    pub fn show_inside(&mut self, ui: &mut Ui) {
        let transform =
            TSTransform::from_translation(ui.min_rect().left_top().to_vec2()) * self.transform;
        let mut offset = Vec2::ZERO;
        if self.selector_panel_enabled {
            let mut node_to_add = None;
            egui::SidePanel::left(self.id.with("node list")).show_inside(ui, |ui| {
                for (index, mut node) in self.registered_nodes.clone().into_iter().enumerate() {
                    let rect = ui.add_enabled_ui(true, |ui| node.show(ui)).response.rect;
                    let response = ui.allocate_rect(rect, Sense::drag());
                    if response.dragged() {
                        egui::Area::new(self.id.with("drag display").with(index))
                            .fixed_pos(
                                ui.ctx()
                                    .input(|i| i.pointer.hover_pos())
                                    .unwrap_or_default(),
                            )
                            .show(ui.ctx(), |ui| {
                                ui.add_enabled_ui(true, |ui| node.show(ui));
                            });
                    }
                    if response.drag_stopped() {
                        if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                            node_to_add = Some((node, pos));
                        }
                    }
                }
            });
            offset = ui.cursor().left_top().to_vec2();
            if let Some((node, pos)) = node_to_add {
                self.add_node(node, transform.inverse().mul_pos(pos - offset));
            }
        }

        let mut link_dropped = self.next_frame_link_dropped;
        let graph_rect = egui::CentralPanel::default()
            .show_inside(ui, |ui| {
                let (id, rect) = ui.allocate_space(ui.available_size());
                let response = ui.interact(rect, id, Sense::click_and_drag());
                if response.dragged() {
                    self.transform.translation += response.drag_delta()
                }
                if response.secondary_clicked() {}
                let transform = TSTransform::from_translation(ui.min_rect().left_top().to_vec2())
                    * self.transform;
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

                let mut input_info_slotmap = SlotMap::new();
                let mut input_info_keys = Vec::new();
                let mut output_info_slotmap = SlotMap::new();
                let mut output_info_keys = Vec::new();
                for (node_key, node_information) in self.nodes.iter_mut() {
                    let window_layer = ui.layer_id();
                    let id = egui::Area::new(id.with(self.id).with(node_key))
                        .default_pos(node_information.position)
                        .order(egui::Order::Middle)
                        .constrain(false)
                        .show(ui.ctx(), |ui| {
                            ui.set_clip_rect(transform.inverse() * rect);
                            let (input_info, output_info) = node_information.node.show(ui);
                            for (i, (t, pos, callback)) in input_info.into_iter().enumerate() {
                                input_info_keys.push((node_key, input_info_slotmap.insert((i, pos, callback))));
                                let response = ui.interact(
                                    Rect::from_two_pos(
                                        pos - Vec2::new(5.0, 5.0),
                                        pos + Vec2::new(5.0, 5.0),
                                    ),
                                    ui.id().with("input").with(i),
                                    Sense::drag(),
                                );
                                if response.drag_started() {
                                    self.link_drag_info = Some((node_key, t, pos, true, i));
                                }
                                if response.drag_stopped() {
                                    self.next_frame_link_dropped = true;
                                }
                                if link_dropped
                                    && !self.link_drag_info.unwrap().3
                                    && self.link_drag_info.unwrap().1 == t
                                    && transform.mul_pos(pos).distance_sq(
                                        ui.ctx()
                                            .input(|i| i.pointer.hover_pos().unwrap_or_default()),
                                    ) <= 100.0
                                {
                                    link_dropped = false;
                                    self.next_frame_link_dropped = false;
                                    self.links.push((
                                        (node_key, i),
                                        (
                                            self.link_drag_info.unwrap().0,
                                            self.link_drag_info.unwrap().4,
                                        ),
                                    ));
                                    self.link_drag_info = None;
                                }
                            }
                            for (i, (t, pos, callback)) in output_info.into_iter().enumerate() {
                                output_info_keys.push((node_key, output_info_slotmap.insert((i, pos, callback))));
                                let response = ui.interact(
                                    Rect::from_two_pos(
                                        pos - Vec2::new(5.0, 5.0),
                                        pos + Vec2::new(5.0, 5.0),
                                    ),
                                    ui.id().with("output").with(i),
                                    Sense::drag(),
                                );
                                if response.drag_started() {
                                    self.link_drag_info = Some((node_key, t, pos, false, i));
                                }
                                if response.drag_stopped() {
                                    self.next_frame_link_dropped = true;
                                }
                                if link_dropped
                                    && self.link_drag_info.unwrap().3
                                    && self.link_drag_info.unwrap().1 == t
                                    && transform.mul_pos(pos).distance_sq(
                                        ui.ctx()
                                            .input(|i| i.pointer.hover_pos().unwrap_or_default()),
                                    ) <= 100.0
                                {
                                    link_dropped = false;
                                    self.next_frame_link_dropped = false;
                                    self.links.push((
                                        (
                                            self.link_drag_info.unwrap().0,
                                            self.link_drag_info.unwrap().4,
                                        ),
                                        (node_key, i),
                                    ));
                                    self.link_drag_info = None;
                                }
                            }
                        })
                        .response
                        .layer_id;
                    ui.ctx().set_transform_layer(id, transform);
                    ui.ctx().set_sublayer(window_layer, id);
                }
                for ((start_key, start_index), (end_key, end_index)) in self.links.iter() {
                    let start = input_info_keys
                        .iter()
                        .filter(|x| x.0 == *start_key)
                        .nth(*start_index)
                        .unwrap();
                    let end = output_info_keys
                        .iter()
                        .filter(|x| x.0 == *end_key)
                        .nth(*end_index)
                        .unwrap();
                    let start = input_info_slotmap.remove(start.1).unwrap();
                    let end = output_info_slotmap.remove(end.1).unwrap();
                    ui.painter().line_segment(
                        [transform.mul_pos(start.1), transform.mul_pos(end.1)],
                        (3.0, Color32::YELLOW),
                    );
                    start.2(end.2());
                }
                // if ui.ctx().input(|i| i.pointer.primary_clicked()) {
                //     dbg!(ui.ctx().input(|i| i.pointer.interact_pos()));
                // }
                // for (i, (t, pos)) in inputs.into_iter().enumerate() {
                //     ui.painter().rect_filled(Rect::from_two_pos(pos - Vec2::new(5.0, 5.0), pos + Vec2::new(5.0, 5.0)), 0.0, Color32::LIGHT_GREEN);
                //     let pos = transform.mul_pos(pos);
                //     ui.painter().rect_filled(Rect::from_two_pos(pos - Vec2::new(5.0, 5.0), pos + Vec2::new(5.0, 5.0)), 0.0, Color32::GREEN);
                //     if ui.interact(Rect::from_two_pos(pos - Vec2::new(5.0, 5.0), pos + Vec2::new(5.0, 5.0)), self.id.with("input").with(i), Sense::click()).clicked() {
                //         println!("clicked {i} {pos}");
                //     }
                // }
            })
            .response
            .rect;

        if let Some((_, _, pos, _, _)) = self.link_drag_info {
            ui.ctx()
                .layer_painter(LayerId::new(
                    egui::Order::Foreground,
                    self.id.with("drag line painter layer"),
                ))
                .with_clip_rect(graph_rect)
                .line_segment(
                    [
                        transform.mul_pos(pos + offset),
                        (ui.ctx()
                            .input(|i| i.pointer.hover_pos().unwrap_or_default())),
                    ],
                    (3.0, Color32::YELLOW),
                );
        }

        if link_dropped {
            self.next_frame_link_dropped = false;
            self.link_drag_info = None;
        }
    }
}
