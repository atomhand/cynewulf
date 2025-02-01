use bevy::prelude::*;
mod time_widget;
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app : &mut App ) {
        app.add_plugins(time_widget::TimeWidgetPlugin);
    }
}