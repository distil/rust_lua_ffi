extern crate libc;
#[macro_use]
extern crate lua_marshalling;
extern crate c_marshalling;

// ffi *must* be pub to expose the C API
pub mod ffi;
