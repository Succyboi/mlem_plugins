use nih_plug_egui::egui::{
    self, vec2, Response, Ui, Widget
};
use nih_plug::{params::BoolParam, prelude::{Param, ParamSetter}};

use crate::interface::interface_utils::param_info;

/// A slider widget similar to [`egui::widgets::Slider`] that knows about NIH-plug parameters ranges
/// and can get values for it. The slider supports double click and control click to reset,
/// shift+drag for granular dragging, text value entry by clicking on the value text.
///
/// TODO: Vertical orientation
/// TODO: Check below for more input methods that should be added
/// TODO: Decouple the logic from the drawing so we can also do things like nobs without having to
///       repeat everything
/// TODO: Add WidgetInfo annotations for accessibility
#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct ParamToggle<'a, P: Param> {
    param: &'a P,
    setter: &'a ParamSetter<'a>,

    /// Will be set in the `ui()` function so we can request keyboard input focus on Alt+click.
    keyboard_focus_id: Option<egui::Id>,
}

impl<'a, P: Param> ParamToggle<'a, P> {
    /// Create a new slider for a parameter. Use the other methods to modify the slider before
    /// passing it to [`Ui::add()`].
    pub fn for_param(param: &'a P, setter: &'a ParamSetter<'a>) -> Self {
        Self {
            param,
            setter,

            keyboard_focus_id: None,
        }
    }

    fn plain_value(&self) -> P::Plain {
        self.param.modulated_plain_value()
    }

    fn bool_value(&self) -> bool {
        self.param.modulated_normalized_value() > 0.5
    }

    fn string_value(&self) -> String {
        self.param.to_string()
    }

    fn begin_drag(&self) {
        self.setter.begin_set_parameter(self.param);
    }

    /// Begin and end drag still need to be called when using this. Returns `false` if the string
    /// could no tbe parsed.
    fn set_from_bool(&self, bool: &bool) {
        let value = self.param.preview_plain(if *bool { 1.0 } else { 0.0 });
        if value != self.plain_value() {
            self.setter.set_parameter(self.param, value);
        }
    }

    /// Begin and end drag still need to be called when using this..
    fn reset_param(&self) {
        self.setter
            .set_parameter(self.param, self.param.default_plain_value());
    }

    fn end_drag(&self) {
        self.setter.end_set_parameter(self.param);
    }
}

impl Widget for ParamToggle<'_, BoolParam> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            // Allocate an automatic ID for keeping track of keyboard focus state
            // FIXME: There doesn't seem to be a way to generate IDs in the public API, not sure how
            //        you're supposed to do this
            let (kb_edit_id, _) = ui.allocate_space(vec2(0.0, 0.0));
            self.keyboard_focus_id = Some(kb_edit_id);

            let original_value = self.bool_value();
            let mut bool_value = original_value;
            let response = ui.toggle_value(&mut bool_value, self.string_value());
            
            if original_value != bool_value {
                self.begin_drag();
                self.set_from_bool(&bool_value);
                self.end_drag();
            }

            if response.secondary_clicked() || response.middle_clicked() {
                self.begin_drag();
                self.reset_param();
                self.end_drag();
            }

            response.on_hover_ui(|ui| {
                param_info(ui, self.param);
            })
        })
        .inner
    }
}