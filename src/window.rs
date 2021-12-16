use deno_runtime::deno_core::Extension;
use deno_runtime::deno_core::op_sync;
use serde::Deserialize;
use deno_runtime::deno_core::OpState;
use deno_runtime::deno_core::ResourceId;
use deno_runtime::deno_core::Resource;
use deno_runtime::deno_core::ZeroCopyBuf;
use deno_runtime::deno_core::error::AnyError;
use winit_main::reexports::dpi::PhysicalSize;
use winit_main::reexports::dpi::Size;
use winit_main::reexports::window::Window;
use winit_main::reexports::window::WindowAttributes;
use winit_main::reexports::window::UserAttentionType;
use winit_main::reexports::window::CursorIcon;
use winit_main::reexports::window::Fullscreen;
use winit_main::reexports::window::Icon;
use winit_main::reexports::dpi::Position;
use winit_main::reexports::dpi::PhysicalPosition;
use raw_window_handle::RawWindowHandle;
use raw_window_handle::HasRawWindowHandle;
use deno_runtime::deno_core::serde_json::Value;

use crate::EVENT_LOOP;
use crate::util::hash;
use crate::event_loop::serialize_physical_position;
use crate::event_loop::serialize_physical_size;

pub struct DynHasRawWindowHandle(pub Box<dyn HasRawWindowHandle>);

unsafe impl HasRawWindowHandle for DynHasRawWindowHandle {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0.raw_window_handle()
    }
}

pub struct WindowResource(pub DynHasRawWindowHandle);

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

    Ok((hash(window.id()), state.resource_table.add(WindowResource(DynHasRawWindowHandle(Box::new(window))))))
}

#[derive(Deserialize)]
pub struct SerdePosition {
    x: i32,
    y: i32,
}

