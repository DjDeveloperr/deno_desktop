use std::cell::RefCell;
use std::rc::Rc;

use deno_runtime::deno_core::Extension;
use deno_runtime::deno_core::op_async;
use deno_runtime::deno_core::OpState;
use deno_runtime::deno_core::error::AnyError;
use serde::Serialize;
use deno_runtime::deno_core::serde_json::Value;
use deno_runtime::deno_core::serde_json::json;
use winit_main::Blocker;
use winit_main::reexports::dpi::PhysicalPosition;
use winit_main::reexports::dpi::PhysicalSize;
use winit_main::reexports::event::DeviceEvent;
use winit_main::reexports::event::ElementState;
use winit_main::reexports::event::Event;
use winit_main::reexports::event::KeyboardInput;
use winit_main::reexports::event::MouseButton;
use winit_main::reexports::event::MouseScrollDelta;
use winit_main::reexports::event::StartCause;
use winit_main::reexports::event::TouchPhase;
use winit_main::reexports::event::WindowEvent;
use winit_main::reexports::window::Theme;
use crate::util::hash;
use crate::EVENT_RECEIVER;

fn serialize_physical_size<T: Serialize>(size: PhysicalSize<T>) -> Value {
    json!({ "width": size.width, "height": size.height })
}

fn serialize_physical_position<T: Serialize>(pos: PhysicalPosition<T>) -> Value {
    json!({ "x": pos.x, "y": pos.y })
}

fn serialize_keyboard_input(input: KeyboardInput) -> Value {
    json!({
        "keyCode": input.scancode,
        "state": match input.state {
            ElementState::Pressed => "pressed",
            ElementState::Released => "released"
        },
    })
}

fn serialize_mouse_scroll_delta(delta: MouseScrollDelta) -> Value {
    match delta {
        MouseScrollDelta::LineDelta(x, y) => json!({ "type": "lineDelta", "x": x, "y": y }),
        MouseScrollDelta::PixelDelta(pos) => json!({ "type": "pixelDelta", "position": serialize_physical_position(pos) }),
    }
}

fn serialize_touch_phase(phase: TouchPhase) -> Value {
    match phase {
        TouchPhase::Started => json!("started"),
        TouchPhase::Moved => json!("moved"),
        TouchPhase::Ended => json!("ended"),
        TouchPhase::Cancelled => json!("cancelled"),
    }
}

