pub mod console;
pub mod interface;
pub mod metadata;
pub mod parameters;
pub mod consts;

use crate::{interface::Interface, metadata::PluginMetadata, parameters::PluginParameters};
use std::sync::Arc;
use nih_plug_egui::{ egui::{ Ui } };
use nih_plug::context::gui::ParamSetter;

pub struct Plugin<T: PluginImplementation> {
    metadata: PluginMetadata,
    implementation: T
}

impl<T: PluginImplementation> Plugin<T> {
    pub fn new(metadata: PluginMetadata, implementation: T) -> Self {
        Self {
            metadata,
            implementation
        }
    }
}

// NOTE FOR FUTURE SELF
// Ideally the only implementation done ever would be in the implementation of this base class.
// This means depencency injection for a number of functions. 
// E.g. interface_center, interface_bar, runtime_process, runtime_init, etc.
// E.g. fn interface_center(&self) -> impl Fn(&dyn PluginImplementation, &mut Ui, &ParamSetter) + 'static + Send + Sync;
pub trait PluginImplementation {
    fn metadata(&self) -> PluginMetadata;
    fn params(&self) ->  Arc<dyn PluginParameters>;
}