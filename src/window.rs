use deno_runtime::deno_core::Extension;
use deno_runtime::deno_core::op_sync;
use deno_runtime::deno_core::error::bad_resource_id;
use serde::Deserialize;
use deno_runtime::deno_core::OpState;
use deno_runtime::deno_core::ResourceId;
use deno_runtime::deno_core::Resource;
use deno_runtime::deno_core::error::AnyError;
use winit_main::reexports::dpi::PhysicalSize;
use winit_main::reexports::dpi::Size;
use winit_main::reexports::window::Window;
use winit_main::reexports::window::WindowAttributes;

use crate::EVENT_LOOP;
use crate::util::hash;

pub struct WindowResource(pub Window);

impl Resource for WindowResource {
    fn name(&self) -> std::borrow::Cow<str> {
        "Window".into()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWindowArgs {
    title: Option<String>,
    resizable: Option<bool>,
    decorations: Option<bool>,
    maximized: Option<bool>,
    visible: Option<bool>,
    transparent: Option<bool>,
    always_on_top: Option<bool>,
    width: Option<u32>,
    height: Option<u32>,
    min_width: Option<u32>,
    min_height: Option<u32>,
    max_width: Option<u32>,
    max_height: Option<u32>,
}

pub fn op_create_window(
    state: &mut OpState,
    args: CreateWindowArgs,
    _: ()
) -> Result<(u32, ResourceId), AnyError> {
    let ev = EVENT_LOOP.lock().unwrap();
    let ev = ev.as_ref().unwrap().lock().unwrap();

    let mut attribs = WindowAttributes::default();

    if let Some(title) = args.title {
        attribs.title = title;
    }

    if let Some(resizable) = args.resizable {
        attribs.resizable = resizable;
    }

    if let Some(decorations) = args.decorations {
        attribs.decorations = decorations;
    }

    if let Some(maximized) = args.maximized {
        attribs.maximized = maximized;
    }

    if let Some(visible) = args.visible {
        attribs.visible = visible;
    }

    if let Some(transparent) = args.transparent {
        attribs.transparent = transparent;
    }

    if let Some(always_on_top) = args.always_on_top {
        attribs.always_on_top = always_on_top;
    }

    if args.width.is_some() || args.height.is_some() {
        let width = args.width.unwrap_or(800);
        let height = args.height.unwrap_or(600);
        let size = PhysicalSize::new(width, height);
        attribs.inner_size = Some(Size::Physical(size));
    }

    if args.min_width.is_some() || args.min_height.is_some() {
        let min_width = args.min_width.unwrap_or(0);
        let min_height = args.min_height.unwrap_or(0);
        let size = PhysicalSize::new(min_width, min_height);
        attribs.min_inner_size = Some(Size::Physical(size));
    }

    if args.max_width.is_some() || args.max_height.is_some() {
        let max_width = args.max_width.unwrap_or(0);
        let max_height = args.max_height.unwrap_or(0);
        let size = PhysicalSize::new(max_width, max_height);
        attribs.max_inner_size = Some(Size::Physical(size));
    }

    let window = ev.create_window(attribs)?;

    Ok((hash(window.id()), state.resource_table.add(WindowResource(window))))
}

pub fn op_window_set_visible(
    state: &mut OpState,
    args: (ResourceId, bool),
    _: ()
) -> Result<(), AnyError> {
    let window = state.resource_table.get::<WindowResource>(args.0);

    if let Ok(window) = window {
        window.0.set_visible(args.1);
        Ok(())
    } else {
        Err(bad_resource_id())
    }
}

pub fn op_window_request_redraw(
    state: &mut OpState,
    rid: ResourceId,
    _: ()
) -> Result<(), AnyError> {
    let window = state.resource_table.get::<WindowResource>(rid);

    if let Ok(window) = window {
        window.0.request_redraw();
        Ok(())
    } else {
        Err(bad_resource_id())
    }
}

pub fn init() -> Extension {
    Extension::builder()
        .ops(vec![
            ("op_create_window", op_sync(op_create_window)),
            ("op_window_set_visible", op_sync(op_window_set_visible)),
            ("op_window_request_redraw", op_sync(op_window_request_redraw)),
        ])
        .build()
}
