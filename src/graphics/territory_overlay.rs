use delaunator::{triangulate, Point};

use crate::galaxy::{OverlaysTriangulationVertex, Star};
use crate::prelude::*;
use bevy::prelude::*;
use bevy::render::storage::ShaderStorageBuffer;

use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    reflect::TypePath,
    render::{
        mesh::{Indices, MeshVertexAttribute, MeshVertexBufferLayoutRef},
        render_asset::RenderAssetUsages,
        render_resource::*,
    },
};
// https://docs.rs/bevy/latest/bevy/pbr/prelude/trait.Material.html

#[derive(ShaderType, Default, Debug, Clone)]
#[repr(C)]
struct LaneFormat {
    enabled: u32,
    color: Vec3,
}

#[derive(ShaderType, Default, Debug, Clone)]
#[repr(C)]
struct StarFormat {
    pos: Vec4,
    color: Vec4,
    empire_halo: Vec4,
    system_halo: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct TerritoryOverlaysMaterial {
    #[storage(1, read_only)]
    star_data_buffer: Handle<ShaderStorageBuffer>,
    //star_data_buffer : Vec<StarFormat>,
    #[storage(2, read_only)]
    //edge_data_buffer : Vec<LaneFormat>,
    edge_data_buffer: Handle<ShaderStorageBuffer>,
    alpha_mode: AlphaMode,
}

const ATTRIBUTE_BARYCENTRIC: MeshVertexAttribute =
    MeshVertexAttribute::new("Barycentric", 2137464976, VertexFormat::Float32x3);

const ATTRIBUTE_STARID: MeshVertexAttribute =
    MeshVertexAttribute::new("StarID", 988540917, VertexFormat::Uint32x3);

const ATTRIBUTE_EDGEID: MeshVertexAttribute =
    MeshVertexAttribute::new("EdgeID", 422059518, VertexFormat::Uint32x3);

impl Material for TerritoryOverlaysMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/shader_territory_overlay.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/shader_territory_overlay.wgsl".into()
    }
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_STARID.at_shader_location(1),
            ATTRIBUTE_BARYCENTRIC.at_shader_location(2),
            ATTRIBUTE_EDGEID.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

#[derive(Component)]
struct StarGfxTag {
    id: usize,
    nearest: f32, // distance to nearest other star
}

#[derive(Resource)]
struct OverlaysData {
    _mesh_handle: Handle<Mesh>,
    star_data: Vec<StarFormat>,
    edge_data: Vec<LaneFormat>,
    material_handle: Handle<TerritoryOverlaysMaterial>,
}

pub struct OverlaysPlugin;

impl Plugin for OverlaysPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<TerritoryOverlaysMaterial>::default())
            .add_systems(BuildGalaxyGraphics, generate_overlays_mesh)
            .add_systems(BuildGalaxyGraphics, update_selection_radius)
            .add_systems(Update, update_overlays);
    }
}

fn update_overlays(
    star_update_query: Query<(Entity, &StarGfxTag, &StarClaim), With<Star>>,
    star_changed_update_query: Query<
        (Entity, &StarGfxTag, &StarClaim),
        (With<Star>, Changed<StarClaim>),
    >,
    stars_query: Query<&Star>,
    empire_query: Query<&Empire>,
    mut mats: ResMut<Assets<TerritoryOverlaysMaterial>>,
    mut overlays_data: ResMut<OverlaysData>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    selection: Res<crate::galaxy::Selection>,
    hypernet: Res<Hypernet>,
) {
    let mut any_change = false;
    if selection.is_changed() {
        any_change = true;
        for edge in &mut overlays_data.edge_data {
            edge.color = Vec3::new(1.0, 0.75, 0.0);
        }
        if let Some(star_b) = selection.hovered.and_then(|b| stars_query.get(b).ok()) {
            if let Some(star_a) = selection
                .selected_system
                .and_then(|a| stars_query.get(a).ok())
            {
                if let Some(path) = hypernet.find_path(star_a.node_id, star_b.node_id) {
                    for edge in path.edges {
                        overlays_data.edge_data[edge as usize].color = Vec3::new(1.0, 0.0, 0.0);
                    }
                }
            }
        }

        // if selection is changed, update halos for all stars
        // otherwise only for stars whose starclaim has changed
        // Could be a little more elegant!

        for (entity, tag, claim) in &star_update_query {
            let col: Srgba = if let Some(owner) = claim.owner {
                empire_query.get(owner).unwrap().color.to_srgba()
            } else {
                Srgba::new(0.0, 0.0, 0.0, 0.0)
            };

            let empire_halo_col = claim
                .owner
                .and_then(|empire| Some(selection.get_selection_state(empire).as_colour()))
                .unwrap_or(Color::NONE);
            let selection_halo = selection.get_selection_state(entity).as_colour();

            overlays_data.star_data[tag.id as usize].system_halo =
                selection_halo.to_linear().to_vec4();
            overlays_data.star_data[tag.id as usize].empire_halo =
                empire_halo_col.to_linear().to_vec4();
            overlays_data.star_data[tag.id as usize].color = col.to_vec4();
        }
    } else {
        for (entity, tag, claim) in &star_changed_update_query {
            let col: Srgba = if let Some(owner) = claim.owner {
                empire_query.get(owner).unwrap().color.to_srgba()
            } else {
                Srgba::new(0.0, 0.0, 0.0, 0.0)
            };

            let empire_halo_col = claim
                .owner
                .and_then(|empire| Some(selection.get_selection_state(empire).as_colour()))
                .unwrap_or(Color::NONE);
            let selection_halo = selection.get_selection_state(entity).as_colour();

            overlays_data.star_data[tag.id as usize].system_halo =
                selection_halo.to_linear().to_vec4();
            overlays_data.star_data[tag.id as usize].empire_halo =
                empire_halo_col.to_linear().to_vec4();
            overlays_data.star_data[tag.id as usize].color = col.to_vec4();

            any_change = true;
        }
    }

    if any_change {
        let Some(mat) = mats.get_mut(&overlays_data.material_handle) else {
            return;
        };
        if let Some(star_buffer) = buffers.get_mut(&mat.star_data_buffer) {
            star_buffer.set_data(overlays_data.star_data.as_slice());
        }
        if let Some(edge_buffer) = buffers.get_mut(&mat.edge_data_buffer) {
            edge_buffer.set_data(overlays_data.edge_data.as_slice());
        }
    }
}

