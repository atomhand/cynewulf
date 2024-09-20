

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
    @location(3) uv: vec2<f32>,
}


@vertex
fn vertex(vertex: Vertex) -> MyVertexOutput {
    let model = get_world_from_local(vertex.instance_index);

    let camera_right = normalize(vec3<f32>(view.clip_from_world.x.x, view.clip_from_world.y.x, view.clip_from_world.z.x));
    let camera_up = normalize(vec3<f32>(view.clip_from_world.x.y, view.clip_from_world.y.y, view.clip_from_world.z.y));

    let world_space = (camera_right * vertex.position.x + camera_up * vertex.position.y)  * (planet_radius+0.0093*0.4);
    let position = view.clip_from_world * model * vec4<f32>(world_space, 1.0);

    var out: MyVertexOutput;
    out.position = position;
    out.sphere_origin = (model * vec4<f32>(0.0,0.0,0.0,1.0)).xyz;
    out.camera_origin = view.world_position;
    out.ray_dir = (model * vec4<f32>(world_space, 1.0)).xyz - view.world_position;
    out.uv = vertex.position.xy;

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

fn subsample(ro : vec3<f32>, rd_in : vec3<f32>, sph : vec4<f32>, uv : vec2<f32>, rd_dx : vec3<f32>, rd_dy : vec3<f32>, weights : vec2<f32>) -> vec4<f32> {
    let rd = normalize(rd_in + rd_dx * weights.x  + rd_dy * weights.y);
    let hit = sphIntersect(ro,rd,sph);

    let v_dx = dpdx(uv);
    let v_dy = dpdy(uv);
    let plane_dist = length(uv + v_dx * weights.x  + v_dy * weights.y);

    if hit > 0.0 {
        let hit_pos = ro + rd * hit;
        let normal = normalize(hit_pos - sph.xyz);
        let attenuation = dot(normal,star_pos-sph.xyz);

        return vec4<f32>(attenuation * surface_color,1.0);
    } else {
        //let halo_strength = smoothstep(0.05,0.3,1.0 - plane_dist);
        let halo_strength = step(0.0,0.9 - plane_dist);
        return vec4<f32>(halo_color*halo_strength,halo_strength);
    }
}
const weights = array<vec2<f32>,4>(
    vec2<f32>(1.0/8.0,3.0/8.0),
    vec2<f32>(3.0/8.0,-1.0/8.0),
    vec2<f32>(-1.0/8.0,-3.0/8.0),
    vec2<f32>(-3.0/8.0,1.0/8.0)
);

const weights_8 = array<vec2<f32>,8>(
    vec2<f32>(1.0/8.0,-3.0/8.0),
    vec2<f32>(-1.0/8.0,3.0/8.0),
    vec2<f32>(5.0/8.0,1.0/8.0),
    vec2<f32>(-3.0/8.0,-5.0/8.0),
    vec2<f32>(-5.0/8.0,5.0/8.0),
    vec2<f32>(-7.0/8.0,-1.0/8.0),
    vec2<f32>(3.0/8.0,7.0/8.0),
    vec2<f32>(7.0/8.0,-7.0/8.0)
);

@fragment
fn fragment(
    mesh: MyVertexOutput,
) -> @location(0) vec4<f32> {
    let dx : vec3<f32> = dpdx(mesh.ray_dir);
    let dy : vec3<f32> = dpdy(mesh.ray_dir);

    let plane_dist = length(mesh.uv);

    let sph = vec4<f32>(mesh.sphere_origin,planet_radius);
    var col = vec4<f32>(0.0);

    col += subsample(mesh.camera_origin, mesh.ray_dir, sph, mesh.uv, dx, dy, weights_8[0]);
    col += subsample(mesh.camera_origin, mesh.ray_dir, sph, mesh.uv, dx, dy, weights_8[1]);
    col += subsample(mesh.camera_origin, mesh.ray_dir, sph, mesh.uv, dx, dy, weights_8[2]);
    col += subsample(mesh.camera_origin, mesh.ray_dir, sph, mesh.uv, dx, dy, weights_8[3]);
    col += subsample(mesh.camera_origin, mesh.ray_dir, sph, mesh.uv, dx, dy, weights_8[4]);
    col += subsample(mesh.camera_origin, mesh.ray_dir, sph, mesh.uv, dx, dy, weights_8[5]);
    col += subsample(mesh.camera_origin, mesh.ray_dir, sph, mesh.uv, dx, dy, weights_8[6]);
    col += subsample(mesh.camera_origin, mesh.ray_dir, sph, mesh.uv, dx, dy, weights_8[7]);

    let planet = col / 8.0;

    return planet;
}