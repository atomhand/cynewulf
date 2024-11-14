#import bevy_ui::ui_vertex_output UiVertexOutput

struct StarMaterial {
    color : vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material :StarMaterial;


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
    col += spectrum/ (d*d*d);
    d = length(pos * vec2<f32>(0.5,50.0)) * ARMS_SCALE;
    col += spectrum / (d*d*d);

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
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv - vec2<f32>(0.5,0.5);
    let dpdx = dpdx(uv);//vec2(dpdx(in.uv),dpdy(in.uv));
    let dpdy = dpdy(uv);

    let intensity = 1.0 / 256.0;//.02*exp(-15.*rnd(1));
    var starcol  = draw_star(uv + dpdx * weights_8[0].x + dpdy * weights_8[0].y, material.color.rgb, intensity);
    starcol     += draw_star(uv + dpdx * weights_8[1].x + dpdy * weights_8[1].y, material.color.rgb, intensity);
    starcol     += draw_star(uv + dpdx * weights_8[2].x + dpdy * weights_8[2].y, material.color.rgb, intensity);
    starcol     += draw_star(uv + dpdx * weights_8[3].x + dpdy * weights_8[3].y, material.color.rgb, intensity);
    starcol     += draw_star(uv + dpdx * weights_8[4].x + dpdy * weights_8[4].y, material.color.rgb, intensity);
    starcol     += draw_star(uv + dpdx * weights_8[5].x + dpdy * weights_8[5].y, material.color.rgb, intensity);
    starcol     += draw_star(uv + dpdx * weights_8[6].x + dpdy * weights_8[6].y, material.color.rgb, intensity);
    starcol     += draw_star(uv + dpdx * weights_8[7].x + dpdy * weights_8[7].y, material.color.rgb, intensity);
    starcol = starcol / 8.0;

    let a = (starcol.x+starcol.y+starcol.z)/3.0;

    //return mix(vec4<f32>(0.0,0.0,0.0,1.0),vec4<f32>(starcol,a),a);
    return vec4<f32>(mix(vec3<f32>(0.0),starcol.rgb,a),1.0);
}
