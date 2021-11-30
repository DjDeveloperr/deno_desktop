[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] idx: u32) -> [[builtin(position)]] vec4<f32> {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(0.0, 0.5),
    vec2<f32>(-0.5, -0.5),
    vec2<f32>(0.5, -0.5)
  );
  var vert = pos[idx];
  return vec4<f32>(vert.x, vert.y, 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main() -> [[location(0)]] vec4<f32> {
  return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}
