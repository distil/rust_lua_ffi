# Lua to Rust FFI code generation
## Motivating example
Rust
```Rust
#[derive(LuaMarshalling)]
pub struct A {
    string: String,
    integer: i32,
}

pub mod extern_ffi {
    pub fn make_a(string: &str, integer: i32) -> A {
        A {
            string: string.to_owned(),
            integer,
        }
    }

    pub fn describe(a: A) -> String {
        format!("A: {:?}", a)
    }
}
```

Lua
```Lua
local example = require('rust-example')

local a = example.make_a("Test string", 42)
print("a", a.string, a.integer)

print("describe", example.describe(a))
```
## Implementation details
### Types
* Supported Rust types include primitives, `Vec`, `Option`, `String` and custom `struct` with `derive(LuaMarshalling)` and any combination of those.
`&str` is supported only as an argument but is faster than `String`. `&[]` is supported only for primitive types.
`Result` is supported only as a return argument.
* `Option`s `None` is `nil` in Lua.
* Only `&str` and `&[]` of primitive types are passed as references to Rust, all other types are copied.
* A Rust `struct` is converted to a Lua `table`, but can still be used as an argument.
For this to work, the Lua table also keeps a reference to the native object pointer.
* The native object pointer is garbage collected by calling back to Rust.
To keep the native pointer and the table consistent, the `table` is immutable.

### `panic` and `error`
* Passing a Lua string to Rust as `&str` or `String` **may** **fail** with an `error` due to UTF-8 requirements.
However, passing a Lua string to Rust as a `&[u8]` or `Vec<u8>` will not.
* Returning a string from Rust **may** **fail** as well with an `error` due to strings containing the zero-byte.
* A Rust `panic` will cause an `error` in Lua.

### Known Issues
* `Vec<Option<T>>` have been disabled.
Lua arrays generally do not handle `null` values well.
See [https://www.lua.org/pil/19.1.html](https://www.lua.org/pil/19.1.html) for more information.
* `struct` typenames must be unique. Separate modules are not enough.
* Identifiers can not be Lua or C reserved keywords. For example, a variable cannot be called `short`.
* The `__` prefix is reserved for hidden identifiers and should not be used as field names or function arguments.

## Setup
### Configuration
```
cargo new example_setup
```
* In `example_setup` create the file `src/build.rs` with the following content

```Rust
extern crate generator;

use std::env;

fn main() {
    let rust_output = ::std::path::Path::new(&env::var("OUT_DIR").unwrap()).join("ffi.rs");

    let output = generator::generate(
        &env::current_dir().unwrap().as_path().join("src/lib.rs"), "example_setup");

    use std::io::Write;
    std::fs::File::create(rust_output.clone()).unwrap().write_all(output.as_bytes()).unwrap();

    assert!(rust_output.exists());
}
```

**Note** the `library_name` parameter to `generator::generator` must be equal to the library name of the crate.

Add the following to the `Cargo.toml` under `[package]`
```
build = "src/build.rs"
```

Under `[dependencies]` add the following
```
libc = "0.2.20"
c-marshalling = { git = "https://github.com/distil/rust_lua_ffi" }
lua-marshalling = { git = "https://github.com/distil/rust_lua_ffi" }
```

Add the following section to the `Cargo.toml` as well
```
[build-dependencies]
generator = { git = "https://github.com/distil/rust_lua_ffi" }

[lib]
crate-type = ["cdylib"]
```

In `src/lib.rs` add the following
```Rust
extern crate libc;
#[macro_use]
extern crate lua_marshalling;
extern crate c_marshalling;

pub mod extern_ffi {
    pub fn hello_world() -> String {
        "Hello World!".to_owned()
    }
}

include!(concat!(env!("OUT_DIR"), "/ffi.rs"));
```

### Building
After the library has been built, the Lua interface code can be generated using the following command
```
LD_LIBRARY_PATH=..path-to-example_setup/target/debug/ \
    luajit ..path-to-rust_lua_ffi/lua/bootstrap.lua example_setup > api.lua
```

### Usage
To use the `api.lua` file generated in the *Building* step, create a Lua file called `example.lua` in the same directory as the Lua interface code containing
```Lua
local api = require('api')

print(api.hello_world())
```

Execute the file using the following command
```Bash
LD_LIBRARY_PATH=..path-to-example_setup/target/debug/ luajit example.lua
```
