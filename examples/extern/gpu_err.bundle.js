function patchPushError(proto, throwError) {
    const _pushError = proto.pushError;
    proto.pushError = function(error) {
        if (error !== null && error.type === "validation") {
            const err = new Error(error.value ?? "unknown");
            err.name = "WebGPUValidationError";
            if (throwError) throw err;
            else console.error(err);
        }
        _pushError.call(proto, error);
    };
}
function enableValidationErrors(device, throwError = false) {
    const _InnerDevice = Object.getOwnPropertySymbols(device).find((e)=>e.description === "[[device]]"
    );
    const innerDevice = device[_InnerDevice];
    patchPushError(innerDevice, throwError);
}
export { enableValidationErrors as enableValidationErrors };