use crate::galaxy::selection::GalaxySelectable;
fn update_selection_radius(mut query: Query<(&StarGfxTag, &mut GalaxySelectable)>) {
    for (tag, mut selectable) in query.iter_mut() {
        selectable.radius = tag.nearest * 0.5 - 1.0;
    }
}

fn generate_overlays_mesh(
    query: Query<(
        Entity,
        &Transform,
        &OverlaysTriangulationVertex,
        Option<&Star>,
    )>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerritoryOverlaysMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    hypernet: Res<Hypernet>,
) {
    let mut points = Vec::<Point>::new();
    let mut in_verts = Vec::<Vec3>::new();
    let mut in_ids = Vec::<u32>::new();

    let mut star_data = vec![StarFormat::default(); hypernet.graph.capacity().0];

    for (entity, transform, overlays_vertex, star) in &query {
        let p = transform.translation;
        in_verts.push(Vec3::new(p.x, -0.01, p.z));
        points.push(Point {
            x: p.x as f64,
            y: p.z as f64,
        });

        let id = overlays_vertex.node_id;

        let mut nearest = f32::MAX;

        for (_entity, transform, _overlays_vertex, _star) in &query {
            let d = p.xz().distance(transform.translation.xz());
            if d > 0.5 {
                nearest = f32::min(nearest, d);
            }
        }

        in_ids.push(id);
        if let Some(_star) = star {
            commands.entity(entity).insert(StarGfxTag {
                id: id as usize,
                nearest,
            });
        }

        // distance to nearest neighbour stored in A channel

        star_data[id as usize] = StarFormat {
            pos: Vec4::new(p.x, p.z, 0.0, nearest),
            ..default()
        };
    }

    let triangulation = triangulate(&points);

    let mut verts = Vec::<Vec3>::new();
    let mut bary = Vec::<Vec3>::new();
    let mut star_ids = Vec::<UVec3>::new();
    let mut star_distances = Vec::<Vec3>::new();

    let mut lane_ids = Vec::<UVec3>::new();

    let n = triangulation.triangles.len();
    let tris = triangulation.triangles;
    let mut t = 0;
    let mut indices = Vec::<u32>::new();

    while t < n {
        bary.push(Vec3::X);
        bary.push(Vec3::Y);
        bary.push(Vec3::Z);

        let c = UVec3::new(in_ids[tris[t]], in_ids[tris[t + 1]], in_ids[tris[t + 2]]);
        let e = UVec3::new(
            hypernet
                .graph
                .find_edge(c.x.into(), c.y.into())
                .and_then(|x| Some(x.index() as u32))
                .unwrap_or(100000),
            hypernet
                .graph
                .find_edge(c.y.into(), c.z.into())
                .and_then(|x| Some(x.index() as u32))
                .unwrap_or(100000),
            hypernet
                .graph
                .find_edge(c.x.into(), c.z.into())
                .and_then(|x| Some(x.index() as u32))
                .unwrap_or(100000),
        );

        let pos_a = in_verts[in_ids[tris[t]] as usize];
        let pos_b = in_verts[in_ids[tris[t + 1]] as usize];
        let pos_c = in_verts[in_ids[tris[t + 2]] as usize];

        for i in 0..3 {
            let p = in_verts[tris[t + i]];

            star_ids.push(c);
            lane_ids.push(e);
            star_distances.push(Vec3::new(
                p.distance(pos_a),
                p.distance(pos_b),
                p.distance(pos_c),
            ));
            indices.push(t as u32 + i as u32);
            verts.push(p);
        }

        t += 3;
    }

    //
    let mut edge_data = vec![
        LaneFormat {
            enabled: 0,
            color: Vec3::new(1.0, 0.75, 0.0)
        };
        hypernet.graph.capacity().1
    ];
    for edge in hypernet.graph.edge_indices() {
        edge_data[edge.index()].enabled = 1;
    }

    let mesh = meshes.add(
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verts)
        .with_inserted_attribute(ATTRIBUTE_BARYCENTRIC, bary)
        .with_inserted_attribute(ATTRIBUTE_STARID, star_ids)
        .with_inserted_attribute(ATTRIBUTE_EDGEID, lane_ids)
        .with_inserted_indices(Indices::U32(indices)),
    );

    let material = materials.add(TerritoryOverlaysMaterial {
        star_data_buffer: buffers.add(ShaderStorageBuffer::from(star_data.clone())),
        edge_data_buffer: buffers.add(ShaderStorageBuffer::from(edge_data.clone())),
        alpha_mode: AlphaMode::Blend,
    });

    commands.insert_resource(OverlaysData {
        _mesh_handle: mesh.clone(),
        star_data,
        edge_data,
        material_handle: material.clone(),
    });

    commands.spawn((Mesh3d(mesh), MeshMaterial3d(material)));
}
