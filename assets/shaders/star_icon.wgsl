#import bevy_ui::ui_vertex_output UiVertexOutput

struct StarMaterial {
    color : vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material :StarMaterial;

// Really the shared elements should be pulled into a common file instead of duplicating across this and star_instancing
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
    let dpdx = dpdx(uv);
    let dpdy = dpdy(uv);

    let intensity = 1.0 / 256.0;
    var starcol = vec3(0.0);
    for(var i =0; i<8; i+=1) {
        starcol += draw_star(uv + dpdx * weights_8[i].x + dpdy * weights_8[i].y, material.color.rgb, intensity);
    }
    starcol = starcol / 8.0;

    let a = (starcol.x+starcol.y+starcol.z)/3.0;
    return vec4<f32>(mix(vec3<f32>(0.0),starcol.rgb,a),1.0);
}
