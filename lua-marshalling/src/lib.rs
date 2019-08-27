#![allow(unused_imports)]

#[macro_use]
extern crate derive_lua_marshalling;
#[macro_use]
extern crate lazy_static;

// procedural crates do not allow exporting code themselves, so re-export the crate and
// implement the library here.
pub use derive_lua_marshalling::*;

fn is_valid_type_prefix(string: &str) -> bool {
    string.is_empty() || {
        let mut bytes = string.as_bytes().iter();
        (match bytes.next().unwrap() {
            #[cfg_attr(rustfmt, rustfmt::skip)]
            | b'a'..=b'z'
            | b'A'..=b'Z'
            | b'_'
            => true,
            _ => false,
        }) && bytes.all(|byte| match *byte {
            #[cfg_attr(rustfmt, rustfmt::skip)]
            | b'a'..=b'z'
            | b'A'..=b'Z'
            | b'0'..=b'9'
            | b'_'
            => true,
            _ => false,
        })
    }
}

#[test]
fn test_is_valid_type_prefix() {
    assert!(is_valid_type_prefix(""));
    assert!(is_valid_type_prefix("__abc__"));
    assert!(is_valid_type_prefix("__abc_3_"));
    assert!(is_valid_type_prefix("F__abc_3_"));
    assert!(is_valid_type_prefix("F"));
    assert!(!is_valid_type_prefix("4F__abc_3_"));
    assert!(!is_valid_type_prefix("F_-abc_3_"));
}

fn prefixed(string: &str) -> String {
    lazy_static! {
        static ref PACKAGE_PREFIX: String = {
            std::env::var("RUST_LUA_FFI_TYPE_PREFIX")
                .map(|name| {
                    assert!(
                        is_valid_type_prefix(&name),
                        format!(
                            "Invalid value ({:#?}) supplied for \
                             RUST_LUA_FFI_TYPE_PREFIX. Value must contain only \
                             ASCII alpha-numeric characters or underscores and \
                             cannot start with a number",
                            name,
                        )
                    );
                    format!("{}__", name)
                })
                .unwrap_or_default()
        };
    }

    format!("{}{}", *PACKAGE_PREFIX, string)
}

#[derive(Debug, Clone)]
pub struct TypeDescription {
    pub typeid: std::any::TypeId,
    pub dependencies: std::collections::HashSet<std::any::TypeId>,
    pub typedeclaration: fn() -> String,
    pub metatype: fn() -> String,
}

pub trait Type {
    // Defaults to false since all derived types outside of this module _won't_
    // be primitive.
    const IS_PRIMITIVE: bool = false;

    fn typename() -> String;
    fn c_typename() -> String {
        Self::typename()
    }
    fn typedeclaration() -> String {
        "".to_owned()
    }
    fn metatype() -> String {
        if Self::IS_PRIMITIVE {
            primitive_type_metatype::<Self>()
        } else {
            ptr_type_metatype::<Self>()
        }
    }
    fn dependencies() -> Dependencies {
        Dependencies::new()
    }
    fn c_function_argument() -> String;
    fn c_mut_function_argument() -> String;
    fn prefixed_typename() -> String {
        if Self::IS_PRIMITIVE {
            Self::typename()
        } else {
            prefixed(&Self::typename())
        }
    }
    fn prefixed_c_typename() -> String {
        if Self::IS_PRIMITIVE {
            Self::c_typename()
        } else {
            prefixed(&Self::c_typename())
        }
    }
}

pub trait FromRawConversion: Type {
    fn function() -> String;
    fn gc() -> bool;
}

pub trait IntoRawConversion: Type {
    fn function() -> String;
    fn create_pointer() -> String;
    fn create_array() -> String;
}

pub type Dependencies = std::collections::HashMap<std::any::TypeId, TypeDescription>;

pub fn make_dependencies<T: Type + 'static>() -> Dependencies {
    let typeid = std::any::TypeId::of::<T>();
    let mut dependencies = T::dependencies();
    let type_dependencies = dependencies.keys().cloned().collect();
    dependencies.insert(
        typeid,
        TypeDescription {
            typeid: std::any::TypeId::of::<T>(),
            dependencies: type_dependencies,
            typedeclaration: T::typedeclaration,
            metatype: T::metatype,
        },
    );
    dependencies
}

pub fn dependency_sorted_type_descriptions(dependencies: &Dependencies) -> Vec<&TypeDescription> {
    let mut remaining: std::collections::HashSet<_> = dependencies.keys().cloned().collect();
    let mut sorted_dependencies = Vec::new();
    while !remaining.is_empty() {
        let typ = {
            let (typ, dependencies) = remaining
                .iter()
                .map(|typ| (typ, dependencies.get(typ).unwrap()))
                .find(|&(_, dependencies)| dependencies.dependencies.is_disjoint(&remaining))
                .unwrap();
            sorted_dependencies.push(dependencies);
            typ.clone()
        };
        assert!(remaining.remove(&typ));
    }
    sorted_dependencies
}

