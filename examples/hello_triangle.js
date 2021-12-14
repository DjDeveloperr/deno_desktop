import { App } from "./common.js";

export class HelloTriangleApp extends App {
  constructor() {
    super("Hello Triangle");

    const vertexes = [
      [   0,  0.5, 0, 1],
      [ 0.5, -0.5, 0, 1],
      [-0.5, -0.5, 0, 1],
    ];

    const colors = [
      [1, 0, 0, 1],
      [0, 1, 0, 1],
      [0, 0, 1, 1],
    ];

    const indices = [
      0, 1, 2,
    ];

    this.vertexes = new Float32Array(vertexes.length * 4 * 2);
    for (let i = 0; i < vertexes.length; i++) {
      this.vertexes.set(vertexes[i], i * 2 * 4);
      this.vertexes.set(colors[i], i * 2 * 4 + 4);
    }

    this.indices = new Uint16Array(indices);
  }

  async init() {
    const shaderModule = await this.loadShader("hello_triangle");

    this.vertexBuffer = this.createBuffer({
      label: "Vertex Buffer",
      data: this.vertexes,
      usage: GPUBufferUsage.VERTEX,
    });

    this.indexBuffer = this.createBuffer({
      label: "Index Buffer",
      data: this.indices,
      usage: GPUBufferUsage.INDEX,
    });

    this.renderPipeline = this.device.createRenderPipeline({
      vertex: {
        module: shaderModule,
        entryPoint: "vs_main",
        buffers: [
          {
            arrayStride: 4 * 4 * 2,
            stepMode: "vertex",
            attributes: [
              {
                offset: 0,
                shaderLocation: 0,
                format: "float32x4",
              },
              {
                offset: 4 * 4,
                shaderLocation: 1,
                format: "float32x4",
              },
            ],
          },
        ],
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
    renderPass.setVertexBuffer(0, this.vertexBuffer);
    renderPass.setIndexBuffer(this.indexBuffer, "uint16");
    renderPass.drawIndexed(this.indices.length, 1);
    renderPass.endPass();
  }
}

if (import.meta.main) {
  const app = new HelloTriangleApp();
  await app.run();
}
