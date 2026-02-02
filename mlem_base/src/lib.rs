pub mod console;
pub mod interface;
pub mod metadata;
pub mod parameters;
pub mod consts;
pub mod runtime;

use crate::{console::ConsoleSender, interface::Interface, metadata::PluginMetadata, parameters::PluginParameters};
use std::sync::Arc;
use nih_plug_egui::egui::{ Context, Ui };
use nih_plug::{context::gui::ParamSetter, params::Params};

// NOTE FOR FUTURE SELF
// Ideally the only implementation done ever would be in the implementation of this trait.
// This means depencency injection for a number of functions. 
// E.g. interface_center, interface_bar, runtime_process, runtime_init, etc.
pub trait PluginImplementation<T: PluginParameters>: 'static + Send + Sync  {
    fn new(params: Arc<T>) -> Self;
    fn metadata(&self) -> PluginMetadata;
    fn params(&self) ->  Arc<T>;
    
    fn interface_build(&self, ctx: &Context);
    fn interface_update_center(&self, ui: &mut Ui, ctx: &Context, setter: &ParamSetter);
    fn interface_update_bar(&self, ui: &mut Ui, ctx: &Context, setter: &ParamSetter);
}