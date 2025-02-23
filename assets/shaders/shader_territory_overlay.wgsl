#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}


struct StarFormat {
    pos : vec4<f32>,
    col : vec4<f32>,
    empire_halo : vec4<f32>,
    system_halo : vec4<f32>,
    pop_rank : f32,
}
struct LaneFormat {
    enabled : u32,
    col : vec3<f32>
}
@group(2) @binding(1) var<storage> star_data_array: array<StarFormat>;
@group(2) @binding(2) var<storage> lane_data_array: array<LaneFormat>;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) @interpolate(flat) star_id: vec3<u32>,
    @location(2) barycentric: vec3<f32>,
    @location(3) edge_id: vec3<u32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) barycentric: vec3<f32>,
    @location(2) star_id: vec3<u32>,
    @location(3) world_pos: vec3<f32>,
    @location(4) edge_id : vec3<u32>,
}

@vertex
fn vertex(@builtin(vertex_index) vertex_index : u32,
    vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = mesh_position_local_to_clip(get_world_from_local(vertex.instance_index), vec4<f32>(vertex.position, 1.0));
    out.star_id = vertex.star_id;
    out.barycentric = vertex.barycentric;
    out.world_pos = vertex.position;
    out.edge_id = vertex.edge_id;
    return out;
}

struct FragmentInput {
    @location(1) barycentric: vec3<f32>,
    @location(2) star_id: vec3<u32>,
    @location(3) world_pos: vec3<f32>,
    @location(4) edge_id : vec3<u32>,
};

// The MIT License
// Copyright © 2018 Inigo Quilez
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions: The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
fn cro(a : vec2<f32>, b : vec2<f32>) -> f32 {
    return a.x*b.y - a.y*b.x;
}
fn sd_uneven_capsule_y(in_p : vec2<f32>, ra : f32, rb : f32, h : f32) -> f32 {
    let p = vec2<f32>(abs(in_p.x),in_p.y);

    let b = (ra-rb)/h;
    let c = vec2<f32>(sqrt(1.0-b*b),b);
    let k = cro(c,p);
    let m = dot(c,p);
    let n = dot(p,p);

    if k < 0.0 {
        return sqrt(n) - ra;
    }
    else if k > c.x*h {
        return sqrt(n+h*h-2.0*h*p.y) - rb;
    }
    return m -ra;
 }
 fn sd_uneven_capsule(in_p : vec2<f32>, pa : vec2<f32>, in_pb : vec2<f32>, ra : f32, rb : f32) -> f32 {
    let p = in_p - pa;
    let pb = in_pb - pa;
    let h = dot(pb,pb);
    var q = vec2<f32>( dot(p,vec2<f32>(pb.y,-pb.x)),dot(p,pb) ) / h;

    q.x = abs(q.x);

    let b = ra-rb;
    let c = vec2<f32>(sqrt(h-b*b),b);

    let k = cro(c,q);
    let m = dot(c,q);
    let n = dot(q,q);

    if k < 0.0 {
        return sqrt(h*n)-ra;
    } else if k > c.x {
        return sqrt(h* (n+1.0-2.0*q.y)) - rb;
    }
    return m - ra;
}
fn sd_circle(p : vec2<f32>, center : vec2<f32>, r : f32) -> f32 {
    return distance(p,center) - r;
}
// END IQUILEZ

fn star_adjusted_distance_factor(a : vec4<f32>, b : vec4<f32>, c : vec4<f32>) -> f32 {
    let pv = a.xy + normalize(c.xy - a.xy) * a.w;
    return line_segment_distance(a.xy,b.xy, pv);
}

fn line_segment_distance(v : vec2<f32>, w : vec2<f32>, p : vec2<f32>) -> f32 {
    let l2 : f32 = dot(w-v,w-v);
    if l2 == 0.0 { return distance(p,v); }

    let t : f32 = saturate(dot((p - v), (w - v)) / l2);
    let projection : vec2<f32> = mix(v,w,t);

    return distance(p,projection);
}

fn smin(a : f32, b : f32, in_k : f32) -> f32 { 
    /*   
    let h : f32 = clamp( 0.5+0.5*(b-a)/k, 0.0, 1.0 );
    return mix( b, a, h ) - k*h*(1.0-h);
*/    
    let k = in_k * 1.0/(1.0-sqrt(0.5));
    let h = max( k-abs(a-b), 0.0)/k;
    return min(a,b) - k*0.5*(1.0+h-sqrt(1.0-h*(h-2.0)));
}

