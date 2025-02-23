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
    let a_inner = vec3(1.0) - smoothstep(vec3(0.0),vec3(pixel_dist*2.0),-dist+offset);
    let a_outer = vec3(1.0) - smoothstep(vec3(0.0),vec3(thickness),dist-(offset));

    return saturate(min(a_inner,a_outer));
}

// from iquilez, modified to remove the ring part...
fn sdRing( _p : vec2<f32>, n : vec2<f32>, r : f32, th : f32) -> f32
{
    var p = _p;
    p.x = abs(p.x);
   
    p = mat2x2(n.x,n.y,-n.y,n.x)*p;

    return length(vec2(p.x,max(0.0,abs(r-p.y)-th*0.5)))*sign(p.x);
}

fn banded_rank_halo(pos : vec2<f32>, pixel_dist : f32,  offset : f32, thickness : f32, near_rank : f32) -> f32 {
    let d : f32 = length(pos);

    let p : f32 = 0.2;//pixel_dist/thickness;
    let transition_factor = saturate(0.4 * pixel_dist / p);

    let range_min : f32 = offset;
    
    let cutoff_max : f32 = offset + thickness * floor(near_rank);
    let range_max : f32 = offset + thickness * 12.0;


    let c_rank = (d-range_min) / thickness;
    let f_dist : f32 = fract(c_rank);    


    let intensity = (d-range_min) / (range_max-range_min);

    // fade out the 

    let f_inner = smoothstep(0.0,p, f_dist);
    let f_outer = smoothstep(0.0,p, 1.0-f_dist);
    let ff : f32 = mix(min(f_inner,f_outer),1.0,transition_factor);// * (f32(near_rank) - c_rank) / 12.0;

    let r_inner = smoothstep(0.0,pixel_dist, d-range_min);
    let r_outer = smoothstep(0.0,pixel_dist, cutoff_max-d);
    let rr: f32= min(r_inner,r_outer);


    let radius = cutoff_max + thickness * 0.5;
    // angle for the outer ring
    let fr = 3.14159*fract(near_rank);
    let cs = vec2(cos(fr),sin(fr));
    let ring_d = sdRing(pos, cs, radius, thickness);
    let ring_w = smoothstep(0.0,p,-ring_d) ;//* (1.0-saturate(transition_factor*2.0-1.0));
    let ring_outer = smoothstep(0.0,pixel_dist, cutoff_max+thickness-d);
    
    let ring = min(min(ring_outer,ff),ring_w);

    return max(ring,saturate(min(ff,rr)) )* intensity;

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
    //let system_halo_weight = halo_weight(halo_distance, antialias_dist, 1.0, 0.6) + halo_weight(halo_distance, antialias_dist, 2.5 + antialias_dist, 0.6);
    let system_halo_weight = gradient_halo(halo_distance, antialias_dist, antialias_dist+0.5, 4.0);
    let system_halo = a.system_halo * system_halo_weight.x + b.system_halo * system_halo_weight.y + c.system_halo * system_halo_weight.z;

    let empire_halo_weight = gradient_halo(distance, antialias_dist, antialias_dist+0.5, 4.0);// + halo_weight(distance, antialias_dist, 2.5 + antialias_dist, 0.5);
    let empire_halo : vec4<f32> = a.empire_halo * empire_halo_weight.x + b.empire_halo * empire_halo_weight.y + c.empire_halo * empire_halo_weight.z;

    // rank halo
    var nearest = c;
    if circle_distance.x < min(circle_distance.y,circle_distance.z) {
        nearest = a;
    } else if circle_distance.y < circle_distance.z {
        nearest = b;
    }

    let rank_halo_weight = banded_rank_halo(p - nearest.pos.xy, antialias_dist, 5.0, 0.8, nearest.pop_rank);
    let rank_halo : vec4<f32> = nearest.col * rank_halo_weight;

    let hyperlane_col = vec4(lane.col,1.0);
    let hyperlane_alpha = 1.0 - smoothstep(0.0,antialias_dist,hyperlane_dist);

    let sel_col = mix(max(empire_halo,rank_halo),system_halo,system_halo.a);

    return mix(mix(territory_col,sel_col,sel_col.a),hyperlane_col,hyperlane_alpha);
}