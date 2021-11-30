use deno_runtime::deno_core::OpState;
use deno_runtime::deno_core::error::AnyError;
use deno_runtime::deno_core::op_sync;
use deno_runtime::deno_core::Extension;
use serde::Serialize;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApplySourceMap {
    file_name: String,
    line_number: i32,
    column_number: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppliedSourceMap {
    file_name: String,
    line_number: u32,
    column_number: u32,
}

fn op_apply_source_map(
    _state: &mut OpState,
    args: ApplySourceMap,
    _: (),
) -> Result<AppliedSourceMap, AnyError> {
    Ok(AppliedSourceMap {
        file_name: args.file_name,
        line_number: args.line_number as u32,
        column_number: args.column_number as u32
    })
}

fn op_format_diagnostic(
    _state: &mut OpState,
    _: (),
    _: (),
) -> Result<String, AnyError> {
    Ok(String::from(""))
}

fn op_format_file_name(
    _state: &mut OpState,
    file_name: String,
    _: (),
) -> Result<String, AnyError> {
    Ok(file_name)
}

pub fn init() -> Extension {
    Extension::builder()
        .ops(vec![
            ("op_apply_source_map", op_sync(op_apply_source_map)),
            ("op_format_diagnostic", op_sync(op_format_diagnostic)),
            ("op_format_file_name", op_sync(op_format_file_name)),
        ])
        .build()
}
