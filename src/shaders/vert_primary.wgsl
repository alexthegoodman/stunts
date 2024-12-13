// // Vertex input structure definition
// struct VertexInput {
//     @location(0) position: vec3<f32>,
//     @location(1) tex_coords: vec2<f32>,
//     @location(2) color: vec4<f32>,  // Receive color from the vertex buffer
// };

// struct VertexOutput {
//     @builtin(position) clip_position: vec4<f32>,
//     @location(0) tex_coords: vec2<f32>,
//     @location(1) color: vec4<f32>,  // Pass color to the fragment shader
// };

// @vertex
// fn vs_main(in: VertexInput) -> VertexOutput {
//     var out: VertexOutput;
//     out.clip_position = vec4(in.position, 1.0);
//     out.tex_coords = in.tex_coords;
//     out.color = in.color;  // Pass color from input to output
//     return out;
// }

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct ModelUniforms {
    model: mat4x4<f32>
};

struct WindowSize {
    width: f32,
    height: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model_uniforms: ModelUniforms;
@group(2) @binding(0) var<uniform> window_size: WindowSize;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // First apply model transform in original coordinate space
    let model_pos = model_uniforms.model * vec4<f32>(vertex.position, 1.0);
    
    // Then convert to NDC space
    var ndc_pos = model_pos.xyz;
    ndc_pos.x = (ndc_pos.x / window_size.width) * 2.0 - 1.0;
    ndc_pos.y = -((ndc_pos.y / window_size.height) * 2.0 - 1.0); // Flip Y coordinate
    
    // Finally apply camera transform
    out.clip_position = camera.view_proj * vec4<f32>(ndc_pos, 1.0);
    out.tex_coords = vertex.tex_coords;
    out.color = vertex.color;
    
    return out;
}