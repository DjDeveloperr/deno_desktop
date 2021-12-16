import { enableValidationErrors } from "./extern/gpu_err.bundle.js";
import { Matrix4 } from "./extern/gmath.bundle.js";

export const OPENGL_TO_WGPU_MATRIX = Matrix4.from(
  1.0, 0.0, 0.0, 0.0,
  0.0, 1.0, 0.0, 0.0,
  0.0, 0.0, 0.5, 0.0,
  0.0, 0.0, 0.5, 1.0,
);

/** Base Application used by all examples */
export class App {
  constructor(title, width = 800, height = 600) {
    this.title = title;
    this.width = width;
    this.height = height;
  }

  // To be extended by subclasses
  async init() {}
  async render(_encoder, _view) {}
  async onEvent(_event) {}

  mainLoop() {
    this.window.requestRedraw();
  }

  async run() {
    this.adapter = await navigator.gpu.requestAdapter();
    if (!this.adapter) {
      throw new Error("No GPU adapter available!");
    }

    this.device = await this.adapter.requestDevice();
    enableValidationErrors(this.device, true);

    this.window = Deno.createWindow({
      title: this.title,
      width: this.width,
      height: this.height,
      resizable: true,
    });

    this.surface = this.window.createSurface(this.device);

    this.format = this.surface.getPreferredFormat();
    this.surface.configure({
      format: this.format,
      width: this.width,
      height: this.height,
    });

    await this.init();

    this.mainLoopInterval = setInterval(() => this.mainLoop(), 1000 / 60);

    for await (const event of Deno.eventLoop()) {
      await this.onEvent(event);
      if (event.type === "windowEvent" && event.windowID === this.window.id) {
        if (event.event.type === "closeRequested") {
          this.cleanup();
          Deno.exit(0);
        } else if (event.event.type === "resized") {
          this.width = event.event.width;
          this.height = event.event.height;
          this.surface.configure({
            format: this.format,
            width: this.width,
            height: this.height,
          });
        }
      } else if (event.type === "redrawRequested" && event.windowID === this.window.id) {
        const texture = this.surface.getCurrentTexture();
        const view = texture.createView();
        const encoder = this.device.createCommandEncoder();
        this.render(encoder, view);
        this.device.queue.submit([encoder.finish()]);
        this.surface.present();
      }
    }
  }

  cleanup() {
    if (this.mainLoopInterval) {
      clearInterval(this.mainLoopInterval);
      this.mainLoopInterval = null;
    }

    this.surface.destroy();
    this.window.close();
    this.device.destroy();
  }

  async loadShader(name) {
    const code = await Deno.readTextFile(new URL(`./shaders/${name}.wgsl`, import.meta.url));
    return this.device.createShaderModule({
      label: name,
      code,
    });
  }

  createBuffer({ label, usage, data, size }) {
    const buffer = this.device.createBuffer({
      label,
      usage,
      size: ((data ? data.byteLength : size)  + 3) & ~3,
      mappedAtCreation: data ? true : false,
    });

    if (data) {
      new data.constructor(buffer.getMappedRange()).set(data);
      buffer.unmap();
    }

    return buffer;
  }
}
