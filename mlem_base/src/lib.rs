pub mod console;
pub mod interface;
pub mod metadata;
pub mod consts;

use crate::{interface::Interface, metadata::PluginMetadata};
use nih_plug_egui::{ egui::{ Ui } };

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

    pub fn create_interface() {
        
    }
}

pub trait PluginImplementation {
    fn metadata() -> PluginMetadata;
    fn interface(&self, ui: &mut Ui) -> impl FnOnce();
}