impl Into<PhysicalPosition<i32>> for SerdePosition {
    fn into(self) -> PhysicalPosition<i32> {
        PhysicalPosition::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
pub struct SerdeSize {
    width: u32,
    height: u32,
}

impl Into<PhysicalSize<u32>> for SerdeSize {
    fn into(self) -> PhysicalSize<u32> {
        PhysicalSize::new(self.width, self.height)
    }
}

fn get_window(state: &mut OpState, rid: ResourceId) -> Result<&Window, AnyError> {
    let window = state.resource_table.get::<WindowResource>(rid)?;
    let window = window.0.0.as_ref() as *const _ as *const Window;
    let window = unsafe { &*window };
    Ok(window)
}

pub fn op_window_fullscreen(
    state: &mut OpState,
    rid: ResourceId,
    _: ()
) -> Result<bool, AnyError> {
    let window = get_window(state, rid)?;
    let fs = window.fullscreen();
    Ok(match fs {
        Some(_) => true,
        None => false,
    })
}

pub fn op_window_inner_position(
    state: &mut OpState,
    rid: ResourceId,
    _: ()
) -> Result<Value, AnyError> {
    let window = get_window(state, rid)?;
    let position = window.inner_position()?;
    Ok(serialize_physical_position(position))
}

pub fn op_window_inner_size(
    state: &mut OpState,
    rid: ResourceId,
    _: ()
) -> Result<Value, AnyError> {
    let window = get_window(state, rid)?;
    let size = window.inner_size();
    Ok(serialize_physical_size(size))
}

pub fn op_window_outer_position(
    state: &mut OpState,
    rid: ResourceId,
    _: ()
) -> Result<Value, AnyError> {
    let window = get_window(state, rid)?;
    let position = window.outer_position()?;
    Ok(serialize_physical_position(position))
}

pub fn op_window_outer_size(
    state: &mut OpState,
    rid: ResourceId,
    _: ()
) -> Result<Value, AnyError> {
    let window = get_window(state, rid)?;
    let size = window.outer_size();
    Ok(serialize_physical_size(size))
}

pub fn op_window_request_redraw(
    state: &mut OpState,
    rid: ResourceId,
    _: ()
) -> Result<(), AnyError> {
    let window = state.resource_table.get::<WindowResource>(rid)?;
    let window = window.0.0.as_ref() as *const _ as *const Window;
    let window = unsafe { &*window };
    window.request_redraw();
    Ok(())
}

pub fn op_window_request_user_attention(
    state: &mut OpState,
    args: (ResourceId, Option<String>),
    _: (),
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    let ty = match args.1 {
        Some(ty) => match ty.as_str() {
            "critical" => Some(UserAttentionType::Critical),
            "informational" => Some(UserAttentionType::Informational),
            _ => None,
        },
        _ => None,
    };
    window.request_user_attention(ty);
    Ok(())
}

pub fn op_window_scale_factor(
    state: &mut OpState,
    rid: ResourceId,
    _: ()
) -> Result<f64, AnyError> {
    let window = get_window(state, rid)?;
    Ok(window.scale_factor())
}

pub fn op_window_set_always_on_top(
    state: &mut OpState,
    args: (ResourceId, bool),
    _: ()
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    window.set_always_on_top(args.1);
    Ok(())
}

pub fn op_window_set_cursor_grab(
    state: &mut OpState,
    args: (ResourceId, bool),
    _: ()
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    window.set_cursor_grab(args.1)?;
    Ok(())
}

pub fn op_window_set_cursor_icon(
    state: &mut OpState,
    args: (ResourceId, String),
    _: (),
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    let cursor = match args.1.as_str() {
        // TODO
        _ => CursorIcon::Default,
    };
    window.set_cursor_icon(cursor);
    Ok(())
}

pub fn op_window_set_cursor_position(
    state: &mut OpState,
    args: (ResourceId, SerdePosition),
    _: (),
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    window.set_cursor_position(Position::Physical(args.1.into()))?;
    Ok(())
}

pub fn op_window_set_cursor_visible(
    state: &mut OpState,
    args: (ResourceId, bool),
    _: ()
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    window.set_cursor_visible(args.1);
    Ok(())
}

pub fn op_window_set_decorations(
    state: &mut OpState,
    args: (ResourceId, bool),
    _: ()
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    window.set_decorations(args.1);
    Ok(())
}

pub fn op_window_set_fullscreen(
    state: &mut OpState,
    args: (ResourceId, bool),
    _: ()
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    window.set_fullscreen(match args.1 {
        true => Some(Fullscreen::Borderless(None)),
        false => None,
    });
    Ok(())
}

pub fn op_window_set_ime_position(
    state: &mut OpState,
    args: (ResourceId, SerdePosition),
    _: ()
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    window.set_ime_position(Position::Physical(args.1.into()));
    Ok(())
}

pub fn op_window_set_inner_size(
    state: &mut OpState,
    args: (ResourceId, SerdeSize),
    _: ()
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    window.set_inner_size(Size::Physical(args.1.into()));
    Ok(())
}

pub fn op_window_set_max_inner_size(
    state: &mut OpState,
    args: (ResourceId, Option<SerdeSize>),
    _: ()
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    window.set_max_inner_size(match args.1 {
        Some(size) => Some(Size::Physical(size.into())),
        None => None,
    });
    Ok(())
}

pub fn op_window_set_maximized(
    state: &mut OpState,
    args: (ResourceId, bool),
    _: ()
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    window.set_maximized(args.1);
    Ok(())
}

pub fn op_window_set_min_inner_size(
    state: &mut OpState,
    args: (ResourceId, Option<SerdeSize>),
    _: ()
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    window.set_min_inner_size(match args.1 {
        Some(size) => Some(Size::Physical(size.into())),
        None => None,
    });
    Ok(())
}

pub fn op_window_set_minimized(
    state: &mut OpState,
    args: (ResourceId, bool),
    _: ()
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    window.set_minimized(args.1);
    Ok(())
}

pub fn op_window_set_outer_position(
    state: &mut OpState,
    args: (ResourceId, SerdePosition),
    _: ()
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    window.set_outer_position(Position::Physical(args.1.into()));
    Ok(())
}

pub fn op_window_set_resizable(
    state: &mut OpState,
    args: (ResourceId, bool),
    _: ()
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    window.set_resizable(args.1);
    Ok(())
}

pub fn op_window_set_title(
    state: &mut OpState,
    args: (ResourceId, String),
    _: ()
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    window.set_title(&args.1);
    Ok(())
}

pub fn op_window_set_visible(
    state: &mut OpState,
    args: (ResourceId, bool),
    _: ()
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    window.set_visible(args.1);
    Ok(())
}

pub fn op_window_set_window_icon(
    state: &mut OpState,
    args: (ResourceId, u32, u32),
    zc: ZeroCopyBuf,
) -> Result<(), AnyError> {
    let window = get_window(state, args.0)?;
    let icon = Icon::from_rgba(zc.to_vec(), args.1, args.2)?;
    window.set_window_icon(Some(icon));
    Ok(())
}

macro_rules! op {
    ($name:ident) => {
        (stringify!($name), op_sync($name))
    }
}

pub fn init() -> Extension {
    Extension::builder()
        .ops(vec![
            op!(op_create_window),
            op!(op_window_fullscreen),
            op!(op_window_inner_position),
            op!(op_window_inner_size),
            op!(op_window_outer_position),
            op!(op_window_outer_size),
            op!(op_window_request_redraw),
            op!(op_window_request_user_attention),
            op!(op_window_scale_factor),
            op!(op_window_set_always_on_top),
            op!(op_window_set_cursor_grab),
            op!(op_window_set_cursor_icon),
            op!(op_window_set_cursor_position),
            op!(op_window_set_cursor_visible),
            op!(op_window_set_decorations),
            op!(op_window_set_fullscreen),
            op!(op_window_set_ime_position),
            op!(op_window_set_inner_size),
            op!(op_window_set_max_inner_size),
            op!(op_window_set_maximized),
            op!(op_window_set_min_inner_size),
            op!(op_window_set_minimized),
            op!(op_window_set_outer_position),
            op!(op_window_set_resizable),
            op!(op_window_set_title),
            op!(op_window_set_visible),
            op!(op_window_set_window_icon),
        ])
        .build()
}
