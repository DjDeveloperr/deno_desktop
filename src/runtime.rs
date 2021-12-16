use deno_runtime::deno_core::FsModuleLoader;
use deno_runtime::deno_core::ModuleSpecifier;
use deno_runtime::deno_broadcast_channel::InMemoryBroadcastChannel;
use deno_runtime::deno_web::BlobStore;
use deno_runtime::permissions::Permissions;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use deno_runtime::deno_core::error::AnyError;
use deno_runtime::BootstrapOptions;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use crate::event_loop;
use crate::surface;
use crate::window;
use crate::extra;

fn get_error_class_name(e: &AnyError) -> &'static str {
    deno_runtime::errors::get_error_class_name(e).unwrap_or("Error")
}

pub async fn start() -> Result<(), AnyError> {
    let module_loader = Rc::new(FsModuleLoader);
    let create_web_worker_cb = Arc::new(|_| {
        todo!("Web workers are not supported");
    });

    let options = WorkerOptions {
        bootstrap: BootstrapOptions {
            apply_source_maps: false,
            args: vec![],
            cpu_count: 1,
            debug_flag: false,
            enable_testing_features: false,
            location: Some(ModuleSpecifier::parse("https://desktop.deno.land").unwrap()),
            no_color: false,
            runtime_version: "0.0.1".to_string(),
            ts_version: "4.4.3".to_string(),
            unstable: true,
        },
        extensions: vec![
            event_loop::init(),
            window::init(),
            // Will be integrated into deno_webgpu later.
            // https://github.com/gfx-rs/wgpu/pull/2279
            surface::init(),
            // Some ops deno_runtime depends on
            // But aren't there (implemented in CLI)
            // https://github.com/denoland/deno/issues/12918
            extra::init(),
        ],
        unsafely_ignore_certificate_errors: None,
        root_cert_store: None,
        user_agent: "deno_desktop".to_string(),
        seed: None,
        js_error_create_fn: None,
        create_web_worker_cb,
        maybe_inspector_server: None,
        should_break_on_first_statement: false,
        module_loader,
        get_error_class_fn: Some(&get_error_class_name),
        origin_storage_dir: None,
        blob_store: BlobStore::default(),
        broadcast_channel: InMemoryBroadcastChannel::default(),
        shared_array_buffer_store: None,
        compiled_wasm_module_store: None,
    };

    let path = std::env::args().nth(1).unwrap_or(String::from("test.js"));
    let js_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let main_module = deno_runtime::deno_core::resolve_path(&js_path.to_string_lossy())?;
    let permissions = Permissions::allow_all();

    let mut worker = MainWorker::bootstrap_from_options(main_module.clone(), permissions, options);

    worker
        .js_runtime
        .execute_script("deno_desktop:core.js", include_str!("core.js"))?;

    worker.execute_main_module(&main_module).await?;
    worker.run_event_loop(false).await?;

    Ok(())
}
