import { App } from "./app.js";

const adapter = await navigator.gpu.requestAdapter();
const device = await adapter.requestDevice();

// Hack
const adapterRid = Number(
  Object.entries(Deno.resources()).find((e) => e[1] === "webGPUAdapter")[0],
);
const deviceRid = Number(
  Object.entries(Deno.resources()).find((e) => e[1] === "webGPUDevice")[0],
);

const dimensions = {
  width: 800,
  height: 600,
};

const win = Deno.createWindow({
  title: "Test",
  resizable: false,
  ...dimensions,
});

const surface = win.createSurface(device, {
  adapter: adapterRid,
  device: deviceRid,
});

const format = surface.getPreferredFormat();
surface.configure({ format });

const app = new App(device, format);
await app.init();

let times = [], fps;

function frame() {
  const texture = surface.getCurrentTexture();
  const view = texture.createView();

  const encoder = device.createCommandEncoder();
  app.render(encoder, view);
  device.queue.submit([encoder.finish()]);

  let s;
  if ((s = surface.present()) != "Good") console.log(s);

  Deno.close(surface.getRidOf(view));
  Deno.close(surface.getRidOf(texture));

  const now = performance.now();
  while (times.length > 0 && times[0] <= now - 1000) {
    times.shift();
  }
  times.push(now);
  fps = times.length;

  Deno.core.print(`\rFPS: ${fps} `);
}

// Call this in main loop
const loop = setInterval(() => win.requestRedraw(), 1000 / 60);

for await (const event of Deno.eventLoop()) {
  if (event.type === "windowEvent") {
    if (event.windowID !== win.id) continue;

    if (event.event.type === "closeRequested") {
      clearInterval(loop);
      Deno.exit(0);
    }
  } else if (event.type === "redrawRequested") {
    frame();
  } else if (event.type === "blocker") {
    Deno.unblock();
  }
}
