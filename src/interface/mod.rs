use bevy::prelude::*;

mod time_widget;
mod time_control;
mod star_label;
mod hovered_item_widget;
mod selection_panel;

mod hud;

pub struct InterfacePlugin;

struct UIConsts;

impl Plugin for InterfacePlugin {
    fn build(&self, app : &mut App) {
        app.add_plugins((
            hovered_item_widget::HoverWidgetPlugin,
            time_widget::TimeWidgetPlugin,
            selection_panel::SelectionPanelPlugin
        ))
            .add_systems(Update, (
                time_control::time_control_system,
                star_label::draw_star_labels,
                star_label::add_star_labels).after(crate::camera_control::camera_control_system));
    }
}