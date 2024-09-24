use eframe::egui::{self, Ui};

use crate::{node_input::NodeInput, node_output::NodeOutput, Node};


#[derive(Default, Clone)]
pub struct SourceNode {
    value: u8,
}
/// A body is made of three things: a list of inputs, the central ui, a list of outputs
/// each input/output can have 2 attached FnOnces: a ui display, and a value callback
/// The ui display is normal |ui| {}
/// The callback is created via a function fn on_connect_input/output<T> where T is the type input output
/// Connecting two connectors together is only possible if they share the same value
/// values are passed as Box<dyn Any> and downcast is used to check if a connection is possible
impl Node for SourceNode {
    fn title(&self) -> &str {
        "Source"
    }

    fn body(
        &mut self,
    ) -> (std::vec::Vec<NodeInput>, Box<(dyn for<'a> FnOnce(&'a mut Ui) + 'static)>, std::vec::Vec<NodeOutput>) { 
        let mut self2 = self.clone();
        let self3 = self.clone();
        let connecter = NodeOutput::new(move |ui| { ui.add(egui::Slider::new(&mut self2.value, 0..=u8::MAX)); }, move || {self3.value});
        self.value = self2.value;
        (vec![], Box::new(|_| {}), vec![connecter])
    }
}