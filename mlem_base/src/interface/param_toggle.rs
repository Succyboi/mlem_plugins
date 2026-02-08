use nih_plug_egui::{egui::{
    self, Response, SelectableLabel, Separator, Ui, Widget, WidgetText, vec2, widgets
}};
use nih_plug::{params::BoolParam, prelude::{Param, ParamSetter}};
use crate::interface::{PARAM_WIDTH, utils::{self, param_info}};

#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct ParamToggle<'a, P: Param> {
    param: &'a P,
    setter: &'a ParamSetter<'a>,

    true_str: &'a str,
    false_str: &'a str,

    /// Will be set in the `ui()` function so we can request keyboard input focus on Alt+click.
    keyboard_focus_id: Option<egui::Id>,
}

impl<'a, P: Param> ParamToggle<'a, P> {
    /// Create a new slider for a parameter. Use the other methods to modify the slider before
    /// passing it to [`Ui::add()`].
    pub fn for_param(param: &'a P, setter: &'a ParamSetter<'a>, true_str: &'a str, false_str: &'a str) -> Self {
        Self {
            param,
            setter,

            true_str,
            false_str,

            keyboard_focus_id: None,
        }
    }

    fn plain_value(&self) -> P::Plain {
        self.param.modulated_plain_value()
    }

    fn bool_value(&self) -> bool {
        self.param.modulated_normalized_value() > 0.5
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
            ui.set_width(PARAM_WIDTH);

            // Allocate an automatic ID for keeping track of keyboard focus state
            // FIXME: There doesn't seem to be a way to generate IDs in the public API, not sure how
            //        you're supposed to do this
            let (kb_edit_id, _) = ui.allocate_space(vec2(0.0, 0.0));
            self.keyboard_focus_id = Some(kb_edit_id);

            let original_value = self.bool_value();
            let mut bool_value = original_value;
            let text = if bool_value { self.true_str } else { self.false_str };
            let response = ui.toggle_value(&mut bool_value, text);
            utils::fill_seperator_available(ui); // TODO fix to max width based on content
            
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