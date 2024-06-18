struct VertexOutput {
    @builtin(position) builtin_position: vec4<f32>,
}

struct Globals {
    view_projection: mat4x4<f32>,
    position_fbc: vec2<f32>,
    dimensions_fbc: vec2<f32>,
}

@group(0) @binding(0) var<uniform> globals: Globals;

fn fbc_to_ndc(fbc: vec2<f32>) -> vec2<f32> {
    return (globals.view_projection * vec4<f32>(fbc, 1.0, 0.0)).xy;
}

// Returns the vertext NDC given the vertex index. 
fn vertex_index_to_quad_ndc(vertext_index: u32) -> vec2f {
    var quad = array(
        // top
        vec2f(-1.0, 1.0),
        vec2f(-1.0, -1.0),
        vec2f(1.0, 1.0),
        // bottom
        vec2f(-1.0, -1.0),
        vec2f(1.0, -1.0),
        vec2f(1.0, 1.0),
    );

    return quad[vertext_index];
}

fn rectangle_sdf(dimensions: vec2<f32>, frag_coord: vec2<f32>) -> f32 {
    let offset = abs(frag_coord) - dimensions;

    return length(max(offset, vec2<f32>(0.0)) + min(max(offset.x, offset.y), 0.0));

}

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;

    output.builtin_position = vec4<f32>(vertex_index_to_quad_ndc(vertex_index), 1.0, 1.0);

    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let distance = rectangle_sdf(vec2<f32>(160.0, 120.0), input.builtin_position.xy - vec2<f32>(320.0, 240.0));

    return vec4<f32>(vec3<f32>(0.8, 0.4, 0.2), smoothstep(1.0, 0.0, distance * 0.04));
}

