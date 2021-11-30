import { App } from "./common.js";

export class HelloTriangleApp extends App {
  constructor() {
    super("Hello Triangle");
  }

  async init() {
    const shaderModule = await this.loadShader("hello_triangle");
    this.renderPipeline = this.device.createRenderPipeline({
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
          loadValue: [0, 0, 0, 1],
        },
      ],
    });
    renderPass.setPipeline(this.renderPipeline);
    renderPass.draw(3, 1);
    renderPass.endPass();
  }
}

if (import.meta.main) {
  const app = new HelloTriangleApp();
  await app.run();
}
