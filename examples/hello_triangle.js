import { App } from "./common.js";

export class HelloTriangleApp extends App {
  constructor() {
    super("Hello Triangle");

    const vertices = [
      [   0,  0.5, 0],
      [ 0.5, -0.5, 0],
      [-0.5, -0.5, 0],
    ];

    const colors = [
      [1, 0, 0, 1],
      [0, 1, 0, 1],
      [0, 0, 1, 1],
    ];

    const indices = [
      0, 1, 2,
    ];

    this.vertices = new Float32Array(vertices.length * 3 + colors.length * 4);
    for (let i = 0; i < vertices.length; i++) {
      const vertex = vertices[i], color = colors[i];
      const offset = i * vertex.length + i * color.length;
      this.vertices.set(vertex, offset);
      this.vertices.set(color, offset + vertex.length);
    }

    this.indices = new Uint16Array(indices);
  }

  async init() {
    const shaderModule = await this.loadShader("hello_triangle");

    this.vertexBuffer = this.createBuffer({
      label: "Vertex Buffer",
      data: this.vertices,
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
            arrayStride: 4 * 3 + 4 * 4,
            stepMode: "vertex",
            attributes: [
              {
                offset: 0,
                shaderLocation: 0,
                format: "float32x3",
              },
              {
                offset: 4 * 3,
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
