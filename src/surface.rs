#![allow(unused_macros)]

use std::borrow::Cow;
use std::rc::Rc;

use deno_runtime::deno_core::Extension;
use deno_runtime::deno_core::op_sync;
use deno_runtime::deno_core::ResourceTable;
use deno_runtime::deno_core::error::bad_resource_id;
use deno_runtime::deno_core::error::AnyError;
use deno_runtime::deno_core::OpState;
use deno_runtime::deno_core::anyhow::anyhow;
use deno_runtime::deno_core::Resource;
use deno_runtime::deno_core::ResourceId;
use deno_runtime::deno_webgpu::texture::GpuTextureFormat;
use serde::Deserialize;
use deno_runtime::deno_webgpu::wgpu_core;
use deno_runtime::deno_webgpu::wgpu_types;
use deno_runtime::deno_webgpu::Instance;
use deno_runtime::deno_webgpu::WebGpuDevice;
use deno_runtime::deno_webgpu::WebGpuAdapter;
use deno_runtime::deno_webgpu::texture::WebGpuTexture;

use crate::window::WindowResource;

fn get_resource<R: Resource>(state: &mut ResourceTable, rid: ResourceId) -> Result<Rc<R>, AnyError> {
    let res = state.get::<R>(rid);

    if res.is_err() {
        Err(bad_resource_id())
    } else {
        Ok(res.unwrap())
    }
}

macro_rules! gfx_select {
    ($id:expr => $global:ident.$method:ident( $($param:expr),* )) => {
        match $id.backend() {
            #[cfg(not(target_os = "macos"))]
            wgpu_types::Backend::Vulkan => $global.$method::<wgpu_core::api::Vulkan>( $($param),* ),
            #[cfg(target_os = "macos")]
            wgpu_types::Backend::Metal => $global.$method::<wgpu_core::api::Metal>( $($param),* ),
            #[cfg(windows)]
            wgpu_types::Backend::Dx12 => $global.$method::<wgpu_core::api::Dx12>( $($param),* ),
            #[cfg(all(unix, not(target_os = "macos")))]
            wgpu_types::Backend::Gl => $global.$method::<wgpu_core::api::Gles>( $($param),+ ),
            other => panic!("Unexpected backend {:?}", other),
        }
    };
}

struct WebGpuSurface(wgpu_core::id::SurfaceId);
impl Resource for WebGpuSurface {
    fn name(&self) -> Cow<str> {
        "webGPUSurface".into()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSurfaceArgs {
    window_rid: ResourceId,
}

pub fn op_webgpu_create_surface(
    state: &mut OpState,
    args: CreateSurfaceArgs,
    _: (),
) -> Result<ResourceId, AnyError> {
    let winres = get_resource::<WindowResource>(&mut state.resource_table, args.window_rid)?;
    let instance = state.borrow::<Instance>();
    let surface_id = instance.instance_create_surface(&winres.0, std::marker::PhantomData);
    Ok(state.resource_table.add(WebGpuSurface(surface_id)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigureSurfaceArgs {
    device_rid: ResourceId,
    surface_rid: ResourceId,
    format: GpuTextureFormat,
    usage: u32,
    width: u32,
    height: u32,
}

pub fn op_webgpu_configure_surface(
    state: &mut OpState,
    args: ConfigureSurfaceArgs,
    _: (),
) -> Result<(), AnyError> {
    let surface = state.resource_table.get::<WebGpuSurface>(args.surface_rid)?;
    let device = state.resource_table.get::<WebGpuDevice>(args.device_rid)?;
    let instance = state.borrow::<Instance>();
    
    let config = wgpu_types::SurfaceConfiguration {
        usage: wgpu_types::TextureUsages::from_bits(args.usage).unwrap(),
        format: args.format.try_into().unwrap(),
        width: args.width,
        height: args.height,
        present_mode: wgpu_types::PresentMode::Fifo,
    };

    match gfx_select!(device.0 => instance.surface_configure(
        surface.0,
        device.0,
        &config
    )) {
        Some(err) => Err(err.into()),
        None => Ok(()),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfacePreferredFormatArgs {
    adapter_rid: ResourceId,
    surface_rid: ResourceId,
}

pub fn op_webgpu_surface_get_preferred_format(
    state: &mut OpState,
    args: SurfacePreferredFormatArgs,
    _: (),
) -> Result<wgpu_types::TextureFormat, AnyError> {
    let surface = state.resource_table.get::<WebGpuSurface>(args.surface_rid)?;
    let adapter = state.resource_table.get::<WebGpuAdapter>(args.adapter_rid)?;
    let instance = state.borrow::<Instance>();
    let fmt = gfx_select!(adapter.0 => instance.surface_get_preferred_format(surface.0, adapter.0))?;
    Ok(fmt)
}

pub fn op_webgpu_surface_get_current_texture(
    state: &mut OpState,
    args: SurfacePreferredFormatArgs,
    _: (),
) -> Result<ResourceId, AnyError> {
    let surface = state.resource_table.get::<WebGpuSurface>(args.surface_rid)?;
    let adapter = state.resource_table.get::<WebGpuAdapter>(args.adapter_rid)?;
    let instance = state.borrow::<Instance>();

    let current_texture = gfx_select!(
        adapter.0 => instance.surface_get_current_texture(surface.0, std::marker::PhantomData)
    )?;
    if let Some(texture) = current_texture.texture_id {
        Ok(state.resource_table.add(WebGpuTexture(texture)))
    } else {
        Err(anyhow!("No texture, status: {:?}", current_texture.status))
    }
}

pub fn op_webgpu_surface_present(
    state: &mut OpState,
    args: SurfacePreferredFormatArgs,
    _: (),
) -> Result<String, AnyError> {
    let surface = state.resource_table.get::<WebGpuSurface>(args.surface_rid)?;
    let adapter = state.resource_table.get::<WebGpuAdapter>(args.adapter_rid)?;
    let instance = state.borrow::<Instance>();

    let status = gfx_select!(adapter.0 => instance.surface_present(surface.0))?;
    Ok(String::from(match status {
        wgpu_types::SurfaceStatus::Good => "good",
        wgpu_types::SurfaceStatus::Suboptimal => "suboptimal",
        wgpu_types::SurfaceStatus::Timeout => "timeout",
        wgpu_types::SurfaceStatus::Outdated => "outdated",
        wgpu_types::SurfaceStatus::Lost => "lost",
    }))
}

pub fn op_webgpu_surface_drop(
    state: &mut OpState,
    rid: ResourceId,
    _: (),
) -> Result<(), AnyError> {
    let surface = state.resource_table.get::<WebGpuSurface>(rid)?;
    let instance = state.borrow::<Instance>();
    instance.surface_drop(surface.0);
    Ok(())
}

pub fn init() -> Extension {
    Extension::builder()
        .ops(vec![
            ("op_webgpu_create_surface", op_sync(op_webgpu_create_surface)),
            ("op_webgpu_configure_surface", op_sync(op_webgpu_configure_surface)),
            ("op_webgpu_surface_get_preferred_format", op_sync(op_webgpu_surface_get_preferred_format)),
            ("op_webgpu_surface_get_current_texture", op_sync(op_webgpu_surface_get_current_texture)),
            ("op_webgpu_surface_present", op_sync(op_webgpu_surface_present)),
            ("op_webgpu_surface_drop", op_sync(op_webgpu_surface_drop)),
        ])
        .build()
}
