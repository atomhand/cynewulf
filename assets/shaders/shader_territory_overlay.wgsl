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

fn get_star_adjusted_distance_factor(a : vec4<f32>, b : vec4<f32>, c : vec4<f32>) -> f32 {
    let pv = a.xy + normalize(c.xy - a.xy) * a.w;
    return line_segment_distance(a.xy,b.xy, pv);
}

fn edge_adjusted_distance_factor(a : vec4<f32>, b : vec4<f32>, c : vec4<f32>, b_weight : f32) -> f32 {
    let af = get_star_adjusted_distance_factor(a,b,c);
    let bf = get_star_adjusted_distance_factor(b,a,c);

    let mid_weight = 0.0;//min(b_weight,1.0-b_weight) * 2.0;

    return mix(mix(af,bf,b_weight),16.0, mid_weight);
}

fn line_segment_distance(v : vec2<f32>, w : vec2<f32>, p : vec2<f32>) -> f32 {
    let l2 : f32 = dot(w-v,w-v);
    if l2 == 0.0 { return distance(p,v); }

    let t : f32 = saturate(dot((p - v), (w - v)) / l2);
    let projection : vec2<f32> = mix(v,w,t);

    return distance(p,projection);
}

/*
fn adjusted_line_segment_distance(star_a : vec4<f32>, star_b : vec4<f32>, adj_a : f32, adj_b : f32, p : vec2<f32>) -> f32 {
    let v : vec2<f32> = star_a.xy;
    let w : vec2<f32> = star_b.xy;

    let l2 : f32 = dot(w-v,w-v);
    if l2 == 0.0 { return distance(p,v) / star_a.w; }

    let t : f32 = saturate(dot((p - v), (w - v)) / l2);
    let projection : vec2<f32> = mix(v,w,t);

    let adj : f32 = mix(adj_a,adj_b,t);

    return distance(p,projection) / adj;
}
*/

/*
fn smin(a : f32, b : f32, in_k : f32) -> f32 {
    let k = in_k * 1.0;
    let r : f32 = exp2(-a/k) + exp2(-b/k);
    return -k*log2(r);
}
*/

