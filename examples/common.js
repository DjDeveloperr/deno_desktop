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

  mainLoop() {
    this.window.requestRedraw();
  }

  async run() {
    this.adapter = await navigator.gpu.requestAdapter();
    if (!this.adapter) {
      throw new Error("No GPU adapter available!");
    }

    this.device = await this.adapter.requestDevice();

    this.window = Deno.createWindow({
      title: this.title,
      width: this.width,
      height: this.height,
      resizable: false,
    });

    this.surface = this.window.createSurface(this.device);
    this.format = this.surface.getPreferredFormat();
    this.surface.configure({ format: this.format });

    await this.init();

    this.mainLoopInterval = setInterval(() => this.mainLoop(), 1000 / 60);

    for await (const event of Deno.eventLoop()) {
      if (event.type === "windowEvent" && event.windowID === this.window.id) {
        if (event.event.type === "closeRequested") {
          this.cleanup();
          Deno.exit(0);
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
