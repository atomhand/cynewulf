use crate::galaxy::selection::{InterfaceIdentifier, SelectionProxy};
use crate::galaxy::Description;
use crate::galaxy::{GalaxyConfig, Selection};
use crate::prelude::*;
use bevy::prelude::*;

/// Marker to find the text entity so we can update it
#[derive(Component)]
struct SelectionPanel;

#[derive(Component)]
struct SelectionPanelTabRoot {
    slot: i32,
}

#[derive(Component)]
struct SelectionPanelTabHeader {
    slot: i32,
}
#[derive(Component)]
struct SelectionPanelTabDetails {
    slot: i32,
}

pub struct SystemOutlinerPlugin;

impl Plugin for SystemOutlinerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_widget)
            .add_systems(PostUpdate, update_widget_system);
    }
}

fn setup_widget(mut commands: Commands) {
    commands
        .spawn((
            SelectionPanel,
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                max_width: Val::Percent(20.),
                width: Val::Auto,
                height: Val::Auto,
                left: Val::Percent(1.),
                bottom: Val::Percent(1.),
                top: Val::Auto,
                right: Val::Auto,
                border: UiRect::all(Val::Px(4.0)),
                padding: UiRect::all(Val::Px(1.0)),
                ..Default::default()
            },
            BackgroundColor(Color::BLACK.with_alpha(1.0)),
            BorderColor(Color::srgb(0.1, 0.1, 0.2)),
            GlobalZIndex(i32::MAX - 1),
        ))
        .with_children(|parent| {
            for i in 0..GalaxyConfig::MAX_SYSTEM_BODIES {
                parent
                    .spawn((
                        SelectionPanelTabRoot { slot: i as i32 },
                        SelectionProxy::new(InterfaceIdentifier::CurrentSystemOrbiter(i as u32)),
                        super::UiSelectionHighlight,
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            position_type: PositionType::Relative,
                            justify_content: JustifyContent::FlexStart,
                            width: Val::Auto, //(100.),
                            border: UiRect::all(Val::Px(4.0)),
                            padding: UiRect::all(Val::Px(2.0)),
                            margin: UiRect::all(Val::Px(1.0)),
                            height: Val::Auto,
                            ..Default::default()
                        },
                        BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                        GlobalZIndex(i32::MAX),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            SelectionPanelTabHeader { slot: i as i32 },
                            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.2)),
                            Text("Header text".to_string()),
                            Pickable {
                                should_block_lower: false,
                                is_hoverable: false,
                            },
                        ));
                        parent.spawn((
                            SelectionPanelTabDetails { slot: i as i32 },
                            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.75)),
                            Text("Tab details text".to_string()),
                            Pickable {
                                should_block_lower: false,
                                is_hoverable: false,
                            },
                        ));
                    });
            }
        });
}

fn update_widget_system(
    mut root_query: Query<(&mut Node, &mut BackgroundColor, &SelectionPanelTabRoot)>,
    mut header_query: Query<(&mut Text, &SelectionPanelTabHeader), Without<SelectionPanelTabRoot>>,
    mut details_query: Query<
        (&mut Text, &mut Node, &SelectionPanelTabDetails),
        (
            Without<SelectionPanelTabRoot>,
            Without<SelectionPanelTabHeader>,
        ),
    >,
    selection: Res<Selection>,
    description_query: Query<&Description, Without<SelectionPanelTabHeader>>,
    star_query: Query<&Star, Without<SelectionPanelTabHeader>>,
    planet_colony_query: Query<(&Planet, Option<&Colony>), Without<SelectionPanelTabHeader>>,
) {
    if selection.is_changed() {
        let Some(star_entity) = selection.selected_system else {
            for (mut style, _, _) in root_query.iter_mut() {
                style.display = Display::None;
            }
            return;
        };
        let Ok(star) = star_query.get(star_entity) else {
            return;
        };
        let star_and_orbiters = &star.orbiters;

        let desc = star_and_orbiters
            .iter()
            .map(|x| description_query.get(*x).unwrap())
            .collect::<Vec<_>>();

        let len = star_and_orbiters.len() as i32;

        for (mut text, panel) in header_query.iter_mut() {
            if panel.slot < len {
                let mut t_name = desc[panel.slot as usize].type_name().to_string();
                if let Ok((_planet, Some(colony))) =
                    planet_colony_query.get(star_and_orbiters[panel.slot as usize])
                {
                    t_name = format!(
                        "({}, {})",
                        colony.population.to_string(),
                        desc[panel.slot as usize].type_name()
                    );
                }
                *text = Text(format!("{} ({})", desc[panel.slot as usize].name, t_name));
            }
        }
        for (mut text, mut style, panel) in details_query.iter_mut() {
            if panel.slot < len {
                style.display = Display::None;
                if Some(star_and_orbiters[panel.slot as usize]) == selection.selected {
                    // try grab colony
                    if let Ok((planet, colony)) =
                        planet_colony_query.get(star_and_orbiters[panel.slot as usize])
                    {
                        let colony_details = if let Some(colony) = colony {
                            format!("\n\n{}\n\n{}", colony.population.details(), colony.economy)
                        } else {
                            "".to_string()
                        };

                        *text = Text(format!(
                            "Size: {} | Insolation: {}{}",
                            planet.radius, planet.insolation, colony_details
                        ));
                    } else {
                        *text = Text(format!(
                            "Panel Details for body {}",
                            desc[panel.slot as usize].name
                        ));
                    }

                    //text.sections[0].style.color = Color::srgb(0.25,0.25,1.0);
                    //text.sections[1].style.color = Color::srgb(0.25,0.25,1.0);

                    style.display = Display::Flex;
                }
            }
        }
        for (mut style, mut bg, panel) in root_query.iter_mut() {
            if panel.slot < len {
                *bg = desc[panel.slot as usize]
                    .empire_color
                    .unwrap_or(Color::srgb(0.1, 0.1, 0.1))
                    .into();

                style.display = Display::Flex;
            } else {
                style.display = Display::None;
            }
        }
    }
}
