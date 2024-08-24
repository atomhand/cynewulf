

// https://github.com/bevyengine/bevy/blob/c75d14586999dc1ef1ff6099adbc1f0abdb46edf/crates/bevy_render/src/view/view.wgsl
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_functions::get_world_from_local
#import bevy_pbr::prepass_io::Vertex

@group(2) @binding(0) var<uniform> surface_color: vec3<f32>;
@group(2) @binding(1) var<uniform> halo_color: vec3<f32>;
@group(2) @binding(2) var<uniform> planet_radius: f32;
@group(2) @binding(3) var<uniform> star_pos: vec3<f32>;

// see https://github.com/kulkalkul/bevy_mod_billboard/blob/main/src/shader/billboard.wgsl

struct MyVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) sphere_origin: vec3<f32>,
    @location(1) camera_origin: vec3<f32>,
    @location(2) ray_dir: vec3<f32>,
    @location(3) vert_xy: vec2<f32>,
}


@vertex
fn vertex(vertex: Vertex) -> MyVertexOutput {
    let model = get_world_from_local(vertex.instance_index);

    let camera_right = normalize(vec3<f32>(view.clip_from_world.x.x, view.clip_from_world.y.x, view.clip_from_world.z.x));
    let camera_up = normalize(vec3<f32>(view.clip_from_world.x.y, view.clip_from_world.y.y, view.clip_from_world.z.y));

    let billboard_scale = 1.0;

    let world_space = (camera_right * vertex.position.x + camera_up * vertex.position.y)  * (planet_radius+0.0093*0.4) * billboard_scale; // billboard is rescaled to match the needed radius only
    let position = view.clip_from_world * model * vec4<f32>(world_space, 1.0);

    var out: MyVertexOutput;
    out.position = position;
    out.sphere_origin = (model * vec4<f32>(0.0,0.0,0.0,1.0)).xyz;
    out.camera_origin = view.world_position;
    out.ray_dir = (model * vec4<f32>(world_space, 1.0)).xyz - view.world_position;
    out.vert_xy = vertex.position.xy;

    return out;
}

fn sphIntersect( ro : vec3<f32> , rd : vec3<f32> ,  sph : vec4<f32> ) -> f32
{
    let oc : vec3<f32> = ro - sph.xyz;
    let b : f32 = dot( oc, rd );
    let c : f32 = dot( oc, oc ) - sph.w*sph.w;
    var h : f32 = b*b - c;
    if( h<0.0 ) { return -1.0; }
    h = sqrt( h );

    return -b - h;
}

fn subsample(ro : vec3<f32>, rd_in : vec3<f32>, sph : vec4<f32>, vert_xy : vec2<f32>, weights : vec2<f32>) -> vec4<f32> {
    let rd_dx = dpdx(rd_in);
    let rd_dy = dpdy(rd_in);
    let rd = normalize(rd_in + weights.x * rd_dx + weights.y * rd_dy);
    let hit = sphIntersect(ro,rd,sph);

    let v_dx = dpdx(vert_xy);
    let v_dy = dpdy(vert_xy);
    let plane_dist = length(vert_xy + weights.x * v_dx + weights.y * v_dy);

    if hit > 0.0 {
        let hit_pos = ro + rd * hit;
        let normal = normalize(hit_pos - sph.xyz);
        let attenuation = dot(normal,star_pos-sph.xyz);

        return vec4<f32>(attenuation * surface_color,1.0);
    } else {
        let halo_strength = smoothstep(0.05,0.3,1.0 - plane_dist);//min(smoothstep(0.05,0.1,1.0 - plane_dist), smoothstep(0.75,0.9,plane_dist));
        return vec4<f32>(halo_color*halo_strength,halo_strength);
    }
}
const weights = array<vec2<f32>,4>(
    vec2<f32>(1.0/8.0,3.0/8.0),
    vec2<f32>(3.0/8.0,-1.0/8.0),
    vec2<f32>(-1.0/8.0,-3.0/8.0),
    vec2<f32>(-3.0/8.0,1.0/8.0)
);

@fragment
fn fragment(
    mesh: MyVertexOutput,
) -> @location(0) vec4<f32> {
    let dx = dpdx(mesh.ray_dir);
    let dy = dpdy(mesh.ray_dir);

    let plane_dist = length(mesh.vert_xy);

    let sph = vec4<f32>(mesh.sphere_origin,planet_radius);
    var col = vec4<f32>(0.0);

    col += subsample(mesh.camera_origin, mesh.ray_dir, sph, mesh.vert_xy, weights[0]);
    col += subsample(mesh.camera_origin, mesh.ray_dir, sph, mesh.vert_xy, weights[1]);
    col += subsample(mesh.camera_origin, mesh.ray_dir, sph, mesh.vert_xy, weights[2]);
    col += subsample(mesh.camera_origin, mesh.ray_dir, sph, mesh.vert_xy, weights[3]);
    /*
    col += subsample(mesh.camera_origin, mesh.ray_dir + weights[0].x * dx + weights[0].y *  dy, sph);
    col += subsample(mesh.camera_origin, mesh.ray_dir + weights[1].x * dx + weights[1].y *  dy, sph);
    col += subsample(mesh.camera_origin, mesh.ray_dir + weights[2].x * dx + weights[2].y *  dy, sph);
    col += subsample(mesh.camera_origin, mesh.ray_dir + weights[3].x * dx + weights[3].y *  dy, sph);
    */

    let planet = col / 4.0;

    return planet;// * planet.a + halo * (1.0 - planet.a);
}