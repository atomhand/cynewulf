#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
#import bevy_pbr::view_transformations::position_world_to_clip;
#import bevy_pbr::mesh_view_bindings::view

struct StarInstancingUniforms {
    system_transition_factor: f32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: Vec3,
#endif
}
@group(2) @binding(0) var<uniform> settings : StarInstancingUniforms;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_pos_scale: vec4<f32>,
    @location(4) i_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    //let model = get_world_from_local(vertex.instance_index);

    let star_radius = vertex.i_pos_scale.w;
    let scale_factor =  mix(15.0,star_radius * 15.0, settings.system_transition_factor);

    let camera_right = normalize(vec3<f32>(view.clip_from_world.x.x, view.clip_from_world.y.x, view.clip_from_world.z.x));
    let camera_up = normalize(vec3<f32>(view.clip_from_world.x.y, view.clip_from_world.y.y, view.clip_from_world.z.y));

    let world_space = (camera_right * vertex.position.x + camera_up * vertex.position.y ) * scale_factor;
    let position = view.clip_from_world * vec4<f32>(world_space+ vertex.i_pos_scale.xyz, 1.0);

    var out: VertexOutput;
    out.clip_position = position;
    out.color = vertex.i_color;
    out.uv = vertex.position.xy;
    // out.uv = vertex.position.xy;// * scale_factor;
    // out.sphere_origin = (model * vec4<f32>(0.0,0.0,0.0,1.0)).xyz;
    // out.camera_origin = view.world_position;
    // out.ray_dir = (model * vec4<f32>(world_space, 1.0)).xyz - view.world_position ;

    return out;

    //let position = vertex.position * vertex.i_pos_scale.w + vertex.i_pos_scale.xyz;
    //var out: VertexOutput;

    // NOTE: Passing 0 as the instance_index to get_world_from_local() is a hack
    // for this example as the instance_index builtin would map to the wrong
    // index in the Mesh array. This index could be passed in via another
    // uniform instead but it's unnecessary for the example.

/*
    As it stands, this shader isn't using the world to local transform, just holding onto it for posterity

    let world_to_local = mat4x4<f32>(
        vec4<f32>(1.0,0.0,0.0,0.0),
        vec4<f32>(0.0,1.0,0.0,0.0),
        vec4<f32>(0.0,0.0,1.0,0.0),
        vec4<f32>(0.0,0.0,0.0,1.0));
    
    mesh_position_local_to_clip(
        world_to_local,
        vec4<f32>(position, 1.0)
    )
*/


    //out.clip_position = position_world_to_clip(position); 

    //out.color = vertex.i_color;
    //return out;
}

fn rnd(val : i32) -> f32{
    return 0.75;
}


fn draw_star(pos : vec2<f32>, star_color : vec3<f32>, I : f32) -> vec3<f32> {
    //let star_luminosity : f32 = 1e3;
    //let star_color : vec3<f32> = vec3<f32>(1.,.3,.1)*star_luminosity;

    let SCALE = 1.0 /8.0;// / 20.0;//star_radius * 0.01;
    var d : f32 = length(pos) * SCALE;

    var col = I * star_color;
    var spectrum = I * star_color;

    col = spectrum / (d*d*d);

    d = length(pos * vec2<f32>(50.0,0.5)) * SCALE;
    col += spectrum/ (d*d*d);
    d = length(pos * vec2<f32>(0.5,50.0)) * SCALE;
    col += spectrum / (d*d*d);

    return col * (1.0 - smoothstep(0.9,1.0,length(pos)));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let I = .02*exp(-15.*rnd(6*1+4));
    let starcol = draw_star(in.uv, in.color.rgb, I);
    let a = (starcol.x+starcol.y+starcol.z)/3.0;//max(starcol.x,max(starcol.y,starcol.z));

    return vec4<f32>(starcol,a);
}
