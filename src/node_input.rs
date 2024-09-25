use std::any::{Any, TypeId};

use eframe::egui::Ui;

/// Add an input connector to a node
/// Can optionally be created with a ui callback and input callback
/// The ui callback will be shown to the right of the node
/// The input callback will be given the value propogated from new connections
/// A node without an input callback cannot connect to any input nodes
/// Input nodes can only connect to output nodes of the same type
pub struct NodeInput<'a, 'b> {
    pub ui_callback: Box<dyn FnOnce(&mut Ui) + 'a>,
    pub input_callback: Box<dyn FnOnce(Box<dyn Any>) + 'b>,
    pub input_type: TypeId,
}

/// Unique internal type to prevent input callbackless nodes from connecting
/// Output callbackless nodes use a different type and thus also can't be connected to
enum EmptyNodeInput {}

impl<'a, 'b> NodeInput<'a, 'b> {
    /// Create a new NodeInput, with both a ui and input callback
    pub fn new<T: 'static>(
        ui_callback: impl FnOnce(&mut Ui) + 'a,
        input_callback: impl FnOnce(T) + 'b,
    ) -> Self {
        Self {
            ui_callback: Box::new(ui_callback),
            input_callback: Box::new(|x| input_callback(*x.downcast::<T>().unwrap())),
            input_type: TypeId::of::<T>(),
        }
    }

    /// Create a new NodeInput with only a ui callback
    pub fn ui(ui_callback: impl FnOnce(&mut Ui) + 'a) -> Self {
        Self {
            ui_callback: Box::new(ui_callback),
            input_callback: Box::new(|_| {}),
            input_type: TypeId::of::<EmptyNodeInput>(),
        }
    }

    /// Create a new NodeInput with only an input callback
    pub fn input<T: 'static>(input_callback: impl FnOnce(T) + 'b) -> Self {
        Self {
            ui_callback: Box::new(|_| {}),
            input_callback: Box::new(|x| input_callback(*x.downcast::<T>().unwrap())),
            input_type: TypeId::of::<T>(),
        }
    }

    /// Create a new NodeInput with no callbacks
    pub fn none() -> Self {
        Self {
            ui_callback: Box::new(|_| {}),
            input_callback: Box::new(|_| {}),
            input_type: TypeId::of::<EmptyNodeInput>(),
        }
    }
}
