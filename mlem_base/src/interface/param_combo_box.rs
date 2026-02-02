use nih_plug_egui::{egui::{
    self, Response, SelectableLabel, Separator, Ui, Widget, WidgetText, vec2, widgets
}};
use nih_plug::{params::{BoolParam, EnumParam, enums::EnumParamInner}, prelude::{Enum, Param, ParamSetter}};

use crate::interface::{PARAM_WIDTH, utils::{self, param_info}};

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
pub struct ParamComboBox<'a, P: Param> {
    param: &'a P,
    setter: &'a ParamSetter<'a>,

    /// Will be set in the `ui()` function so we can request keyboard input focus on Alt+click.
    keyboard_focus_id: Option<egui::Id>,
}

impl<'a, P: Param> ParamComboBox<'a, P> {
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

    fn begin_drag(&self) {
        self.setter.begin_set_parameter(self.param);
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

impl<T: Enum + PartialEq> Widget for ParamComboBox<'_, EnumParam<T>> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.set_width(PARAM_WIDTH);

            // Allocate an automatic ID for keeping track of keyboard focus state
            // FIXME: There doesn't seem to be a way to generate IDs in the public API, not sure how
            //        you're supposed to do this
            let (kb_edit_id, _) = ui.allocate_space(vec2(0.0, 0.0));
            self.keyboard_focus_id = Some(kb_edit_id);

            let original_value = self.param.normalized_value_to_string(self.param.modulated_normalized_value(), false);
            let mut value = original_value.clone();
            let step_count = self.param.step_count().expect("Parameter does not have a step count.") + 1;
            let mut values = Vec::new();

            for i in 0..step_count {
                values.push(self.param.normalized_value_to_string(i as f32 / (step_count as f32 - 1.0), false));
            }
            
            let response = egui::ComboBox::from_id_salt(self.param.name())
                .selected_text(value.clone())
                .show_ui(ui, |ui| {
                    for val in values {
                        ui.selectable_value(&mut value, val.clone(), val);
                    }
                }
            ).response;
            utils::fill_seperator(ui);

            if value != original_value {
                if let Some(value) = self.param.string_to_normalized_value(&value) {
                    self.begin_drag();
                    self.setter.set_parameter_normalized(self.param, value);
                    self.end_drag();
                }
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