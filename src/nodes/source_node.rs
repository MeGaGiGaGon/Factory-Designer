use std::cell::RefCell;

use eframe::egui;
use eframe::egui::Ui;

use crate::{node_input::NodeInput, node_output::NodeOutput, Node};


#[derive(Default, Clone)]
pub struct SourceNode {
    value: RefCell<u8>,
}

impl Node for SourceNode {
    fn title(&self) -> &str {
        "Source"
    }

    fn body<'a>(
        &'a mut self,
    ) -> (std::vec::Vec<NodeInput>, Box<(dyn FnOnce(&mut Ui) + 'a)>, std::vec::Vec<NodeOutput>) { 
        (vec![], Box::new(|_| {}), vec![NodeOutput::new(|ui| {ui.add(egui::Slider::new(&mut *self.value.borrow_mut(), 0..=u8::MAX));}, || self.value.borrow().clone())])
    }
}