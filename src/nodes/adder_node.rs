use std::cell::RefCell;

use eframe::egui::Ui;

use crate::unselectable_label;
use crate::{node_input::NodeInput, node_output::NodeOutput, Node};

#[derive(Default, Clone)]
pub struct AdderNode {
    value_1: RefCell<u8>,
    value_2: RefCell<u8>,
}

impl Node for AdderNode {
    fn title(&self) -> &str {
        "Adder"
    }

    fn body<'a>(
        &'a mut self,
    ) -> (
        std::vec::Vec<NodeInput>,
        Box<(dyn FnOnce(&mut Ui) + 'a)>,
        std::vec::Vec<NodeOutput>,
    ) {
        (
            vec![NodeInput::new(
                |ui| {
                    unselectable_label(ui, self.value_1.borrow().to_string());
                },
                |x| {
                    self.value_1.replace(x);
                },
            ),NodeInput::new(
                |ui| {
                    unselectable_label(ui, self.value_2.borrow().to_string());
                },
                |x| {
                    self.value_2.replace(x);
                },
            )],
            Box::new(|_| {}),
            vec![NodeOutput::new(|ui| {
                unselectable_label(ui, (self.value_1.borrow().wrapping_add(*self.value_2.borrow())).to_string());
            }, || {
                self.value_1.borrow().wrapping_add(*self.value_2.borrow())
            })],
        )
    }
}
