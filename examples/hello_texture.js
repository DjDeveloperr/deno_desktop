import { App } from "./common.js";
import { decode } from "./extern/pngs.bundle.js";

export class HelloTextureApp extends App {
  constructor() {
    super("Hello Texture");

    const vertices = [
      [-0.0868241, 0.49240386, 0],
      [-0.49513406, 0.06958647, 0],
      [-0.21918549, -0.44939706, 0],
      [0.35966998, -0.3473291, 0],
      [0.44147372, 0.2347359, 0],
    ];

    const texCoords = [
      [0.4131759, 0.00759614],
      [0.0048659444, 0.43041354],
      [0.28081453, 0.949397],
      [0.85967, 0.84732914],
      [0.9414737, 0.2652641],
    ];

    const indices = [
      0, 1, 4,
      1, 2, 4,
      2, 3, 4,
      0,
    ];

    this.vertices = new Float32Array(vertices.length * vertices[0].length + texCoords.length * texCoords[0].length);
    for (let i = 0; i < vertices.length; i++) {
      const vertex = vertices[i], texCoord = texCoords[i];
      const offset = i * vertex.length + i * texCoord.length;
      this.vertices.set(vertex, offset);
      this.vertices.set(texCoord, offset + vertex.length);
    }

    this.indices = new Uint16Array(indices);

    this.textureImage = decode(Deno.readFileSync(new URL("./extern/tree.png", import.meta.url)));
  }

  async init() {
    const shaderModule = await this.loadShader("hello_texture");

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

    this.texture = this.device.createTexture({
      label: "Tree Texture",
      size: {
        width: this.textureImage.width,
        height: this.textureImage.height,
        depthOrArrayLayers: 1,
      },
      mipLevelCount: 1,
      sampleCount: 1,
      dimension: "2d",
      format: "rgba8unorm-srgb",
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
    });

    this.device.queue.writeTexture({
      texture: this.texture,
    }, this.textureImage.image, {
      offset: 0,
      bytesPerRow: this.textureImage.width * 4,
      rowsPerImage: this.textureImage.height,
    }, {
      width: this.textureImage.width,
      height: this.textureImage.height,
      depthOrArrayLayers: 1,
    });

    this.textureView = this.texture.createView();
    this.textureSampler = this.device.createSampler({
      label: "Tree Texture Sampler",
      addressModeU: "clamp-to-edge",
      addressModeV: "clamp-to-edge",
      addressModeW: "clamp-to-edge",
      magFilter: "linear",
      minFilter: "nearest",
      mipmapFilter: "nearest",
    });

    this.texBindGroupLayout = this.device.createBindGroupLayout({
      label: "Texture Bind Group Layout",
      entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.FRAGMENT,
          texture: {
            multisampled: false,
            viewDimension: "2d",
            sampleType: "float",
          },
        },
        {
          binding: 1,
          visibility: GPUShaderStage.FRAGMENT,
          sampler: {
            type: "filtering",
          },
        },
      ],
    });

    this.texBindGroup = this.device.createBindGroup({
      label: "Texture Bind Group",
      layout: this.texBindGroupLayout,
      entries: [
        {
          binding: 0,
          resource: this.textureView,
        },
        {
          binding: 1,
          resource: this.textureSampler,
        },
      ],
    });

    this.pipelineLayout = this.device.createPipelineLayout({
      label: "Pipeline Layout",
      bindGroupLayouts: [this.texBindGroupLayout],
    });

    this.renderPipeline = this.device.createRenderPipeline({
      layout: this.pipelineLayout,
      vertex: {
        module: shaderModule,
        entryPoint: "vs_main",
        buffers: [
          {
            arrayStride: 4 * 3 + 4 * 2,
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
                format: "float32x2",
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
    renderPass.setBindGroup(0, this.texBindGroup);
    renderPass.setVertexBuffer(0, this.vertexBuffer);
    renderPass.setIndexBuffer(this.indexBuffer, "uint16");
    renderPass.drawIndexed(this.indices.length, 1);
    renderPass.endPass();
  }
}

if (import.meta.main) {
  const app = new HelloTextureApp();
  await app.run();
}