fn pick(rd : vec3<f32>, v : f32) -> vec3<f32> {
    var res = rd;
    if rd.x <= min(rd.y,rd.z) {
        res.y = v;
        res.z = v;
    } else if rd.y <= rd.z {
        res.x = v;
        res.z = v;
    } else {
        res.y = v;
        res.x = v;
    }
    return saturate(res);
}

fn halo_weight(dist : vec3<f32>, pixel_dist : f32,  offset : f32, thickness : f32) -> vec3<f32> {
    let a_inner = vec3(1.0) - smoothstep(vec3(0.0),vec3(pixel_dist),-dist+offset-thickness);
    let a_outer = vec3(1.0) - smoothstep(vec3(0.0),vec3(pixel_dist),dist-(offset));

    return saturate(min(a_inner,a_outer));
}

fn gradient_halo(dist : vec3<f32>, pixel_dist : f32,  offset : f32, thickness : f32) -> vec3<f32> {
    let a_inner = vec3(1.0) - smoothstep(vec3(0.0),vec3(pixel_dist),-dist+offset);
    let a_outer = vec3(1.0) - smoothstep(vec3(0.0),vec3(thickness),dist-(offset));

    return saturate(min(a_inner,a_outer));
}

fn banded_rank_halo(dist : vec3<f32>, pixel_dist : f32,  offset : f32, thickness : f32, near_rank : i32) -> vec3<f32> {
    let d : vec3<f32> = -(dist);
    let range_min : f32 = abs(offset);
    
    let cutoff_min : f32 = range_min + thickness * (12.0 - f32(near_rank));
    let range_max : f32 = abs(offset) + thickness * 1.0 * 12.0;


    let c_rank = (d-range_min) / vec3(thickness);
    let f_dist : vec3<f32> = fract(c_rank);    

    let p : f32 = 0.2;//pixel_dist/thickness;

    let intensity = 1.0 - (d-range_min) / (range_max-range_min);

    // fade out the 
    let transition_factor = saturate(0.4 * pixel_dist / p);

    let f_inner = smoothstep(vec3(0.0),vec3(p), f_dist);
    let f_outer = smoothstep(vec3(0.0),vec3(p), 1.0-f_dist);
    let ff : vec3<f32> = mix(min(f_inner,f_outer),vec3(1.0),vec3(transition_factor));// * (f32(near_rank) - c_rank) / 12.0;

    let r_inner = smoothstep(vec3(0.0),vec3(pixel_dist), d-vec3(cutoff_min));
    let r_outer = smoothstep(vec3(0.0),vec3(pixel_dist), vec3(range_max)-d);
    let rr: vec3<f32>= min(r_inner,r_outer);

    return saturate(min(ff,rr)) * intensity;

/*
    var res = vec3(0.0);
    for(var i: i32=0; i<12; i++) {
        if i >= near_rank {
            break;
        }
        let l_offset = thickness + offset - (thickness) * 1.5 * f32(i);
        let a_inner = vec3(1.0) - smoothstep(vec3(0.0),vec3(pixel_dist),-dist+l_offset);
        let a_outer = vec3(1.0) - smoothstep(vec3(0.0),vec3(pixel_dist),dist-(l_offset+thickness-pixel_dist));

        res = max(res,min(a_inner,a_outer));
    }

    return res;*/
}

