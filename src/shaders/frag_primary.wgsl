@group(1) @binding(1) var texture: texture_2d<f32>;
@group(1) @binding(2) var texture_sampler: sampler;

struct FragmentInput {
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(texture, texture_sampler, in.tex_coords);
    return tex_color * in.color;
}