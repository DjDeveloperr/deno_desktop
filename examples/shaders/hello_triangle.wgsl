struct VertexInput {
  [[location(0)]] pos: vec3<f32>;
  [[location(1)]] color: vec4<f32>;
};

struct VertexOutput {
  [[builtin(position)]] pos: vec4<f32>;
  [[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(input: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.color = input.color;
  out.pos = vec4<f32>(input.pos, 1.0);
  return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
  return in.color;
}
