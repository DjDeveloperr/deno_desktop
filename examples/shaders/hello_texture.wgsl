struct VertexInput {
  [[location(0)]] position: vec3<f32>;
  [[location(1)]] tex_coords: vec2<f32>;
};

struct VertexOutput {
  [[builtin(position)]] clip_position: vec4<f32>;
  [[location(0)]] tex_coords: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(in: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.tex_coords = in.tex_coords;
  out.clip_position = vec4<f32>(in.position, 1.0);
  return out;
}

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
  return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