pub fn ptr_type_metatype<T: Type + ?Sized>() -> String {
    format!(
        r#"
local __typename_{self_typename} = ffi.metatype("{c_typename}", {{}})
local __const_c_typename_{self_typename} = ffi.typeof("const {c_typename}[?]")
local __c_function_argument_{self_typename} = ffi.typeof("{c_function_argument}[?]")
local __c_mut_function_argument_{self_typename} = ffi.typeof("{c_mut_function_argument}[?]")
"#,
        self_typename = T::typename(),
        c_typename = T::prefixed_c_typename(),
        c_function_argument = T::c_function_argument(),
        c_mut_function_argument = T::c_mut_function_argument()
    )
}

pub fn primitive_type_metatype<T: Type + ?Sized>() -> String {
    format!(
        r#"
local __const_c_typename_{self_typename} = ffi.typeof("const {c_typename}[?]")
local __c_function_argument_{self_typename} = ffi.typeof("{c_function_argument}[?]")
local __c_mut_function_argument_{self_typename} = ffi.typeof("{c_mut_function_argument}[?]")
"#,
        self_typename = T::typename(),
        c_typename = T::c_typename(),
        c_function_argument = T::c_function_argument(),
        c_mut_function_argument = T::c_mut_function_argument()
    )
}

pub fn ptr_type_create_pointer<T: IntoRawConversion>() -> String {
    T::function()
}

pub fn ptr_type_create_array<T: IntoRawConversion>() -> String {
    format!(
        r#"function(value)
    local result = {{}}
    local f = {function}
    for i, value in pairs(value) do
        local tmp = f(value)
        result[i] = tmp[0]
    end
    return __const_c_typename_{typename}(#result, result)
end"#,
        function = T::function(),
        typename = <T as Type>::typename()
    )
}

pub fn immediate_type_create_array<T: IntoRawConversion>() -> String {
    format!(
        r#"function(value)
    local result = {{}}
    local f = {function}
    for i, value in pairs(value) do
        result[i] = f(value)
    end
    return __const_c_typename_{typename}(#result, result)
end"#,
        function = T::function(),
        typename = <T as Type>::typename()
    )
}

fn primitive_type_create_pointer<T: IntoRawConversion>() -> String {
    format!(
        r#"function(value)
    return __const_c_typename_{typename}(1, {{ value }})
end"#,
        typename = <T as Type>::typename()
    )
}

fn primitive_type_create_array<T: IntoRawConversion>() -> String {
    format!(
        r#"function(value)
    return __const_c_typename_{typename}(#value, value)
end"#,
        typename = T::typename()
    )
}

impl<T: Type + 'static> Type for Option<T> {
    const IS_PRIMITIVE: bool = false;
    fn typename() -> String {
        format!("Option_{}", T::typename())
    }
    fn typedeclaration() -> String {
        format!(
            r#"typedef struct {{
    const {c_typename} *ptr;
}} {self_typename};"#,
            c_typename = <T as Type>::prefixed_c_typename(),
            self_typename = Self::prefixed_typename()
        )
    }
    fn dependencies() -> Dependencies {
        make_dependencies::<T>()
    }
    fn c_function_argument() -> String {
        format!("const {}*", Self::prefixed_c_typename())
    }
    fn c_mut_function_argument() -> String {
        format!("{}*", Self::prefixed_c_typename())
    }
}

impl<T: FromRawConversion + 'static> FromRawConversion for Option<T> {
    fn function() -> String {
        format!(
            r#"function(value)
    if value.ptr ~= nil then
        local f = {function}
        return f(value.ptr[0])
    else
        return nil
    end
end"#,
            function = T::function()
        )
    }
    fn gc() -> bool {
        true
    }
}

impl<T: IntoRawConversion + 'static> IntoRawConversion for Option<T> {
    fn function() -> String {
        format!(
            r#"
function(value)
    local f = {create_pointer}
    if value ~= nil then
        return __typename_{self_typename}(f(value))
    else
        return __typename_{self_typename}(nil)
    end
end
"#,
            self_typename = <Self as Type>::typename(),
            create_pointer = <T as IntoRawConversion>::create_pointer()
        )
    }
    fn create_pointer() -> String {
        ptr_type_create_pointer::<Self>()
    }
    fn create_array() -> String {
        panic!("Array of Option<T> are unreliable and have been disabled");
    }
}

impl<T: Type + 'static, E: Type + 'static> Type for Result<T, E> {
    const IS_PRIMITIVE: bool = false;

