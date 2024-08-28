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

#[derive(Component)]
struct UiSelectionHighlight;

fn selection_proxy_highlight(
    mut query : Query<(&mut BorderColor,&crate::galaxy::selection::SelectionProxy),With<UiSelectionHighlight>>,
    selection : Res<crate::galaxy::Selection>
) {
    for (mut border, proxy) in query.iter_mut() {
        *border = if proxy.resolved_target == selection.hovered {
            if proxy.resolved_target== selection.selected {
                Color::srgb(1.0,80./255.,0.)
            } else {
                Color::WHITE
            }
        } else if proxy.resolved_target == selection.selected {
            Color::srgb(1.0,165./255.,0.)
        } else {
            Color::srgb(0.1,0.1,0.1)
        }.into();
    }
}

impl Plugin for InterfacePlugin {
    fn build(&self, app : &mut App) {
        app.add_plugins((
            time_widget::TimeWidgetPlugin,
            selection_panel::SelectionPanelPlugin,
            empire_panel::EmpirePanelPlugin
        ))
            .add_systems(Update, (
                selection_proxy_highlight,
                time_control::time_control_system,
                star_label::draw_star_labels,
                star_label::add_star_labels).after(crate::camera::camera_control_system));
    }
}