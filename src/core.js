Deno.eventLoop = async function* () {
  let event;
  while ((event = (await Deno.core.opAsync("op_next_event")))) {
    yield event;
  }
};

Deno.unblock = function () {
  Deno.core.opSync("op_drop_blocker");
};

Deno.createWindow = function (options) {
  return new WinitWindow(Deno.core.opSync("op_create_window", options), options);
};

let symRid;

class GPUCanvasContext {
  #window;
  #rid;
  #rids;
  #baseTex;

  constructor(window, rid, rids, device) {
    this.#window = window;
    this.#rid = rid;
    this.#rids = rids;

    if (!this.#baseTex) {
      this.#baseTex = device.createTexture({
        size: {
          width: 1,
          height: 1,
        },
        format: "bgra8unorm-srgb",
        usage: GPUTextureUsage.RENDER_ATTACHMENT,
      });

      if (!symRid) {
        const symbols = Object.getOwnPropertySymbols(this.#baseTex);
        symRid = symbols.find((sym) => sym.description === "[[rid]]");
      }
      Deno.close(this.#baseTex[symRid]);
    }
  }

  getPreferredFormat() {
    const format = Deno.core.opSync("op_webgpu_surface_get_preferred_format", {
      surfaceRid: this.#rid,
      adapterRid: this.#rids.adapter,
    });
    return ({
      Bgra8UnormSrgb: "bgra8unorm-srgb",
    })[format];
  }

  configure(options = {}) {
    const done = Deno.core.opSync("op_webgpu_configure_surface", {
      surfaceRid: this.#rid,
      format: options.format ?? this.getPreferredFormat(),
      width: options.width ?? this.#window.width,
      height: options.height ?? this.#window.height,
      usage: options.usage ?? GPUTextureUsage.RENDER_ATTACHMENT,
      deviceRid: this.#rids.device,
    });
    if (!done) throw new Error("failed");
  }

  present() {
    return Deno.core.opSync("op_webgpu_surface_present", {
      surfaceRid: this.#rid,
      adapterRid: this.#rids.adapter,
    });
  }

  getCurrentTexture() {
    const currentTextureRid = Deno.core.opSync(
      "op_webgpu_surface_get_current_texture",
      {
        surfaceRid: this.#rid,
        adapterRid: this.#rids.adapter,
      }
    );
    this.#baseTex[symRid] = currentTextureRid;
    return this.#baseTex;
  }

  getRidOf(obj) {
    return obj[symRid];
  }
}

class WinitWindow {
  #rid;
  #id;
  #visible;
  #width;
  #height;

  get rid() {
    return this.#rid;
  }

  get id() {
    return this.#id;
  }
  
  constructor([id, rid], options) {
    this.#id = id;
    this.#rid = rid;
    this.#visible = options.visible ?? true;
    this.#width = options.width ?? 800;
    this.#height = options.height ?? 600;
  }

  get visible() {
    return this.#visible;
  }

  set visible(visible) {
    Deno.core.opSync("op_window_set_visible", [this.#rid, visible]);
  }

  get width() {
    return this.#width;
  }
  
  get height() {
    return this.#height;
  }

  requestRedraw() {
    Deno.core.opSync("op_window_request_redraw", this.#rid);
  }

  createSurface(device, rids) {
    const rid = Deno.core.opSync("op_webgpu_create_surface", { windowRid: this.#rid });
    return new GPUCanvasContext(this, rid, rids, device);
  }

  close() {
    Deno.close(this.rid);
  }
}
