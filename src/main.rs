#[macro_use]
extern crate lazy_static;

pub mod event_loop;
pub mod surface;
pub mod util;
pub mod window;

use winit_main::Blocker;
use winit_main::EventLoopHandle;
use winit_main::EventReceiver;

use deno_runtime::deno_core::error::AnyError;
use deno_runtime::deno_core::op_async;
use deno_runtime::deno_core::op_sync;
use deno_runtime::deno_core::FsModuleLoader;
use deno_runtime::deno_broadcast_channel::InMemoryBroadcastChannel;
use deno_runtime::deno_web::BlobStore;
use deno_runtime::permissions::Permissions;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use deno_runtime::BootstrapOptions;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use event_loop::op_drop_blocker;
use event_loop::op_next_event;

use window::op_create_window;
use window::op_window_set_visible;
use window::op_window_request_redraw;

use surface::op_webgpu_configure_surface;
use surface::op_webgpu_create_surface;
use surface::op_webgpu_surface_get_preferred_format;
use surface::op_webgpu_surface_get_current_texture;
use surface::op_webgpu_surface_present;

thread_local! {
    pub static EVENTLOOP: RefCell<Option<EventLoopHandle>> = RefCell::new(None);
}

lazy_static! {
    pub static ref EVENT_RECEIVER: Mutex<Option<Arc<Mutex<EventReceiver>>>> = Mutex::new(None);
    pub static ref BLOCKER: Mutex<Option<Blocker>> = Mutex::new(None);
}

fn get_error_class_name(e: &AnyError) -> &'static str {
    deno_runtime::errors::get_error_class_name(e).unwrap_or("Error")
}

#[winit_main::main]
fn main(event_loop: EventLoopHandle, events: EventReceiver) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let arc_events = Arc::new(Mutex::new(events));

    runtime.block_on(async {
        EVENTLOOP.with(|ev| {
            let mut ev = ev.borrow_mut();
            *ev = Some(event_loop);
        });

        *EVENT_RECEIVER.lock().unwrap() = Some(arc_events);

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
                location: None,
                no_color: false,
                runtime_version: "0.0.1".to_string(),
                ts_version: "4.4.3".to_string(),
                unstable: true,
            },
            extensions: vec![],
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
        let main_module = deno_runtime::deno_core::resolve_path(&js_path.to_string_lossy()).unwrap();
        let permissions = Permissions::allow_all();

        let mut worker =
            MainWorker::bootstrap_from_options(main_module.clone(), permissions, options);

        worker
            .js_runtime
            .register_op("op_next_event", op_async(op_next_event));
        worker
            .js_runtime
            .register_op("op_drop_blocker", op_sync(op_drop_blocker));
        worker
            .js_runtime
            .register_op("op_create_window", op_sync(op_create_window));
        worker
            .js_runtime
            .register_op("op_window_set_visible", op_sync(op_window_set_visible));
        worker
            .js_runtime
            .register_op("op_window_request_redraw", op_sync(op_window_request_redraw));
        worker.js_runtime.register_op(
            "op_webgpu_create_surface",
            op_sync(op_webgpu_create_surface),
        );
        worker.js_runtime.register_op(
            "op_webgpu_configure_surface",
            op_sync(op_webgpu_configure_surface),
        );
        worker.js_runtime.register_op(
            "op_webgpu_surface_get_preferred_format",
            op_sync(op_webgpu_surface_get_preferred_format),
        );
        worker.js_runtime.register_op(
            "op_webgpu_surface_get_current_texture",
            op_sync(op_webgpu_surface_get_current_texture),
        );
        worker.js_runtime.register_op(
            "op_webgpu_surface_present",
            op_sync(op_webgpu_surface_present),
        );

        worker.js_runtime.sync_ops_cache();

        worker
            .js_runtime
            .execute_script("deno_desktop:core.js", include_str!("core.js"))
            .unwrap();

        worker.execute_main_module(&main_module).await.unwrap();
        worker.run_event_loop(false).await.unwrap();
    });
}
