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
Calls involving strings should always be made inside a Lua `pcall`.
However, passing a Lua string to Rust as a `String` will use `to_string_lossy` as the string is copied.
* Returning a string from Rust **may** **fail** as well with an `error` due to strings containing the zero-byte.
Calls involving strings should always be made inside a Lua `pcall`.
* A Rust `panic` will cause an `error` in Lua.
Calls that might `panic` should always be made inside a Lua `pcall`.

### Known Issues
* `Vec<Option<T>>` have been disabled.
Lua arrays generally do not handle `null` values well.
See [https://www.lua.org/pil/19.1.html](https://www.lua.org/pil/19.1.html) for more information.