fn smin(a : f32, b : f32, in_k : f32) -> f32 {
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

@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
    let d = min(input.barycentric.x, min(input.barycentric.y, input.barycentric.z));
    let t = 0.05 * (0.85 + sin(5.0 * 1));

    let p = input.world_pos.xz;

    // star colours
    let a = textureSample(material_color_texture,material_color_sampler, (vec2<f32>(f32(input.star_id.x % 128), f32(input.star_id.x / 128)) + vec2<f32>(0.5,0.5)) / 128.0);
    let b = textureSample(material_color_texture,material_color_sampler, (vec2<f32>(f32(input.star_id.y % 128), f32(input.star_id.y / 128)) + vec2<f32>(0.5,0.5)) / 128.0);
    let c = textureSample(material_color_texture,material_color_sampler, (vec2<f32>(f32(input.star_id.z % 128), f32(input.star_id.z / 128)) + vec2<f32>(0.5,0.5)) / 128.0);

    // star positions
    let a_pos = textureSample(star_position_texture,star_position_sampler, (vec2<f32>(f32(input.star_id.x % 128), f32(input.star_id.x / 128)) + vec2<f32>(0.5,0.5)) / 128.0);
    let b_pos = textureSample(star_position_texture,star_position_sampler, (vec2<f32>(f32(input.star_id.y % 128), f32(input.star_id.y / 128)) + vec2<f32>(0.5,0.5)) / 128.0);
    let c_pos = textureSample(star_position_texture,star_position_sampler, (vec2<f32>(f32(input.star_id.z % 128), f32(input.star_id.z / 128)) + vec2<f32>(0.5,0.5)) / 128.0);

/*
    let dist = vec3(distance(input.world_pos.xz,a_pos.xy), distance(input.world_pos.xz,b_pos.xy), distance(input.world_pos.xz,c_pos.xy));

    var edge_dist_weights = vec3<f32>(2.0);

    // Distance to each star regularised to each star's distance to its nearest neighbour
    // ... the actual territory projection range is half this distance
    let regularised_dist = dist / vec3(a_pos.w,b_pos.w,c_pos.w);
    // ab
    if all(a==b)
    {
        let b_weight = input.barycentric.y / (input.barycentric.x + input.barycentric.y);
        
        let adj = edge_adjusted_distance_factor(a_pos,b_pos,c_pos,b_weight);
        let d = max(line_segment_distance(a_pos.xy,b_pos.xy, input.world_pos.xz) / adj,
            1.0 - regularised_dist.z);
        edge_dist_weights.x = min(edge_dist_weights.x,d);
        edge_dist_weights.y = min(edge_dist_weights.y,d);
    }
    if all(b==c)
    {
        let c_weight = input.barycentric.z / (input.barycentric.y + input.barycentric.z);

        let adj = edge_adjusted_distance_factor(b_pos,c_pos,a_pos,c_weight);
        let d = max(line_segment_distance(b_pos.xy,c_pos.xy, input.world_pos.xz) / adj,
            1.0 - regularised_dist.x);
        edge_dist_weights.y = min(edge_dist_weights.y,d);
        edge_dist_weights.z = min(edge_dist_weights.z,d);
    }
    if all(a==c)
    {
        let c_weight = input.barycentric.z / (input.barycentric.x + input.barycentric.z);

        let adj = edge_adjusted_distance_factor(a_pos,c_pos,b_pos,c_weight);
        let d = max(line_segment_distance(a_pos.xy,c_pos.xy, input.world_pos.xz) / adj,
            1.0 - regularised_dist.y);
        edge_dist_weights.z = min(edge_dist_weights.z,d);
        edge_dist_weights.x = min(edge_dist_weights.x,d);
    }

    let rd = pick(smin(pick(regularised_dist),pick(edge_dist_weights),0.1));
    */
    //let rd = pick(min(pick(edge_dist_weights),regularised_dist));
    //let rd = pick(regularised_dist);

    // calculate colours for the terrain overlay

    /*
    let edge_inner = smoothstep(vec3(0.44),vec3(0.46),rd);
    let edge_outer = smoothstep(vec3(0.5),vec3(0.52),saturate(1.0-rd));
    let edge = min(edge_inner,edge_outer);

    let inner_glow = smoothstep(vec3(0.0),vec3(1.0),rd);
    let edge_glow = smoothstep(vec3(0.4),vec3(0.47),rd) * 0.5;

    var c_weight = min(edge_outer,inner_glow + edge_glow);
    */

    var distance = vec3<f32>(
        sd_circle(p, a_pos.xy, a_pos.w / 2.0),
        sd_circle(p, b_pos.xy, b_pos.w / 2.0),
        sd_circle(p, c_pos.xy, c_pos.w / 2.0)
    );
    
    if all(b==c)
    {
        let bf = get_star_adjusted_distance_factor(b_pos,c_pos,a_pos) / 2.0;
        let cf = get_star_adjusted_distance_factor(c_pos,b_pos,a_pos) / 2.0;
        let f = min(bf,cf);

        distance.y = min(distance.z,distance.y);
        distance.z = 1000.0;
        distance.y = min(distance.y, sd_uneven_capsule(p, b_pos.xy, c_pos.xy, f, f));
    }
    if all(a==b)
    {
        let af = get_star_adjusted_distance_factor(a_pos,b_pos,c_pos) / 2.0;
        let bf = get_star_adjusted_distance_factor(b_pos,a_pos,c_pos) / 2.0;
        let f = min(af,bf);

        distance.x = min(distance.x,distance.y);
        distance.x = min(distance.x, sd_uneven_capsule(p, a_pos.xy, b_pos.xy, f, f));
        distance.y = 1000.0;
    }
    if all(a==c)
    {
        let af = get_star_adjusted_distance_factor(a_pos,c_pos,b_pos) / 2.0;
        let cf = get_star_adjusted_distance_factor(c_pos,a_pos,b_pos) / 2.0;
        let f = min(af,cf);

        distance.x = min(distance.x,distance.z);
        distance.z = 1000.0;
        distance.x = min(distance.x, sd_uneven_capsule(p, a_pos.xy, c_pos.xy, f, f));
    }

    let edge_inner = vec3(1.0) - smoothstep(vec3(0.0),vec3(16.0), -distance);
    let edge_outer = vec3(1.0) - smoothstep(vec3(0.0), vec3(0.1), distance);

    let c_weight = saturate(min(edge_inner,edge_outer));

    return a * c_weight.x + b * c_weight.y + c * c_weight.z;
}