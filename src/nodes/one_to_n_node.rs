use std::cell::RefCell;
use std::ops::AddAssign;

use eframe::egui::Ui;

use crate::unselectable_label;
use crate::{node_input::NodeInput, node_output::NodeOutput, Node};

#[derive(Default, Clone)]
pub struct OneToNNode {
    value_1: RefCell<u8>,
    output_count: RefCell<u8>,
}

impl Node for OneToNNode {
    fn title(&self) -> &str {
        "OneToN"
    }

    fn body<'a>(
        &'a mut self,
    ) -> (
        std::vec::Vec<NodeInput>,
        Box<(dyn FnOnce(&mut Ui) + 'a)>,
        std::vec::Vec<NodeOutput>,
    ) {
        let mut output_callbacks = Vec::new();
        for _ in 0..self.output_count.borrow().clone() {
            output_callbacks.push(NodeOutput::output(|| {
                self.output_count.borrow_mut().add_assign(1);
                self.value_1.borrow().clone()
            }));
        }
        self.output_count.replace(1);
        (
            vec![NodeInput::input(|x| {
                self.value_1.replace(x);
            })],
            Box::new(|ui| {
                unselectable_label(ui, self.value_1.borrow().to_string());
            }),
            output_callbacks,
        )
    }
}
