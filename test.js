import { App } from "./app.js";

const adapter = await navigator.gpu.requestAdapter();
const device = await adapter.requestDevice();

const dimensions = {
  width: 800,
  height: 600,
};

const win = Deno.createWindow({
  title: "Test",
  resizable: false,
  ...dimensions,
});

const surface = win.createSurface(device);

const format = surface.getPreferredFormat();
surface.configure({ format });

const app = new App({ device, format });
await app.init();

function frame() {
  const texture = surface.getCurrentTexture();
  const view = texture.createView();

  const encoder = device.createCommandEncoder();
  app.render(encoder, view);
  device.queue.submit([encoder.finish()]);
}

setInterval(() => win.requestRedraw(), 1000 / 60);

for await (const event of Deno.eventLoop()) {
  if (event.type === "windowEvent" && event.windowID === win.id) {
    if (event.event.type === "closeRequested") {
      Deno.exit(0);
    }
  } else if (event.type === "redrawRequested" && event.windowID === win.id) {
    frame();
    surface.present();
  }
}