    fn typename() -> String {
        format!(
            "Result_{T_typename}_{E_typename}",
            T_typename = T::typename(),
            E_typename = E::typename()
        )
    }
    fn typedeclaration() -> String {
        format!(
            r#"typedef struct {{
    const {T_c_typename} *ok;
    const {E_c_typename} *err;
}} {self_typename};"#,
            T_c_typename = T::prefixed_c_typename(),
            E_c_typename = E::prefixed_c_typename(),
            self_typename = Self::prefixed_typename()
        )
    }
    fn dependencies() -> Dependencies {
        let mut dependencies = make_dependencies::<T>();
        dependencies.extend(make_dependencies::<E>());
        dependencies
    }
    fn c_function_argument() -> String {
        format!("const {}*", Self::prefixed_c_typename())
    }
    fn c_mut_function_argument() -> String {
        format!("{}*", Self::prefixed_c_typename())
    }
}

impl<T: FromRawConversion + 'static, E: FromRawConversion + 'static> FromRawConversion
    for Result<T, E>
{
    fn function() -> String {
        format!(
            r#"function(value)
    if value.ok ~= nil then
        local f = {T_function}
        return f(value.ok[0])
    else
        local f = {E_function}
        return nil, f(value.err[0])
    end
end"#,
            T_function = T::function(),
            E_function = E::function()
        )
    }
    fn gc() -> bool {
        true
    }
}

impl<T: Type + 'static> Type for Vec<T> {
    const IS_PRIMITIVE: bool = false;

    fn typename() -> String {
        format!("Vec_{}", T::typename())
    }
    fn typedeclaration() -> String {
        format!(
            r#"typedef struct {{
    const {c_typename} *ptr;
    size_t len;
    size_t capacity;
}} {self_typename};"#,
            c_typename = <T as Type>::prefixed_c_typename(),
            self_typename = Self::prefixed_typename()
        )
    }
    fn dependencies() -> Dependencies {
        make_dependencies::<T>()
    }
    fn c_function_argument() -> String {
        format!("const {}*", <Self as Type>::prefixed_c_typename())
    }
    fn c_mut_function_argument() -> String {
        format!("{}*", <Self as Type>::prefixed_c_typename())
    }
}

impl<T: FromRawConversion + 'static> FromRawConversion for Vec<T> {
    fn function() -> String {
        format!(
            r#"function(value)
    local ret = {{}}
    local len = tonumber(value.len)
    local f = {function}
    for i = 1,len do
        ret[i] = f(value.ptr[i - 1])
    end
    return ret
end"#,
            function = T::function()
        )
    }
    fn gc() -> bool {
        true
    }
}

impl<T: IntoRawConversion + 'static> IntoRawConversion for Vec<T> {
    fn function() -> String {
        format!(
            r#"
function(value)
    if type(value) == "string" then
        return __typename_{self_typename}(value, #value)
    else
        local f = {create_array}
        return __typename_{self_typename}(f(value), #value, 0)
    end
end
"#,
            self_typename = <Self as Type>::typename(),
            create_array = <T as IntoRawConversion>::create_array()
        )
    }
    fn create_pointer() -> String {
        ptr_type_create_pointer::<Self>()
    }
    fn create_array() -> String {
        immediate_type_create_array::<Self>()
    }
}

impl Type for String {
    const IS_PRIMITIVE: bool = true;

    fn c_typename() -> String {
        "char *".to_owned()
    }
    fn typename() -> String {
        "__string_ptr".to_owned()
    }
    fn c_function_argument() -> String {
        format!("const {}", Self::c_typename())
    }
    fn c_mut_function_argument() -> String {
        <Self as Type>::c_typename()
    }
}

impl FromRawConversion for String {
    fn function() -> String {
        "ffi.string".to_owned()
    }
    fn gc() -> bool {
        true
    }
}

impl IntoRawConversion for String {
    fn function() -> String {
        "function(value) return value end".to_owned()
    }
    fn create_pointer() -> String {
        primitive_type_create_pointer::<Self>()
    }
    fn create_array() -> String {
        primitive_type_create_array::<Self>()
    }
}

macro_rules! primitive_lua_from_native {
    ($($typ:ty)*) => {
        $(
            impl Type for $typ {
                const IS_PRIMITIVE: bool = true;

                fn typename() -> String {
                    stringify!($typ).to_owned()
                }
                fn c_typename() -> String {
                    stringify!($typ).to_owned()
                }
                fn c_function_argument() -> String {
                    <Self as Type>::c_typename()
                }
                fn c_mut_function_argument() -> String {
                    <Self as Type>::c_typename()
                }
            }

            impl FromRawConversion for $typ {
                fn function() -> String {
                    "function(value) return value end".to_owned()
                }
                fn gc() -> bool {
                    false
                }
            }

            impl IntoRawConversion for $typ {

                fn function() -> String {
                    "function(value) return value end".to_owned()
                }
                fn create_pointer() -> String {
                    primitive_type_create_pointer::<Self>()
                }
                fn create_array() -> String {
                    primitive_type_create_array::<Self>()
                }
            }
        )*
    };
}

macro_rules! primitive_slice_lua_native {
    ($($typ:ty)*) => {
        $(
            impl<'a> Type for &'a [$typ] {
                const IS_PRIMITIVE: bool = false;

                fn typename() -> String {
                    format!("Slice_{}", stringify!($typ))
                }

                fn typedeclaration() -> String {
                    format!(r#"typedef struct {{
    const {c_name} *ptr;
    size_t len;
}} {self_typename};"#,
                        c_name = stringify!($typ),
                        self_typename = Self::prefixed_typename())
                }
                fn c_function_argument() -> String {
                    format!("const {}*", Self::prefixed_c_typename())
                }
                fn c_mut_function_argument() -> String {
                    // Mutable not supported
                    Self::c_function_argument()
                }
            }
        )*
    };
}

