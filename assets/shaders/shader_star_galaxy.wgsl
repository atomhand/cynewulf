

// https://github.com/bevyengine/bevy/blob/c75d14586999dc1ef1ff6099adbc1f0abdb46edf/crates/bevy_render/src/view/view.wgsl
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_functions::get_world_from_local
#import bevy_pbr::prepass_io::Vertex

@group(2) @binding(0) var<uniform> material_color: vec3<f32>;
@group(2) @binding(1) var<uniform> star_radius: f32;
@group(2) @binding(2) var<uniform> system_transition_factor: f32;

// see https://github.com/kulkalkul/bevy_mod_billboard/blob/main/src/shader/billboard.wgsl

struct MyVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) sphere_origin: vec3<f32>,
    @location(2) camera_origin: vec3<f32>,
    @location(3) ray_dir: vec3<f32>,
}


@vertex
fn vertex(vertex: Vertex) -> MyVertexOutput {
    let model = get_world_from_local(vertex.instance_index);

    let scale_factor = mix(15.0,star_radius * 15.0, system_transition_factor);

    let camera_right = normalize(vec3<f32>(view.clip_from_world.x.x, view.clip_from_world.y.x, view.clip_from_world.z.x));
    let camera_up = normalize(vec3<f32>(view.clip_from_world.x.y, view.clip_from_world.y.y, view.clip_from_world.z.y));

    let world_space = (camera_right * vertex.position.x + camera_up * vertex.position.y ) * scale_factor;
    let position = view.clip_from_world * model * vec4<f32>(world_space, 1.0);

    var out: MyVertexOutput;
    out.position = position;
    out.uv = vertex.position.xy;// * scale_factor;
    out.sphere_origin = (model * vec4<f32>(0.0,0.0,0.0,1.0)).xyz;
    out.camera_origin = view.world_position;
    out.ray_dir = (model * vec4<f32>(world_space, 1.0)).xyz - view.world_position ;

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


fn draw_star(pos : vec2<f32>, I : f32) -> vec3<f32> {
    //let star_luminosity : f32 = 1e3;
    //let star_color : vec3<f32> = vec3<f32>(1.,.3,.1)*star_luminosity;
    let star_color = material_color.rgb;

    let SCALE = 1.0 /8.0;// / 20.0;//star_radius * 0.01;
    var d : f32 = length(pos) * SCALE;

    var col = I * star_color;
    var spectrum = I * star_color;

    col = spectrum / (d*d*d);

    d = length(pos * vec2<f32>(50.0,0.5)) * SCALE;
    //col += spectrum/ (d*d*d);
    d = length(pos * vec2<f32>(0.5,50.0)) * SCALE;
    col += spectrum / (d*d*d);

    return col * (1.0 - smoothstep(0.9,1.0,length(pos)));
}

fn rnd(val : i32) -> f32{
    return 0.75;
}

@fragment
fn fragment(
    mesh: MyVertexOutput,
) -> @location(0) vec4<f32> {

    let sphere_hit = sphIntersect(mesh.camera_origin, normalize(mesh.ray_dir), vec4<f32>(mesh.sphere_origin,star_radius*1.0));

    let I = .02*exp(-15.*rnd(6*1+4));
    let starcol = draw_star(mesh.uv , I);
    let a = max(starcol.x,max(starcol.y,starcol.z));


    return vec4<f32>(starcol,a);
    /*
    if sphere_hit > 0.0 {
        return material_color;
    } else {
        return vec4<f32>(sphere_hit, 0.0,0.0,1.0);
    }
    */
}