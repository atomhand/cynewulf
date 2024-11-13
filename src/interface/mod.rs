use bevy::prelude::*;

mod time_widget;
mod time_control;
mod star_label;
mod hovered_item_widget;
mod selection_panel;
mod selected_fleet;
mod empire_panel;
mod empire_outliner;
mod hud;

struct UiConsts;

impl UiConsts {
    const STANDARD_UI_FONT_SIZE : f32 = 20.0;
}

pub struct InterfacePlugin;

#[derive(Component)]
struct UiSelectionHighlight;

fn selection_proxy_highlight(
    mut query : Query<(&mut BorderColor,&crate::galaxy::selection::SelectionProxy),With<UiSelectionHighlight>>,
    selection : Res<crate::galaxy::Selection>
) {
    for (mut border, proxy) in query.iter_mut() {
        *border = proxy.resolved_target.and_then(|target|
            Some(selection.get_selection_state(target).as_colour_with_default(Color::linear_rgb(0.1,0.1,0.1)).into())
        ).unwrap_or(Color::NONE.into());
    }
}

impl Plugin for InterfacePlugin {
    fn build(&self, app : &mut App) {
        app
        .add_plugins((
            time_widget::TimeWidgetPlugin,
            selection_panel::SelectionPanelPlugin,
            selected_fleet::FleetSelectionPanelPlugin,
            empire_panel::EmpirePanelPlugin,
            empire_outliner::EmpireOutlinerPlugin
        ))
        .add_systems(Update, (
            selection_proxy_highlight,
            time_control::time_control_system,
            star_label::draw_star_labels,
            star_label::add_star_labels).after(crate::camera::camera_control_system));
    }
}