macro_rules! primitive_slice_lua_to_native {
    ($($typ:ty)*) => {
        $(
            impl<'a> IntoRawConversion for &'a [$typ] {

                fn function() -> String {
                    format!(
            r#"function(value)
    local result = {{}}
    for i, value in pairs(value) do
        result[i] = value
    end
    return __typename_{self_typename}(__c_function_argument_{typename}(#result, result), #result)
end"#,
                        self_typename = Self::typename(),
                        typename = <$typ as Type>::typename())
                }
                fn create_pointer() -> String {
                    ptr_type_create_pointer::<Self>()
                }
                fn create_array() -> String {
                    ptr_type_create_array::<Self>()
                }
            }
        )*
    };
}

use libc::{
    int16_t, int32_t, int64_t, int8_t, size_t, ssize_t, uint16_t, uint32_t, uint64_t, uint8_t,
};

#[allow(non_camel_case_types)]
type float = f32;
#[allow(non_camel_case_types)]
type double = f64;

primitive_lua_from_native!(
    int8_t
    int16_t
    int32_t
    int64_t
    uint8_t
    uint16_t
    uint32_t
    uint64_t
    ssize_t
    size_t
    float
    double
);

primitive_slice_lua_native!(
    int8_t
    int16_t
    int32_t
    int64_t
    uint8_t
    uint16_t
    uint32_t
    uint64_t
    ssize_t
    size_t
    float
    double
);

primitive_slice_lua_to_native!(
    int8_t
    int16_t
    int32_t
    int64_t
    uint16_t
    uint32_t
    uint64_t
    ssize_t
    size_t
    float
    double
);

impl<'a> IntoRawConversion for &'a [u8] {
    fn function() -> String {
        format!(
            r#"function(value)
    if type(value) == "string" then
        return __typename_{self_typename}(value, #value)
    else
        local result = {{}}
        for i, value in pairs(value) do
            result[i] = value
        end
        return __typename_{self_typename}(
            __c_function_argument_{typename}(#result, result), #result)
    end
end"#,
            self_typename = <Self as Type>::typename(),
            typename = <u8 as Type>::typename()
        )
    }
    fn create_pointer() -> String {
        ptr_type_create_pointer::<Self>()
    }
    fn create_array() -> String {
        ptr_type_create_array::<Self>()
    }
}

impl<'a> Type for &'a str {
    const IS_PRIMITIVE: bool = true;

    fn typename() -> String {
        "_str_ptr__".to_owned()
    }
    fn c_typename() -> String {
        "char *".to_owned()
    }
    fn c_function_argument() -> String {
        format!("const {}", Self::c_typename())
    }
    fn c_mut_function_argument() -> String {
        // Mutable not supported
        Self::c_function_argument()
    }
}

impl<'a> IntoRawConversion for &'a str {
    fn function() -> String {
        "function(value) return value end".to_owned()
    }
    fn create_pointer() -> String {
        primitive_type_create_pointer::<Self>()
    }
    fn create_array() -> String {
        primitive_type_create_array::<Self>()
    }
}

impl Type for bool {
    const IS_PRIMITIVE: bool = true;

    fn typename() -> String {
        stringify!(bool).to_owned()
    }
    fn c_typename() -> String {
        stringify!(int8_t).to_owned()
    }
    fn c_function_argument() -> String {
        Self::c_typename()
    }
    fn c_mut_function_argument() -> String {
        Self::c_typename()
    }
}

impl FromRawConversion for bool {
    fn function() -> String {
        "function(value) return value ~= 0 end".to_owned()
    }
    fn gc() -> bool {
        false
    }
}

impl IntoRawConversion for bool {
    fn function() -> String {
        "function(value) return value and 1 or 0 end".to_owned()
    }
    fn create_pointer() -> String {
        primitive_type_create_pointer::<Self>()
    }
    fn create_array() -> String {
        primitive_type_create_array::<Self>()
    }
}
