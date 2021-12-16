Deno.nextEvent = function nextEvent() {
  const promise = Deno.core.opAsync("op_next_event");
  // Unref the op so that it does not keep event loop alive.
  Deno.core.unrefOp(promise[Symbol.for("Deno.core.internalPromiseId")]);
  return promise;
}

Deno.eventLoop = async function* eventLoop() {
  let event;
  while ((event = (await Deno.nextEvent()))) {
    yield event;
  }
};

Deno.createWindow = function createWindow(options) {
  return new WinitWindow(Deno.core.opSync("op_create_window", options));
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
  #currentTexture;

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
    this.#currentTexture = null;
  }

  present() {
    const status = Deno.core.opSync("op_webgpu_surface_present", {
      surfaceRid: this.#rid,
      adapterRid: this.#rids.adapter,
    });
    this.#currentTexture = null;
    return status;
  }

  getCurrentTexture() {
    if (this.#currentTexture) return this.#currentTexture;
    const currentTextureRid = Deno.core.opSync(
      "op_webgpu_surface_get_current_texture",
      {
        surfaceRid: this.#rid,
        adapterRid: this.#rids.adapter,
      }
    );
    this.#baseTex[getSymbolOf(this.#baseTex, "[[rid]]")] = currentTextureRid;
    this.#currentTexture = this.#baseTex;
    return this.#baseTex;
  }

  destroy() {
    Deno.core.opSync("op_webgpu_surface_drop", this.#rid);
  }
}

globalThis.GPUCanvasContext = GPUCanvasContext;

class WinitWindow {
  #rid;
  #id;

  constructor([id, rid]) {
    this.#id = id;
    this.#rid = rid;
  }

  get rid() {
    return this.#rid;
  }

  get id() {
    return this.#id;
  }

  get width() {
    return this.getSize().width;
  }
  
  get height() {
    return this.getSize().height;
  }

  requestRedraw() {
    Deno.core.opSync("op_window_request_redraw", this.#rid);
  }

  requestUserAttention(type) {
    Deno.core.opSync("op_window_request_user_attention", [this.#rid, type ?? null]);
  }

  getFullscreen() {
    return Deno.core.opSync("op_window_fullscreen", this.#rid);
  }

  getInnerPosition() {
    return Deno.core.opSync("op_window_inner_position", this.#rid);
  }

  getSize() {
    return Deno.core.opSync("op_window_inner_size", this.#rid);
  }

  getOuterSize() {
    return Deno.core.opSync("op_window_outer_size", this.#rid);
  }

  getScaleFactor() {
    return Deno.core.opSync("op_window_scale_factor", this.#rid);
  }

  setAlwaysOnTop(value) {
    Deno.core.opSync("op_window_set_always_on_top", [this.#rid, value]);
  }

  setCursorGrab(value) {
    Deno.core.opSync("op_window_set_cursor_grab", [this.#rid, value]);
  }

  setCursorIcon(icon) {
    Deno.core.opSync("op_window_set_cursor_icon", [this.#rid, icon]);
  }

  setCursorPosition(pos) {
    Deno.core.opSync("op_window_set_cursor_position", [this.#rid, pos]);
  }

  setCursorVisible(value) {
    Deno.core.opSync("op_window_set_cursor_visible", [this.#rid, value]);
  }

  setDecorations(value) {
    Deno.core.opSync("op_window_set_decorations", [this.#rid, value]);
  }

  setFullscreen(fullscreen) {
    Deno.core.opSync("op_window_set_fullscreen", [this.#rid, fullscreen]);
  }

  setImePosition(pos) {
    Deno.core.opSync("op_window_set_ime_position", [this.#rid, pos]);
  }

  setSize(size) {
    Deno.core.opSync("op_window_set_inner_size", [this.#rid, size]);
  }

  setMaxSize({ width, height }) {
    Deno.core.opSync("op_window_set_max_inner_size", [this.#rid, width, height]);
  }

  setMaximized(value) {
    Deno.core.opSync("op_window_set_maximized", [this.#rid, value]);
  }

  setMinSize({ width, height }) {
    Deno.core.opSync("op_window_set_min_inner_size", [this.#rid, width, height]);
  }

  setMinimized(value) {
    Deno.core.opSync("op_window_set_minimized", [this.#rid, value]);
  }

  setPosition(pos) {
    Deno.core.opSync("op_window_set_outer_position", [this.#rid, pos]);
  }

  setResizable(value) {
    Deno.core.opSync("op_window_set_resizable", [this.#rid, value]);
  }

  setTitle(title) {
    Deno.core.opSync("op_window_set_title", [this.#rid, title]);
  }

  setVisible(value) {
    Deno.core.opSync("op_window_set_visible", [this.#rid, value]);
  }

  setIcon({ data, width, height }) {
    Deno.core.opSync("op_window_set_window_icon", [this.#rid, width, height], data);
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

Deno.WinitWindow = WinitWindow;