pub fn serialize_event<'a>(event: Event<'a, Blocker>) -> Value {
    match event {
        Event::NewEvents(cause) => {
            let cause = match cause {
                StartCause::ResumeTimeReached { start, requested_resume } => json!({ "type": "resumeTimeReached", "start": start.elapsed().as_millis().to_string(), "requestedResume": requested_resume.elapsed().as_millis().to_string() }),
                StartCause::WaitCancelled { start, requested_resume } => json!({ "type": "waitCancelled", "start": start.elapsed().as_millis().to_string(), "requestedResume": if let Some(requested_resume) = requested_resume { Some(requested_resume.elapsed().as_millis().to_string()) } else { None }}),
                StartCause::Poll => json!({ "type": "poll" }),
                StartCause::Init => json!({ "type": "init" }),
            };
            json!({ "type": "newEvents", "cause": cause })
        },
        Event::WindowEvent { window_id, event } => {
            let event = match event {
                WindowEvent::Resized(size) => json!({ "type": "resized", "size": serialize_physical_size(size) }),
                WindowEvent::Moved(pos) => json!({ "type": "moved", "position": serialize_physical_position(pos) }),
                WindowEvent::CloseRequested => json!({ "type": "closeRequested" }),
                WindowEvent::Destroyed => json!({ "type": "destroyed" }),
                WindowEvent::DroppedFile(path) => json!({ "type": "droppedFile", "path": path.into_os_string().into_string() }),
                WindowEvent::HoveredFile(path) => json!({ "type": "hoveredFile", "path": path.into_os_string().into_string() }),
                WindowEvent::HoveredFileCancelled => json!({ "type": "hoveredFileCancelled" }),
                WindowEvent::ReceivedCharacter(ch) => json!({ "type": "receivedCharacter", "char": ch }),
                WindowEvent::Focused(focused) => json!({ "type": "focused", "focused": focused }),
                WindowEvent::KeyboardInput { device_id, input, is_synthetic } => json!({
                    "type": "keyboardInput",
                    "deviceID": hash(device_id),
                    "isSynthetic": is_synthetic,
                    "input": serialize_keyboard_input(input)
                }),
                WindowEvent::ModifiersChanged(state) => json!({ "type": "modifiersChanged", "state": state.bits() }),
                WindowEvent::CursorMoved { device_id, position, .. } => json!({
                    "type": "cursorMoved",
                    "deviceID": hash(device_id),
                    "position": serialize_physical_position(position)
                }),
                WindowEvent::CursorEntered { device_id } => json!({
                    "type": "cursorEntered",
                    "deviceID": hash(device_id)
                }),
                WindowEvent::CursorLeft { device_id } => json!({ "type": "cursorLeft", "deviceID": hash(device_id) }),
                WindowEvent::MouseWheel { device_id, delta, phase, .. } => json!({
                    "type": "mouseWheel",
                    "deviceID": hash(device_id),
                    "delta": serialize_mouse_scroll_delta(delta),
                    "phase": serialize_touch_phase(phase),
                }),
                WindowEvent::MouseInput { device_id, state, button, .. } => json!({
                    "type": "mouseInput",
                    "deviceID": hash(device_id),
                    "state": match state {
                        ElementState::Pressed => "pressed",
                        ElementState::Released => "released"
                    },
                    "button": match button {
                        MouseButton::Left => json!("left"),
                        MouseButton::Right => json!("right"),
                        MouseButton::Middle => json!("middle"),
                        MouseButton::Other(n) => json!(n),
                    },
                }),
                WindowEvent::TouchpadPressure { device_id, pressure, stage } => json!({
                    "type": "touchpadPressure",
                    "deviceID": hash(device_id),
                    "pressure": pressure,
                    "stage": stage,
                }),
                WindowEvent::AxisMotion { device_id, axis, value } => json!({
                    "type": "axisMotion",
                    "deviceID": hash(device_id),
                    "axis": axis,
                    "value": value,
                }),
                WindowEvent::Touch(input) => json!({
                    "type": "touch",
                    "device_id": hash(input.device_id),
                    "phase": serialize_touch_phase(input.phase),
                    "location": serialize_physical_position(input.location),
                    // leaving out input.force as that's ios specific for now
                    "id": input.id,
                }),
                WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } => json!({
                    "type": "scaleFactorChanged",
                    "scaleFactor": scale_factor,
                    "newInnerSize": serialize_physical_size(*new_inner_size)
                }),
                WindowEvent::ThemeChanged(theme) => json!({
                    "type": "themeChanged",
                    "theme": match theme {
                        Theme::Light => "light",
                        Theme::Dark => "dark"
                    }
                }),
            };
            json!({ "type": "windowEvent", "windowID": hash(window_id), "event": event })
        },
        Event::DeviceEvent { device_id, event } => {
            let event = match event {
                DeviceEvent::Added => json!({ "type": "added" }),
                DeviceEvent::Removed => json!({ "type": "removed" }),
                DeviceEvent::MouseMotion { delta } => json!({ "type": "mouseMotion", "delta": delta }),
                DeviceEvent::MouseWheel { delta } => json!({ "type": "mouseWheel", "delta": serialize_mouse_scroll_delta(delta) }),
                DeviceEvent::Motion { axis, value } => json!({ "type": "motion", "axis": axis, "value": value }),
                DeviceEvent::Button { button, state } => json!({ "type": "button", "button": button, "state": match state { ElementState::Pressed => "pressed", ElementState::Released => "released" } }),
                DeviceEvent::Key(input) => json!({ "type": "key", "input": serialize_keyboard_input(input) }),
                DeviceEvent::Text { codepoint } => json!({ "type": "text", "codepoint": codepoint }),
            };
            json!({ "type": "deviceEvent", "deviceID": hash(device_id), "event": event })
        },
        Event::UserEvent(_) => json!({ "type": "blocker" }),
        Event::Suspended => json!({ "type": "suspended" }),
        Event::Resumed => json!({ "type": "resumed" }),
        Event::MainEventsCleared => json!({ "type": "mainEventsCleared" }),
        Event::RedrawRequested(wid) => json!({ "type": "redrawRequested", "windowID": hash(wid) }),
        Event::RedrawEventsCleared => json!({ "type": "redrawEventsCleared" }),
        Event::LoopDestroyed => json!({ "type": "loopDestroyed" }),
    }
}

pub async fn op_next_event(_: Rc<RefCell<OpState>>, _: (), _: ()) -> Result<Value, AnyError> {
    Ok(tokio::task::spawn_blocking(|| {
        let arc = EVENT_RECEIVER.lock().unwrap();
        let asref = arc.as_ref();
        let arc = asref.unwrap();
        let er = arc.lock().unwrap();
        let next = er.recv();

        serialize_event(next)
    })
    .await
    .unwrap())
}

pub fn init() -> Extension {
    Extension::builder()
        .ops(vec![
            ("op_next_event", op_async(op_next_event)),
        ])
        .build()
}
