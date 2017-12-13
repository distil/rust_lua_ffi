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
* `Option`s `None` is `nil` in Lua.
* Only `&str` and `&[]` of primitive types are passed as references to Rust, all other types are copied.
* A Rust `struct` is converted to a Lua `table`, but can still be used as an argument.
For this to work, the Lua table also keeps a reference to the native object pointer.
* The native object pointer is garbage collected by calling back to Rust.
To keep the native pointer and the table consistent, the `table` is immutable.

### `panic` and `error`
* Passing a Lua string to Rust as `&str` **may** **fail** with an `error` due to UTF-8 requirements.
However, passing a Lua string to Rust as a `String` will use `to_string_lossy` as the string is copied.
* Returning a string from Rust **may** **fail** as well with an `error` due to strings containing the zero-byte.
* A Rust `panic` will cause an `error` in Lua.

### Known Issues
* `Vec<Option<T>>` have been disabled.
Lua arrays generally do not handle `null` values well.
See [https://www.lua.org/pil/19.1.html](https://www.lua.org/pil/19.1.html) for more information.

## Setup
### Configuration
```
cargo new example_setup
```
* In `example_setup` create the file `src/build.rs` with the following content

```Rust
use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let env_out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&env_out_dir);
    let rust_output = out_dir.join("ffi.rs");

    let output = Command::new("lua-c-ffi-marshalling")
        .args(&["--input", env::current_dir().unwrap().as_path().join("src/lib.rs").to_str().unwrap()])
        .args(&["--output", rust_output.to_str().unwrap()])
        .args(&["--library-name", "example_setup"])
        .output()
        .expect("lua-c-ffi-marshalling failed");
    print!("--- stdout\n{}\n", String::from_utf8_lossy(&output.stdout).as_ref());
    print!("--- stderr\n{}\n", String::from_utf8_lossy(&output.stderr).as_ref());
    assert!(output.status.success());
    assert!(rust_output.exists());
}
```

**Note** the `--library-name` parameter must be equal to the library name of the crate.

Add the following to the `Cargo.toml` under `[package]`
```
build = "src/build.rs"
```

Under `[dependencies]` add the following
```
libc = "0.2.20"
c-marshalling = { path = "..path-to/c-marshalling" }
lua-marshalling = { path = "..path-to/lua-marshalling" }
```

Add the following section to the `Cargo.toml` as well
```
[lib]
crate-type = ["dylib"]
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
To build the `example_setup` crate, cargo must be able to execute the code generator. In order to do so, set the `PATH` as following:
```Bash
PATH=${PATH}:..path-to/lua-c-ffi-marshalling/target/release/ cargo build
```

After the library has been built, the Lua interface code can be generated using the following command
```
LD_LIBRARY_PATH=..path-to-example_setup/target/debug/ luajit ..path-to-rust_lua_ffi/lua/bootstrap.lua example_setup > api.lua
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