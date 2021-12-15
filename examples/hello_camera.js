import { App, OPENGL_TO_WGPU_MATRIX } from "./common.js";
import { decode } from "./extern/pngs.bundle.js";
import { Vector3, Matrix4, PerspectiveFov, Deg } from "./extern/gmath.bundle.js";

export class CameraController {
  speed = 0;
  isLeftPressed = false;
  isRightPressed = false;
  isForwardPressed = false;
  isBackwardPressed = false;

  constructor(speed) {
    this.speed = speed;
  }

  onEvent(event) {
    if (event.type === "windowEvent") {
      event = event.event;
      if (event.type === "keyboardInput") {
        const pressed = event.input.state === "pressed";

        switch (event.input.keyCode) {
          case 57416: // up
            this.isForwardPressed = pressed;
            break;

          case 57424: // down
            this.isBackwardPressed = pressed;
            break;

          case 57419: // left
            this.isLeftPressed = pressed;
            break;

          case 57421: // right
            this.isRightPressed = pressed;
            break;
        }
      }
    }
  }

  /** @param {Camera} camera */
  updateCamera(camera) {
    let forward = camera.target.sub(camera.eye);
    const forwardNorm = forward.normal();
    let forwardMag = forward.mag();

    if (this.isForwardPressed && forwardMag > this.speed) {
      camera.eye = camera.eye.add(forwardNorm.mul(this.speed));
    }

    if (this.isBackwardPressed) {
      camera.eye = camera.eye.sub(forwardNorm.mul(this.speed));
    }

    const right = forwardNorm.cross(camera.up);

    forward = camera.target.sub(camera.eye);
    forwardMag = forward.mag();

    if (this.isRightPressed) {
      camera.eye = camera.target.sub(forward.add(right.mul(this.speed)).normal().mul(forwardMag));
    }

    if (this.isLeftPressed) {
      camera.eye = camera.target.sub(forward.sub(right.mul(this.speed)).normal().mul(forwardMag));
    }
  }
}

export class Camera {
  eye = new Vector3(0, 1, 2);
  target = new Vector3(0, 0, 0);
  up = Vector3.up();
  aspect = 0;
  fovy = 45.0;
  znear = 0.1;
  zfar = 100.0;

  constructor(width, height) {
    this.aspect = width / height;
  }

  buildViewProjMatrix() {
    const view = Matrix4.lookAtRh(this.eye, this.target, this.up);
    const proj = new PerspectiveFov(new Deg(this.fovy), this.aspect, this.znear, this.zfar)
      .toPerspective()
      .toMatrix4();
    return OPENGL_TO_WGPU_MATRIX.mul(proj.mul(view)).toFloat32Array();
  }
}

export class HelloCameraApp extends App {
  constructor() {
    super("Hello Camera");

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

    this.textureImage = decode(Deno.readFileSync(new URL("./extern/deno.png", import.meta.url)));

    this.camera = new Camera(this.width, this.height);
    this.cameraUniform = new Float32Array(4 * 4);
    this.updateViewProjMatrix();
    this.cameraController = new CameraController(0.2);
  }

  onEvent(event) {
    this.cameraController.onEvent(event);
  }

  updateViewProjMatrix() {
    const viewProjMatrix = this.camera.buildViewProjMatrix();
    this.cameraUniform.set(viewProjMatrix, 0);
  }

  async init() {
    const shaderModule = await this.loadShader("hello_camera");

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

    this.cameraBuffer = this.createBuffer({
      label: "Camera Buffer",
      data: this.cameraUniform,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
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

    this.cameraBindGroupLayout = this.device.createBindGroupLayout({
      entries: [
        {
          binding: 0,
          buffer: {
            type: "uniform",
          },
          visibility: GPUShaderStage.VERTEX,
        },
      ],
    });

    this.cameraBindGroup = this.device.createBindGroup({
      layout: this.cameraBindGroupLayout,
      entries: [
        {
          binding: 0,
          resource: {
            buffer: this.cameraBuffer,
          },
        },
      ],
    });

    this.pipelineLayout = this.device.createPipelineLayout({
      label: "Pipeline Layout",
      bindGroupLayouts: [this.texBindGroupLayout, this.cameraBindGroupLayout],
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
    this.cameraController.updateCamera(this.camera);
    this.updateViewProjMatrix();
    this.device.queue.writeBuffer(this.cameraBuffer, 0, this.cameraUniform);

    const renderPass = encoder.beginRenderPass({
      colorAttachments: [
        {
          view,
          storeOp: "store",
          loadValue: [0.1, 0.2, 0.3, 1],
        },
      ],
    });
    
    renderPass.setPipeline(this.renderPipeline);
    renderPass.setBindGroup(0, this.texBindGroup);
    renderPass.setBindGroup(1, this.cameraBindGroup);
    renderPass.setVertexBuffer(0, this.vertexBuffer);
    renderPass.setIndexBuffer(this.indexBuffer, "uint16");
    renderPass.drawIndexed(this.indices.length, 1);
    renderPass.endPass();
  }
}

if (import.meta.main) {
  const app = new HelloCameraApp();
  await app.run();
}
