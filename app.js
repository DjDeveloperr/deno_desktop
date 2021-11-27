export class App {
  constructor(device, format) {
    this.device = device;
    this.format = format;
  }

  init() {
    const shaderCode = `
[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] in_vertex_index: u32) -> [[builtin(position)]] vec4<f32> {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}
[[stage(fragment)]]
fn fs_main() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
`;

    const shaderModule = this.device.createShaderModule({
      code: shaderCode,
    });

    const pipelineLayout = this.device.createPipelineLayout({
      bindGroupLayouts: [],
    });

    this.renderPipeline = this.device.createRenderPipeline({
      layout: pipelineLayout,
      vertex: {
        module: shaderModule,
        entryPoint: "vs_main",
      },
      fragment: {
        module: shaderModule,
        entryPoint: "fs_main",
        targets: [
          {
            format: this.format,
          },
        ],
      },
    });
  }

  render(encoder, view) {
    const renderPass = encoder.beginRenderPass({
      colorAttachments: [
        {
          view,
          storeOp: "store",
          loadValue: {r: 0, g:1, b:0, a:1},
        },
      ],
    });
    renderPass.setPipeline(this.renderPipeline);
    renderPass.draw(3, 1);
    renderPass.endPass();
  }
}
