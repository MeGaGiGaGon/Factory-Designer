use std::any::{Any, TypeId};

use eframe::egui::Ui;

/// Add an output connector to a node
/// Can optionally be created with a ui callback and output callback
/// The ui callback will be shown to the left of the node
/// The output callback will give the value to be propogated along new connections
/// A node without an output callback cannot connect to any input nodes
/// Output nodes can only connect to input nodes of the same type
pub struct NodeOutput<'a, 'b> {
    pub ui_callback: Box<dyn FnOnce(&mut Ui) + 'a>,
    pub output_callback: Box<dyn FnOnce() -> Box<dyn Any> + 'b>,
    pub output_type: TypeId,
}

/// Unique internal type to prevent output callbackless nodes from connecting
/// Input callbackless nodes use a different type and thus also can't be connected to
struct EmptyNodeOutput {}

impl<'a, 'b> NodeOutput<'a, 'b> {
    /// Create a new NodeOutput, with both a ui and output callback
    pub fn new<T: 'static>(
        ui_callback: impl FnOnce(&mut Ui) + 'a,
        output_callback: impl FnOnce() -> T + 'b,
    ) -> Self {
        Self {
            ui_callback: Box::new(ui_callback),
            output_callback: Box::new(|| Box::new(output_callback())),
            output_type: TypeId::of::<T>(),
        }
    }

    /// Create a new NodeOutput with only a ui callback
    pub fn ui(ui_callback: impl FnOnce(&mut Ui) + 'a) -> Self {
        Self {
            ui_callback: Box::new(ui_callback),
            output_callback: Box::new(|| Box::new(EmptyNodeOutput {})),
            output_type: TypeId::of::<EmptyNodeOutput>(),
        }
    }

    /// Create a new NodeOutput with only an output callback
    pub fn output<T: 'static>(output_callback: impl FnOnce() -> T + 'b) -> Self {
        Self {
            ui_callback: Box::new(|_| {}),
            output_callback: Box::new(|| Box::new(output_callback())),
            output_type: TypeId::of::<T>(),
        }
    }

    /// Create a new NodeOutput with no callbacks
    pub fn none() -> Self {
        Self {
            ui_callback: Box::new(|_| {}),
            output_callback: Box::new(|| Box::new(EmptyNodeOutput {})),
            output_type: TypeId::of::<EmptyNodeOutput>(),
        }
    }
}