@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
    let d = min(input.barycentric.x, min(input.barycentric.y, input.barycentric.z));
    let t = 0.05 * (0.85 + sin(5.0 * 1));

    let p = input.world_pos.xz;

    let a = star_data_array[input.star_id.x];
    let b = star_data_array[input.star_id.y];
    let c = star_data_array[input.star_id.z];

    // NOTE FOR POTENTIAL OPTIMISATION
    // Process
    // 1. Detect the nearest star "X" out of the 3 (can get it directly from the barycentrics)
    // 2. Sample the circle that includes X
    // 3. Sample + combine the capsules XY and XZ
    // (4. Below code doesn't do it right now, but it's also potentially necessary to subtract the circles Y and Z IF they don't share a colour with X.
    //      This is to stop hyperlanes overlapping with star circles)
    // This should give the same result as the below code for ~1/3 the operations
    // It should also be neater and simpler since we work with 1 distance value instead of 3

    // NOTE:
    // Tried it, there were more visual artefacts than I thought lol

    /*
    var m = 0;
    if input.barycentric.x > max(input.barycentric.y,input.barycentric.z) {
        m = 0;
    } else if input.barycentric.y > input.barycentric.z {
        m = 1;
    } else {
        m = 2;
    }
    let n = (m+1) % 3;
    let o = (m+2) % 3;

    var distance = sd_circle(p, pos[m].xy, pos[m].w / 2.0);

    if all(col[m] == col[n]) {
        let m_f = star_adjusted_distance_factor(pos[m],pos[n],pos[o]) / 2.0;
        let n_f = star_adjusted_distance_factor(pos[n],pos[m],pos[o]) / 2.0;
        let f = min(m_f,n_f);
        distance = min(distance, sd_uneven_capsule(p, pos[m].xy,pos[n].xy, f, f));
    }
    if all(col[m] == col[o]) {
        let m_f = star_adjusted_distance_factor(pos[m],pos[o],pos[n]) / 2.0;
        let o_f = star_adjusted_distance_factor(pos[o],pos[m],pos[n]) / 2.0;
        let f = min(m_f,o_f);
        distance = min(distance, sd_uneven_capsule(p, pos[m].xy,pos[o].xy, f, f));
    }
    
    let edge_inner = 1.0 - smoothstep(0.0,16.0, -distance);
    let edge_outer = 1.0 - smoothstep(0.0,0.1, distance);

    let c_weight = saturate(min(edge_inner,edge_outer));

    return c_weight * col[m];
    */

    let circle_distance = vec3<f32>(
        sd_circle(p, a.pos.xy, a.pos.w / 2.0),
        sd_circle(p, b.pos.xy, b.pos.w / 2.0),
        sd_circle(p, c.pos.xy, c.pos.w / 2.0)
    );

    var distance = circle_distance + vec3(1.0);
    var halo_distance = distance;

    var hyperlane_dist = 10000.f;

    var lane = LaneFormat(0,vec3(0.0));    

    let hyperlane_w = 0.1;
    let hyperlane_offset = 12.0;
    if input.edge_id.x < 99999 {
        let dir = normalize(b.pos.xy - a.pos.xy) * hyperlane_offset;
        let d = sd_uneven_capsule(
                p,
                a.pos.xy + dir,
                b.pos.xy - dir,
                hyperlane_w,
                hyperlane_w
            );
        if d < hyperlane_dist {
            hyperlane_dist = d;
            lane = lane_data_array[input.edge_id.x];
        }
    }
    if input.edge_id.y < 99999 {
        let dir = normalize(c.pos.xy - b.pos.xy) * hyperlane_offset;
        let d = sd_uneven_capsule(
                p,
                b.pos.xy + dir,
                c.pos.xy - dir,
                hyperlane_w,
                hyperlane_w
            );
        if d < hyperlane_dist {
            hyperlane_dist = d;
            lane = lane_data_array[input.edge_id.y];
        }
    }
    if input.edge_id.z < 99999 {
        let dir = normalize(c.pos.xy - a.pos.xy) * hyperlane_offset;
        let d = sd_uneven_capsule(
                p,
                a.pos.xy + dir,
                c.pos.xy - dir,
                hyperlane_w,
                hyperlane_w
            );
        if d < hyperlane_dist {
            hyperlane_dist = d;
            lane = lane_data_array[input.edge_id.z];
        }
    }

    let contraction_fac : f32 = 3.0;
    let smin_fac : f32 = 1.0;
    
    if b.col.a != 0.0 && all(b.col==c.col)
    {
        distance.y = min(distance.z,distance.y);
        distance.z = 1000.0;
        if input.edge_id.y < 99999 {
            let bf = star_adjusted_distance_factor(b.pos,c.pos,a.pos) / 2.0;
            let cf = star_adjusted_distance_factor(c.pos,b.pos,a.pos) / 2.0;
            let f = min(bf,cf) - contraction_fac;

            // fade out when point is at edge AC or AB
            // ie. when bary.y or bary.z == 0.0

            let fadeout = smoothstep(0.9,1.0, 1.0 - min(input.barycentric.y,input.barycentric.z));
            let d = sd_uneven_capsule(p, b.pos.xy, c.pos.xy, f, f);
            let c = min(circle_distance.z,circle_distance.y);
            distance.y = smin(distance.y, d, smin_fac);
        }

        if all(b.system_halo == c.system_halo) {
            halo_distance.y = distance.y;
            halo_distance.z = distance.z;
        }
    }
    if a.col.a != 0.0 && all(a.col==b.col)
    {
        distance.x = min(distance.x,distance.y);
        distance.y = 1000.0;
        if input.edge_id.x < 99999 {
            let af = star_adjusted_distance_factor(a.pos,b.pos,c.pos) / 2.0;
            let bf = star_adjusted_distance_factor(b.pos,a.pos,c.pos) / 2.0;
            let f = min(af,bf) - contraction_fac;

            // fade out when point is at edge BC or AC
            // ie. when bary.x or bary.y == 0.0

            let fadeout = smoothstep(0.9,1.0, 1.0 - min(input.barycentric.y,input.barycentric.x));
            let d = sd_uneven_capsule(p, a.pos.xy, b.pos.xy, f, f);
            let c = min(circle_distance.x,circle_distance.y);
            distance.x = smin(distance.x, d, smin_fac);
        }

        if all(b.system_halo == a.system_halo) {
            halo_distance.x = distance.x;
            halo_distance.y = distance.y;
        }
    }
    if a.col.a != 0.0 && all(a.col==c.col)
    {
        distance.x = min(distance.x,distance.z);
        distance.z = 1000.0;
        if input.edge_id.z < 99999 {
            let af = star_adjusted_distance_factor(a.pos,c.pos,b.pos) / 2.0;
            let cf = star_adjusted_distance_factor(c.pos,a.pos,b.pos) / 2.0;
            let f = min(af,cf) - contraction_fac;

            // fade out when point is at edge AB or BC
            // ie. when bary.z or bary.x == 0.0
            let fadeout = smoothstep(0.9,1.0, 1.0 - min(input.barycentric.z,input.barycentric.x));
            let d = sd_uneven_capsule(p, a.pos.xy, c.pos.xy, f, f);
            let c = min(circle_distance.x,circle_distance.z);
            distance.x = smin(distance.x, d, smin_fac);
        }

        if all(a.system_halo == c.system_halo) {
            halo_distance.x = distance.x;
            halo_distance.z = distance.z;
        }
    }

    // adjust this parameter to dilate or contract the border shape
    distance += vec3(4.0);

    halo_distance += vec3(4.0);
    // Get a cheap antialiasing by softly fading out any edges over a short distance scaled with the pixel derivatives
    let antialias_dist = length(fwidth(input.world_pos.xz));

    let territory_falloff_band = 3.0;
    let edge_inner = vec3(1.0) - 0.9 * smoothstep(vec3(0.0),vec3(territory_falloff_band), -distance);
    let edge_outer = vec3(1.0) - smoothstep(vec3(0.0), vec3(antialias_dist), distance);

    let c_weight = saturate(min(edge_inner,edge_outer));

    let territory_col = a.col * c_weight.x + b.col * c_weight.y + c.col * c_weight.z;

    // selection halo
    let system_halo_weight = halo_weight(halo_distance, antialias_dist, 1.0, 1.0) + halo_weight(halo_distance, antialias_dist, 2.5 + antialias_dist, 0.5);
    let system_halo = a.system_halo * system_halo_weight.x + b.system_halo * system_halo_weight.y + c.system_halo * system_halo_weight.z;

    let empire_halo_weight = gradient_halo(distance, antialias_dist, -antialias_dist, 4.0);// + halo_weight(distance, antialias_dist, 2.5 + antialias_dist, 0.5);
    let empire_halo = a.empire_halo * empire_halo_weight.x + b.empire_halo * empire_halo_weight.y + c.empire_halo * empire_halo_weight.z;

    // rank halo
    var near_rank = 0;
    if circle_distance.x < min(circle_distance.y,circle_distance.z) {
        near_rank = i32(a.pop_rank);
    } else if circle_distance.y < circle_distance.z {
        near_rank = i32(b.pop_rank);
    } else {
        near_rank = i32(c.pop_rank);
    }
    let rank_halo_weight = banded_rank_halo(circle_distance, antialias_dist, -7.0, 0.8, near_rank);
    let rank_halo = a.col * rank_halo_weight.x + b.col * rank_halo_weight.y + c.col* rank_halo_weight.z;

    let hyperlane_col = vec4(lane.col,1.0);
    let hyperlane_alpha = 1.0 - smoothstep(0.0,antialias_dist,hyperlane_dist);

    let sel_col = mix(max(empire_halo,rank_halo),system_halo,system_halo.a);

    return mix(mix(territory_col,sel_col,sel_col.a),hyperlane_col,hyperlane_alpha);
}