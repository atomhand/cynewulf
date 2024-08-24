
use delaunator::{Point,triangulate};

use bevy::prelude::*;
use crate::galaxy::{Star,OverlaysTriangulationVertex};
use crate::simulation::data::{StarClaim,Empire};

use bevy::{
    reflect::TypePath,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    render::{
        mesh::{MeshVertexAttribute, Indices, MeshVertexBufferLayoutRef},
        render_asset::RenderAssetUsages,
        render_resource::*,
    },
};


#[derive(Asset,TypePath,AsBindGroup,Debug,Clone)]
struct TerritoryOverlaysMaterial {
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    star_position_texture: Option<Handle<Image>>,
    alpha_mode : AlphaMode,
}

const ATTRIBUTE_BARYCENTRIC: MeshVertexAttribute =
    MeshVertexAttribute::new("Barycentric", 2137464976, VertexFormat::Float32x3);

const ATTRIBUTE_STARID: MeshVertexAttribute =
    MeshVertexAttribute::new("StarID", 988540917, VertexFormat::Uint32x3);

const ATTRIBUTE_STARDIST: MeshVertexAttribute =
    MeshVertexAttribute::new("StarDist", 23642678092, VertexFormat::Float32x3);

impl Material for TerritoryOverlaysMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/shader_territory_overlay.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/shader_territory_overlay.wgsl".into()
    }
    fn specialize(
        _pipeline : &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_STARID.at_shader_location(1),
            ATTRIBUTE_BARYCENTRIC.at_shader_location(2),
            ATTRIBUTE_STARDIST.at_shader_location(3),
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
    id : usize
}

#[derive(Resource)]
struct OverlaysData {
    _mesh_handle : Handle<Mesh>,
    image_handle : Handle<Image>,
    material_handle : Handle<TerritoryOverlaysMaterial>
}

pub struct OverlaysPlugin;

impl Plugin for OverlaysPlugin {
    fn build(&self, app : &mut App) {
        app.add_plugins(MaterialPlugin::<TerritoryOverlaysMaterial>::default())
            .add_systems(Startup, generate_overlays_mesh.after(crate::galaxy::galaxy_generation::setup_stars))
            .add_systems(Update,update_overlays);
    }
}


fn update_overlays(
    query : Query<(&StarGfxTag,&StarClaim),(With<Star>,Changed<StarClaim>)>,
    empire_query : Query<&Empire>,
    mut images : ResMut<Assets<Image>>,
    mut mats : ResMut<Assets<TerritoryOverlaysMaterial>>,
    overlays_data : Res<OverlaysData>
) {
    let image = images.get_mut(&overlays_data.image_handle).unwrap();
    for (tag,claim) in &query {
        let col = if let Some(owner) = claim.owner {
            empire_query.get(owner).unwrap().color
        } else {
            Color::srgba(0.0,0.0,0.0,0.0)
        }.to_srgba().to_u8_array();

        image.data[tag.id * 4..tag.id * 4+4].copy_from_slice(&col);
    }

    // this marks the material as changed so that the texture update is registered
    mats.get_mut(&overlays_data.material_handle);
} 

fn generate_overlays_mesh(
    query : Query<(Entity,&Transform,Option<&Star>),With<OverlaysTriangulationVertex>>,
    mut commands : Commands,
    mut meshes : ResMut<Assets<Mesh>>,
    mut materials : ResMut<Assets<TerritoryOverlaysMaterial>>,
    mut textures : ResMut<Assets<Image>>,
) {
    let mut points = Vec::<Point>::new();
    let mut in_verts = Vec::<Vec3>::new();
    let mut in_ids = Vec::<u32>::new();

    const TEXTURE_SIZE : usize = 128;
    let colour_texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];

    let mut star_position_texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];

    let mut i =0;
    for (entity, transform,star) in &query {
        let p = transform.translation;
        in_verts.push(Vec3::new(p.x,-0.01,p.z));
        points.push(Point { x : p.x as f64, y : p.z as f64 });

        in_ids.push(i);
        if let Some(_star) = star {
            commands.entity(entity).insert(StarGfxTag{ id : i as usize });
        }
        
        star_position_texture_data[(i as usize *16)..(i as usize*16)+4].copy_from_slice(&p.x.to_le_bytes());
        star_position_texture_data[(i as usize *16)+4..(i as usize*16)+8].copy_from_slice(&p.z.to_le_bytes());

        let mut nearest = f32::MAX;
        
        for (_entity, transform,_star) in &query {
            let d = p.xz().distance(transform.translation.xz());
            if d > 0.5 {
                nearest = f32::min(nearest,d);
            }
        }

        // distance to nearest neighbour stored in A channel
        star_position_texture_data[(i as usize *16)+12..(i as usize*16)+16].copy_from_slice(&nearest.to_le_bytes());

        i+=1;
    }

    let colours_texture = Image::new_fill(
        Extent3d {
            width : TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers:1,
        },
        TextureDimension::D2,
        &colour_texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD
    );

    let star_position_texture = Image::new_fill(
        Extent3d {
            width : TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers:1,
        },
        TextureDimension::D2,
        &star_position_texture_data,
        TextureFormat::Rgba32Float,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD
    );

    let triangulation = triangulate(&points);

    let mut verts = Vec::<Vec3>::new();
    let mut bary = Vec::<Vec3>::new();
    let mut star_ids = Vec::<UVec3>::new();
    let mut star_distances = Vec::<Vec3>::new();

    let n = triangulation.triangles.len();
    let tris = triangulation.triangles;
    let mut t = 0;
    let mut indices = Vec::<u32>::new();
    while t < n {
        bary.push(Vec3::X);
        bary.push(Vec3::Y);
        bary.push(Vec3::Z);

        let c = UVec3::new(in_ids[tris[t]],in_ids[tris[t+1]],in_ids[tris[t+2]]);

        let pos_a = in_verts[in_ids[tris[t]] as usize];
        let pos_b = in_verts[in_ids[tris[t+1]] as usize];
        let pos_c = in_verts[in_ids[tris[t+2]] as usize];

        for i in 0..3 {
            let p = in_verts[tris[t+i]];

            star_ids.push(c);
            star_distances.push(Vec3::new(
                p.distance(pos_a),
                p.distance(pos_b),
                p.distance(pos_c)
            ));
            indices.push(t as u32 + i as u32);
            verts.push(p);
        }

        t += 3;
    }

    let mesh = meshes.add( Mesh::new(PrimitiveTopology::TriangleList,  RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        verts
    ).with_inserted_attribute(
        ATTRIBUTE_BARYCENTRIC,        
        bary
    )
    .with_inserted_attribute(
        ATTRIBUTE_STARID,
        star_ids
    )
    .with_inserted_attribute(
        ATTRIBUTE_STARDIST,
        star_distances
    )
    .with_inserted_indices(
        Indices::U32(indices)
    ));

    let tex = textures.add(colours_texture);
    let star_pos_tex = textures.add(star_position_texture);
    let material = materials.add(TerritoryOverlaysMaterial { color_texture: Some(tex.clone()), star_position_texture : Some(star_pos_tex), alpha_mode : AlphaMode::Blend });

    commands.insert_resource(OverlaysData{
        _mesh_handle : mesh.clone(),
        image_handle : tex,
        material_handle : material.clone()
    });

    commands.spawn(
        MaterialMeshBundle {
            mesh,
            material : material,
            ..default()
        }
    );
}