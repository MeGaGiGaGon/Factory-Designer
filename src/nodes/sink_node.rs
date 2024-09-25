use std::cell::RefCell;

use eframe::egui::Ui;

use crate::unselectable_label;
use crate::{node_input::NodeInput, node_output::NodeOutput, Node};

#[derive(Default, Clone)]
pub struct SinkNode {
    value: RefCell<u8>,
}

impl Node for SinkNode {
    fn title(&self) -> &str {
        "Sink"
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
                    unselectable_label(ui, self.value.borrow().to_string());
                },
                |x| { self.value.replace(x); },
            )],
            Box::new(|_| {}),
            vec![],
        )
    }
}
