mod changes;
mod hooks;
mod loader;
mod log;
mod mods;
mod win;

use std::ffi::c_void;

type Bool = i32;
type Dword = u32;
type Hinstance = *mut c_void;
type Hresult = i32;
type Lpvoid = *mut c_void;
type RefGuid = *const win::Guid;

const TRUE: Bool = 1;

#[no_mangle]
pub extern "system" fn DllMain(module: Hinstance, _reason: Dword, _reserved: Lpvoid) -> Bool {
    loader::initialize_once(module);
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn DirectInput8Create(
    instance: Hinstance,
    version: Dword,
    riid: RefGuid,
    out: *mut Lpvoid,
    outer: *mut c_void,
) -> Hresult {
    let function = win::direct_input8_create();
    function(instance, version, riid, out, outer)
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> Hresult {
    let function = win::dll_can_unload_now();
    function()
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    class_id: RefGuid,
    interface_id: RefGuid,
    out: *mut Lpvoid,
) -> Hresult {
    let function = win::dll_get_class_object();
    function(class_id, interface_id, out)
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> Hresult {
    let function = win::dll_register_server();
    function()
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> Hresult {
    let function = win::dll_unregister_server();
    function()
}
