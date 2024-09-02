
use delaunator::{Point,triangulate};

use bevy::prelude::*;
use crate::galaxy::{Star,OverlaysTriangulationVertex};
use crate::prelude::*;

use bevy::{
    reflect::TypePath,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    render::{
        mesh::{MeshVertexAttribute, Indices, MeshVertexBufferLayoutRef},
        render_asset::RenderAssetUsages,
        render_resource::*,
    },
};
// https://docs.rs/bevy/latest/bevy/pbr/prelude/trait.Material.html

#[derive(ShaderType,Default,Debug,Clone)]
#[repr(C)]
struct StarFormat {
    pos : Vec4,
    color : Vec4,
}

#[derive(Asset,TypePath,AsBindGroup,Debug,Clone)]
struct TerritoryOverlaysMaterial {
    #[storage(1, read_only)]
    star_data_buffer : Vec<StarFormat>,
    alpha_mode : AlphaMode,
}

const ATTRIBUTE_BARYCENTRIC: MeshVertexAttribute =
    MeshVertexAttribute::new("Barycentric", 2137464976, VertexFormat::Float32x3);

const ATTRIBUTE_STARID: MeshVertexAttribute =
    MeshVertexAttribute::new("StarID", 988540917, VertexFormat::Uint32x3);

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
    material_handle : Handle<TerritoryOverlaysMaterial>
}

pub struct OverlaysPlugin;

impl Plugin for OverlaysPlugin {
    fn build(&self, app : &mut App) {
        app.add_plugins(MaterialPlugin::<TerritoryOverlaysMaterial>::default())
            .add_systems(Startup, generate_overlays_mesh.after(crate::generators::galaxy_generation::setup_stars))
            .add_systems(Update,update_overlays);
    }
}

fn update_overlays(
    query : Query<(&StarGfxTag,&StarClaim),With<Star>>,
    empire_query : Query<&Empire>,
    mut mats : ResMut<Assets<TerritoryOverlaysMaterial>>,
    overlays_data : Res<OverlaysData>,
    selection : Res<crate::galaxy::Selection>,
    time : Res<Time>
) {
    let Some(mat) = mats.get_mut(&overlays_data.material_handle) else { return; };

    for (tag,claim) in &query {
        let col = if let Some(owner) = claim.owner {
            let c = empire_query.get(owner).unwrap().color.to_srgba();
            let t = (time.elapsed_seconds_wrapped() % 2.0) / 2.0;
            let anim = f32::sin(t * 2.0 * std::f32::consts::PI) * 0.5 + 0.5;
            if claim.owner == selection.hovered {               
                c.mix(&Srgba::new(1.0 - c.red,1.0 - c.green,1.0-c.blue,1.0),anim)
            } else if claim.owner == selection.selected {                
                c.mix(&Color::WHITE.to_srgba(),anim)
            } else {                
                c
            }
        } else {
            Srgba::new(0.0,0.0,0.0,0.0)
        };

        mat.star_data_buffer[tag.id as usize].color = col.to_vec4();
    }
} 

fn generate_overlays_mesh(
    query : Query<(Entity,&Transform,Option<&Star>),With<OverlaysTriangulationVertex>>,
    mut commands : Commands,
    mut meshes : ResMut<Assets<Mesh>>,
    mut materials : ResMut<Assets<TerritoryOverlaysMaterial>>,
) {
    let mut points = Vec::<Point>::new();
    let mut in_verts = Vec::<Vec3>::new();
    let mut in_ids = Vec::<u32>::new();

    let mut star_data =  Vec::with_capacity(1024);//

    let mut i =0;
    for (entity, transform,star) in &query {
        let p = transform.translation;
        in_verts.push(Vec3::new(p.x,-0.01,p.z));
        points.push(Point { x : p.x as f64, y : p.z as f64 });

        in_ids.push(i);
        if let Some(_star) = star {
            commands.entity(entity).insert(StarGfxTag{ id : i as usize });
        }

        let mut nearest = f32::MAX;
        
        for (_entity, transform,_star) in &query {
            let d = p.xz().distance(transform.translation.xz());
            if d > 0.5 {
                nearest = f32::min(nearest,d);
            }
        }

        // distance to nearest neighbour stored in A channel

        star_data.push(StarFormat {
            pos : Vec4::new(p.x,p.z, 0.0, nearest),
            .. default()
        });

        i+=1;
    }

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
    .with_inserted_indices(
        Indices::U32(indices)
    ));

    let material = materials.add(TerritoryOverlaysMaterial {
        star_data_buffer : star_data,
        alpha_mode : AlphaMode::Blend }
    );

    commands.insert_resource(OverlaysData{
        _mesh_handle : mesh.clone(),
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