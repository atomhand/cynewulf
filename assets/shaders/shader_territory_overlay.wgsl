#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}


@group(2) @binding(1) var material_color_texture: texture_2d<f32>;
@group(2) @binding(2) var material_color_sampler: sampler;
@group(2) @binding(3) var star_position_texture: texture_2d<f32>;
@group(2) @binding(4) var star_position_sampler: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) star_id: vec3<u32>,
    @location(2) barycentric: vec3<f32>,
    @location(3) star_distance: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) barycentric: vec3<f32>,
    @location(2) star_id: vec3<u32>,
    @location(3) world_pos: vec3<f32>,
}

@vertex
fn vertex(@builtin(vertex_index) vertex_index : u32,
    vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = mesh_position_local_to_clip(get_world_from_local(vertex.instance_index), vec4<f32>(vertex.position, 1.0));
    out.star_id = vertex.star_id;
    out.barycentric = vertex.barycentric;
    out.world_pos = vertex.position;
    return out;
}

struct FragmentInput {
    @location(1) barycentric: vec3<f32>,
    @location(2) star_id: vec3<u32>,
    @location(3) world_pos: vec3<f32>,
};

@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
    let d = min(input.barycentric.x, min(input.barycentric.y, input.barycentric.z));
    let t = 0.05 * (0.85 + sin(5.0 * 1));

    // star colours
    let a = textureSample(material_color_texture,material_color_sampler, (vec2<f32>(f32(input.star_id.x % 128), f32(input.star_id.x / 128)) + vec2<f32>(0.5,0.5)) / 128.0);
    let b = textureSample(material_color_texture,material_color_sampler, (vec2<f32>(f32(input.star_id.y % 128), f32(input.star_id.y / 128)) + vec2<f32>(0.5,0.5)) / 128.0);
    let c = textureSample(material_color_texture,material_color_sampler, (vec2<f32>(f32(input.star_id.z % 128), f32(input.star_id.z / 128)) + vec2<f32>(0.5,0.5)) / 128.0);

    // star positions
    let a_pos = textureSample(star_position_texture,star_position_sampler, (vec2<f32>(f32(input.star_id.x % 128), f32(input.star_id.x / 128)) + vec2<f32>(0.5,0.5)) / 128.0);
    let b_pos = textureSample(star_position_texture,star_position_sampler, (vec2<f32>(f32(input.star_id.y % 128), f32(input.star_id.y / 128)) + vec2<f32>(0.5,0.5)) / 128.0);
    let c_pos = textureSample(star_position_texture,star_position_sampler, (vec2<f32>(f32(input.star_id.z % 128), f32(input.star_id.z / 128)) + vec2<f32>(0.5,0.5)) / 128.0);

    let dist = vec3(distance(input.world_pos.xz,a_pos.xy), distance(input.world_pos.xz,b_pos.xy), distance(input.world_pos.xz,c_pos.xy));

    // Distance to each star regularised to each star's distance to its nearest neighbour
    // ... the actual territory projection range is half this distance
    let regularised_dist = dist / vec3(a_pos.w,b_pos.w,c_pos.w);

    // calculate colours for the terrain overlay
    let edge_inner = smoothstep(vec3(0.44),vec3(0.46),regularised_dist);
    let edge_outer = smoothstep(vec3(0.5),vec3(0.52),1.0-regularised_dist);
    let edge = min(edge_inner,edge_outer);

    let inner_glow = smoothstep(vec3(0.0),vec3(1.0),regularised_dist);
    let edge_glow = smoothstep(vec3(0.4),vec3(0.47),regularised_dist) * 0.5;

    let c_weight = min(edge_outer,inner_glow + edge_glow);

    return a * c_weight.x + b * c_weight.y + c * c_weight.z;
}