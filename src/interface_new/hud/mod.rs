use bevy::prelude::*;
mod empire_outliner;
mod system_outliner;
mod time_widget;
pub struct HudPlugin;

struct UiConsts;

impl UiConsts {
    const STANDARD_UI_FONT_SIZE: f32 = 20.0;
}

#[derive(Component)]
struct UiSelectionHighlight;

fn selection_proxy_highlight(
    mut query: Query<
        (&mut BorderColor, &crate::galaxy::selection::SelectionProxy),
        With<UiSelectionHighlight>,
    >,
    selection: Res<crate::galaxy::Selection>,
) {
    for (mut border, proxy) in query.iter_mut() {
        *border = proxy
            .resolved_target
            .map(|target| {
                selection
                    .get_selection_state(target)
                    .as_colour_with_default(Color::linear_rgb(0.1, 0.1, 0.1))
                    .into()
            })
            .unwrap_or(Color::NONE.into());
    }
}

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, selection_proxy_highlight);
        app.add_plugins((
            time_widget::TimeWidgetPlugin,
            empire_outliner::EmpireOutlinerPlugin,
            system_outliner::SystemOutlinerPlugin,
        ));
    }
}
