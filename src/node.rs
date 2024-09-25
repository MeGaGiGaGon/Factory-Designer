use std::any::Any;
use std::any::TypeId;

use dyn_clone::clone_trait_object;
use dyn_clone::DynClone;
use eframe::egui;
use eframe::egui::Color32;
use eframe::egui::Pos2;
use eframe::egui::Ui;
use eframe::egui::Vec2;

use crate::node_input::NodeInput;
use crate::node_output::NodeOutput;
use crate::unselectable_label;


pub trait Node: DynClone {
    /// The title to display for the node
    fn title(&self) -> &str;
    /// A body is made of three things: a list of inputs, the central ui, a list of outputs
    /// each input/output can have 2 attached FnOnces: a ui display, and a value callback
    /// The ui display is normal |ui| {}
    /// The callback is created via a function fn on_connect_input/output<T> where T is the type input output
    /// Connecting two connectors together is only possible if they share the same value
    /// values are passed as Box<dyn Any> and downcast is used to check if a connection is possible
    fn body<'a>(&'a mut self) -> (Vec<NodeInput>, Box<dyn FnOnce(&mut Ui) + 'a>, Vec<NodeOutput>);
    /// The method used to display the node
    /// Contains a default implementation that should cover most use cases
    /// Returns a Vec for the types and locations of inputs and outputs to
    /// be used by the NodeGraph for connection handling
    fn show<'a, 'b, 'c: 'a + 'b>(&'c mut self, ui: &mut Ui) -> (Vec<(TypeId, Pos2, Box<dyn FnOnce(Box<dyn Any>) + 'a>)>, Vec<(TypeId, Pos2, Box<dyn FnOnce() -> Box<dyn Any> + 'b>)>) {
        egui::Frame::default()
            .inner_margin(8.0)
            .fill(ui.style().visuals.window_fill)
            .stroke(ui.ctx().style().visuals.window_stroke)
            .show(ui, |ui| {
                unselectable_label(ui, self.title());
                ui.separator();
                ui.horizontal(|ui| {
                    let (inputs, body, outputs) = self.body();
                    let mut input_positions = Vec::new();
                    ui.vertical(|ui| {
                        for input in inputs {
                            ui.horizontal(|ui| {
                                let (_, rect) = ui.allocate_space(Vec2::new(10.0, 10.0));
                                let input_position = rect.left_top() + Vec2::new(5.0, 5.0);
                                ui.painter_at(rect).circle_filled(input_position, 5.0, Color32::BLUE);
                                input_positions.push((input.input_type, input_position, input.input_callback));
                                (input.ui_callback)(ui);
                            });
                        }
                    });
                    ui.add_enabled_ui(true, body);
                    let mut output_positions = Vec::new();
                    ui.vertical(|ui| {
                        for output in outputs {
                            ui.horizontal(|ui| {
                                (output.ui_callback)(ui);
                                let (_, rect) = ui.allocate_space(Vec2::new(10.0, 10.0));
                                let output_position = rect.left_top() + Vec2::new(5.0, 5.0);
                                ui.painter_at(rect).circle_filled(output_position, 5.0, Color32::RED);
                                output_positions.push((output.output_type, output_position, output.output_callback));
                            });
                        }
                    });
                    (input_positions, output_positions)
                }).inner
            }).inner
    }
}

clone_trait_object!(Node);