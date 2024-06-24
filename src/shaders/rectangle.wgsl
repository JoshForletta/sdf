struct VertexOutput {
    @builtin(position) builtin_position: vec4<f32>,
}

struct Globals {
    view_projection: mat4x4<f32>,
    position: vec2<f32>,
    dimensions: vec2<f32>,
    corner_radii: vec4<f32>,
    outer_color: vec4<f32>,
    inner_color: vec4<f32>,
    phase: f32,
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

fn select_corner_radius(corner_radii: vec4<f32>, point: vec2<f32>) -> f32 {
    var right = mix(
        corner_radii.x, // top-right
        corner_radii.y, // bottom-right
        step(0.0, point.y)
    );
    var left = mix(
        corner_radii.z, // top-left
        corner_radii.w, // bottom-left
        step(0.0, point.y)
    );

    return mix(left, right, step(0.0, point.x));
}

fn rectangle_sdf(
    position: vec2<f32>,
    dimensions: vec2<f32>,
    corner_radii: vec4<f32>,
    frag_coord: vec2<f32>
) -> f32 {
    var point = frag_coord - position;
    let corner_radius = select_corner_radius(corner_radii, point);
    let offset = abs(point) - dimensions + corner_radius;

    return length(max(offset, vec2<f32>(0.0))) + min(max(offset.x, offset.y), 0.0) - corner_radius;
}

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;

    output.builtin_position = vec4<f32>(vertex_index_to_quad_ndc(vertex_index), 1.0, 1.0);

    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let outer_color = vec3<f32>(0.8, 0.4, 0.2);
    let inner_color = vec3<f32>(0.2, 0.4, 0.8);
    var color: vec3<f32> = outer_color;

    let distance = rectangle_sdf(
        globals.position,
        globals.dimensions,
        globals.corner_radii,
        input.builtin_position.xy,
    );

    if distance < 0.0 {
        color = inner_color;
    }

    let alpha = (sin(distance * 0.5 + globals.phase) + 1) / 2;
    let fade = 1.0 / abs(distance * 0.02);

    return vec4<f32>(color, alpha * fade);
}

