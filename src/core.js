Deno.eventLoop = async function* () {
  let event;
  while ((event = (await Deno.core.opAsync("op_next_event")))) {
    yield event;
  }
};

Deno.createWindow = function (options) {
  return new WinitWindow(Deno.core.opSync("op_create_window", options), options);
};

let symbolCache = {};

function getSymbolOf(obj, name) {
  if (symbolCache[name]) {
    return symbolCache[name];
  }
  const symbol = Object.getOwnPropertySymbols(obj).find((symbol) => symbol.description === name);
  if (!symbol) {
    throw new Error(`No symbol ${name} found`);
  }
  symbolCache[name] = symbol;
  return symbol;
}

function getRidOf(obj) {
  return obj[getSymbolOf(obj, "[[rid]]")];
}

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
      this.#baseTex.destroy();
    }
  }

  getPreferredFormat() {
    return Deno.core.opSync("op_webgpu_surface_get_preferred_format", {
      surfaceRid: this.#rid,
      adapterRid: this.#rids.adapter,
    });
  }

  configure(options = {}) {
    Deno.core.opSync("op_webgpu_configure_surface", {
      surfaceRid: this.#rid,
      deviceRid: this.#rids.device,
      format: options.format ?? this.getPreferredFormat(),
      width: options.width ?? this.#window.width,
      height: options.height ?? this.#window.height,
      usage: options.usage ?? GPUTextureUsage.RENDER_ATTACHMENT,
    });
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
    this.#baseTex[getSymbolOf(this.#baseTex, "[[rid]]")] = currentTextureRid;
    return this.#baseTex;
  }

  destroy() {
    Deno.core.opSync("op_webgpu_surface_drop", this.#rid);
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

  createSurface(device) {
    const rid = Deno.core.opSync("op_webgpu_create_surface", { windowRid: this.#rid });
    const adapter = device[getSymbolOf(device, "[[device]]")].adapter;
    return new GPUCanvasContext(this, rid, {
      device: device[getSymbolOf(device, "[[device]]")].rid,
      adapter: adapter[getSymbolOf(adapter, "[[adapter]]")].rid,
    }, device);
  }

  close() {
    Deno.close(this.rid);
  }
}
