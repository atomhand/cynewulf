use bevy::prelude::*;

mod time_widget;
mod time_control;
mod star_label;
mod hovered_item_widget;
mod selection_panel;
mod empire_panel;

mod hud;

struct UiConsts;

impl UiConsts {
    const STANDARD_UI_FONT_SIZE : f32 = 20.0;
}

pub struct InterfacePlugin;

impl Plugin for InterfacePlugin {
    fn build(&self, app : &mut App) {
        app.add_plugins((
            time_widget::TimeWidgetPlugin,
            selection_panel::SelectionPanelPlugin,
            empire_panel::EmpirePanelPlugin
        ))
            .add_systems(Update, (
                time_control::time_control_system,
                star_label::draw_star_labels,
                star_label::add_star_labels).after(crate::camera::camera_control_system));
    }
}