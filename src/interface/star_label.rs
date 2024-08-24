use bevy::prelude::*;
use crate::galaxy::Star;
use bevy_mod_picking::prelude::*;

#[derive(Component)]
pub struct StarLabel {
    star : Entity,
    label_text : Entity,
    icons_banner : Entity
}

struct StarLabelConsts {}

impl StarLabelConsts {
    const BANNER_HEIGHT_RATIO : f32 = 0.333f32;
}

pub fn add_star_labels(
    stars : Query<(&Star,Entity),Added<Star>>,
    mut commands : Commands
) {
    for (star, entity) in stars.iter() {
        let label = &star.name;
        let text_fps = commands.spawn((
            TextBundle {
                // use two sections, so it is easy to update just the number
                text: Text::from_sections([
                    TextSection {
                        value: label.into(),
                        style: TextStyle {
                            font_size: 11.0,
                            color: Color::WHITE,
                            // font: my_font_handle
                            ..default()
                        }
                    },
                ])
                .with_justify(JustifyText::Center),
                ..Default::default()
            }
            .with_no_wrap(),
            Pickable::IGNORE
        )).id();

        let icons_banner = commands.spawn((
            NodeBundle {
                // give it a dark background for readability
                background_color: BackgroundColor(Color::WHITE.with_alpha(1.0)),
                // make it "always on top" by setting the Z index to maximum
                z_index: ZIndex::Global(100),
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(1.0)),
                    ..Default::default()
                },
                visibility : Visibility::Hidden,
                ..Default::default()
            },
            Pickable::IGNORE
        )).id();

        let holder = commands.spawn((
            NodeBundle {
                // give it a dark background for readability
                //background_color: BackgroundColor(Color::RED.with_a(0.5)),
                //z_index: ZIndex::Global(100),
                style: Style {
                    flex_direction : FlexDirection::Column,
                    align_items : AlignItems::Center,
                    position_type: PositionType::Absolute,
                    justify_content : JustifyContent::Center,
                    //width: Val::Px(64.),
                    //height: Val::Px(12.),
                    //height: Val::Px(48.),
                    // position it at the top-right corner
                    // 1% away from the top window edge
                    right: Val::Auto,
                    top: Val::Auto,
                    // set bottom/left to Auto, so it can be
                    // automatically sized depending on the text
                    bottom: Val::Auto,
                    left: Val::Auto,
                    // give it some padding for readability
                    //padding: UiRect::all(Val::Px(4.0)),
                    ..Default::default()
                },
                visibility: Visibility::Visible,
                ..Default::default()
            },
            Pickable::IGNORE,
            StarLabel { star: entity, label_text : text_fps, icons_banner },
        )).id();
        // create our text



        commands.entity(holder).push_children(&[text_fps, icons_banner]);

        // add children for icons bannerr

        let icon_1 = commands.spawn((
            NodeBundle {
                // give it a dark background for readability
                background_color: BackgroundColor(Color::srgb(0.0,1.0,0.0)),
                // make it "always on top" by setting the Z index to maximum
                // we want it to be displayed over all other UI
                //z_index: ZIndex::Global(101),
                style: Style {
                    width: Val::Percent(100.0 * StarLabelConsts::BANNER_HEIGHT_RATIO),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(1.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            Pickable::IGNORE
        )).id();
        let icon_2 = commands.spawn((
            NodeBundle {
                // give it a dark background for readability
                background_color: BackgroundColor(Color::srgb(1.0,0.0,0.0)),
                // make it "always on top" by setting the Z index to maximum
                // we want it to be displayed over all other UI
                //z_index: ZIndex::Global(i32::MAX-1),
                style: Style {
                    width: Val::Percent(100.0 * StarLabelConsts::BANNER_HEIGHT_RATIO),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(1.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            Pickable::IGNORE
        )).id();
        let icon_3 = commands.spawn((
            NodeBundle {
                // give it a dark background for readability
                background_color: BackgroundColor(Color::srgb(0.0,0.0,1.0)),
                // make it "always on top" by setting the Z index to maximum
                // we want it to be displayed over all other UI
                //z_index: ZIndex::Global(i32::MAX-1),
                style: Style {
                    width: Val::Percent(100.0 * StarLabelConsts::BANNER_HEIGHT_RATIO),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(1.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            Pickable::IGNORE
        )).id();

        commands.entity(icons_banner).push_children(&[icon_1,icon_2,icon_3]);
    }
}


pub fn draw_star_labels(
    star_query : Query<&Star,Without<StarLabel>>,
    mut text_query : Query<&mut Text,(Without<Camera>,Without<StarLabel>)>,
    mut starlabel_query : Query<(&mut Style,&mut Visibility,&StarLabel),Without<Camera>>,
    mut style_query : Query<&mut Style,(Without<StarLabel>,Without<Text>,Without<Camera>)>,
    camera_query: Query<(&Camera, &Transform),Changed<Transform>>,
) {
    let (camera, camera_transform) = camera_query.single();

    let g_transform = GlobalTransform::from(*camera_transform);

    for (mut style, mut visibility, starlabel) in starlabel_query.iter_mut() {
        let star = star_query.get(starlabel.star).expect("Star label could not retrieve star");

        *visibility = Visibility::Hidden;

        let Some(pos) = camera.world_to_viewport(&g_transform, star.pos) else { continue; };
        let Some(tl) = camera.world_to_viewport(&g_transform, star.pos + Vec3::new(10.0,0.0,-5.0)) else { continue; };
        let Some(br) = camera.world_to_viewport(&g_transform, star.pos + Vec3::new(-10.0,0.0,-10.0)) else { continue; };

        *visibility = Visibility::Inherited;

        let w = br.x - tl.x;
        let h = w * StarLabelConsts::BANNER_HEIGHT_RATIO;//br.y - tl.y;

        style.left = Val::Px(pos.x - w/2.0);
        style.top = Val::Px(tl.y);
        style.width = Val::Px(w);
        style.height = Val::Auto;//Px(h);

        let font_target = (h + 4.0).round().clamp(8.0,16.0);

        if let Ok(mut text) = text_query.get_mut(starlabel.label_text) {
            for ts in &mut text.sections {
                ts.style.font_size = font_target;
            }
        }

         
        if let Ok(mut node) = style_query.get_mut(starlabel.icons_banner) {
            let w = w.min(64.);
            let h = w * StarLabelConsts::BANNER_HEIGHT_RATIO;
            node.width = Val::Px(w);
            node.height = Val::Px(h);
        }
        
    }
}