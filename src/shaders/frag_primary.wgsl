struct FragmentInput {
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,  // Receive color from vertex shader
};

@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    return in.color;
}