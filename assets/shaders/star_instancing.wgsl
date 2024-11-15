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

    let star_radius = 0.0093;//vertex.i_pos_scale.w;
    let scale_factor =  mix(7.0,star_radius * 30.0, settings.system_transition_factor);

    let camera_right = normalize(vec3<f32>(view.clip_from_world.x.x, view.clip_from_world.y.x, view.clip_from_world.z.x));
    let camera_up = normalize(vec3<f32>(view.clip_from_world.x.y, view.clip_from_world.y.y, view.clip_from_world.z.y));

    let world_space = (camera_right * vertex.position.x + camera_up * vertex.position.y ) * scale_factor;
    let position = view.clip_from_world * vec4<f32>(world_space+ vertex.i_pos_scale.xyz, 1.0);

    var out: VertexOutput;
    out.clip_position = position;
    out.color = vertex.i_color;
    out.uv = vertex.position.xy;

    return out;
}

fn rnd(n : i32) -> f32{
    return fract(sin(f32(n)*543.21)*43758.5453);
}


fn draw_star(pos : vec2<f32>, star_color : vec3<f32>, I : f32) -> vec3<f32> {
    let a = (star_color.r + star_color.g + star_color.b) / 3.0;

    let c = star_color * (0.5 + 0.5 / a);//star_color * (24.0 / a);

    var SCALE = 1.0;// /16.0;
    var d : f32 = length(pos) * SCALE * 1.3;

    var col = I * c;
    var spectrum = I * c;

    col = spectrum / (d*d*d);

    let ARMS_SCALE = SCALE / 1.4;

    d = length(pos * vec2<f32>(50.0,0.5)) * ARMS_SCALE;
    col += spectrum/ (d*d*d) * (1.0 - settings.system_transition_factor);
    d = length(pos * vec2<f32>(0.5,50.0)) * ARMS_SCALE;
    col += spectrum / (d*d*d) * (1.0 - settings.system_transition_factor);

    return col ;//* (1.0 - smoothstep(0.9,1.0,length(pos)));
}

const weights_4 = array<vec2<f32>,4>(
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
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let dpdx = dpdx(in.uv);//vec2(dpdx(in.uv),dpdy(in.uv));
    let dpdy = dpdy(in.uv);

    let intensity = 1.0 / 256.0;//.02*exp(-15.*rnd(1));
    var starcol  = draw_star(in.uv + dpdx * weights_8[0].x + dpdy * weights_8[0].y, in.color.rgb, intensity);
    starcol     += draw_star(in.uv + dpdx * weights_8[1].x + dpdy * weights_8[1].y, in.color.rgb, intensity);
    starcol     += draw_star(in.uv + dpdx * weights_8[2].x + dpdy * weights_8[2].y, in.color.rgb, intensity);
    starcol     += draw_star(in.uv + dpdx * weights_8[3].x + dpdy * weights_8[3].y, in.color.rgb, intensity);
    starcol     += draw_star(in.uv + dpdx * weights_8[4].x + dpdy * weights_8[4].y, in.color.rgb, intensity);
    starcol     += draw_star(in.uv + dpdx * weights_8[5].x + dpdy * weights_8[5].y, in.color.rgb, intensity);
    starcol     += draw_star(in.uv + dpdx * weights_8[6].x + dpdy * weights_8[6].y, in.color.rgb, intensity);
    starcol     += draw_star(in.uv + dpdx * weights_8[7].x + dpdy * weights_8[7].y, in.color.rgb, intensity);
    starcol = starcol / 8.0;

    let a = (starcol.x+starcol.y+starcol.z)/3.0;

    return vec4<f32>(starcol,a);
}
