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

pub trait PluginImplementation {
    fn metadata() -> PluginMetadata;
    fn params(&self) ->  Arc<dyn PluginParameters>;
    fn interface_center(&self) -> impl Fn(&mut Ui, &ParamSetter) + '_ + Send + Sync;
}