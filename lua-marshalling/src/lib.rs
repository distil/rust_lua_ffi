#![allow(unused_imports)]

extern crate libc;
#[macro_use]
extern crate derive_lua_marshalling;

// procedural crates do not allow exporting code themselves, so re-export the crate and
// implement the library here.
pub use derive_lua_marshalling::*;

#[derive(Debug, Clone)]
pub struct TypeDescription {
    pub typeid: ::std::any::TypeId,
    pub dependencies: ::std::collections::HashSet<::std::any::TypeId>,
    pub typedeclaration: fn() -> String,
    pub metatype: fn() -> String,
}

pub trait Type {
    fn typename() -> String;
    fn c_typename() -> String {
        Self::typename()
    }
    fn typedeclaration() -> String {
        "".to_owned()
    }
    fn metatype() -> String;
    fn dependencies() -> Dependencies {
        vec![]
            .into_iter()
            .collect()
    }
    fn c_function_argument() -> String;
    fn c_mut_function_argument() -> String;
}

pub trait FromRawConversion : Type {
    fn function() -> String;
    fn gc() -> bool;
}

pub trait IntoRawConversion : Type {
    fn function() -> String;
    fn create_pointer() -> String;
    fn create_array() -> String;
}

pub type Dependencies = ::std::collections::HashMap<::std::any::TypeId, TypeDescription>;

pub fn make_dependencies<T: Type + 'static>() -> Dependencies {
    let typeid = ::std::any::TypeId::of::<T>();
    let mut dependencies = T::dependencies();
    let type_dependencies = dependencies.keys().cloned().collect();
    dependencies
        .insert(
            typeid,
            TypeDescription{
                typeid: ::std::any::TypeId::of::<T>(),
                dependencies: type_dependencies,
                typedeclaration: T::typedeclaration,
                metatype: T::metatype,
            });
    dependencies
}

pub fn dependency_sorted_type_descriptions<'a>(
    dependencies: &'a Dependencies) -> Vec<&'a TypeDescription> {
    let mut remaining : ::std::collections::HashSet<_> = dependencies.keys().cloned().collect();
    let mut sorted_dependencies = Vec::new();
    while !remaining.is_empty() {
        let typ = {
            let (typ, dependencies) = remaining
                .iter()
                .map(|typ| (typ, dependencies.get(typ).unwrap()) )
                .find(|&(_, dependencies)| {
                    dependencies.dependencies.is_disjoint(&remaining)
                })
                .unwrap();
            sorted_dependencies.push(dependencies);
            typ.clone()
        };
        assert!(remaining.remove(&typ));
    }
    sorted_dependencies
}

pub fn ptr_type_metatype<T: Type>() -> String {
    format!(r#"
local __typename_{self_typename} = ffi.metatype("{c_typename}", {{}})
local __const_c_typename_{self_typename} = ffi.typeof("const {c_typename}[?]")
local __c_function_argument_{self_typename} = ffi.typeof("{c_function_argument}[?]")
local __c_mut_function_argument_{self_typename} = ffi.typeof("{c_mut_function_argument}[?]")
"#,
    self_typename = T::typename(),
    c_typename = T::c_typename(),
    c_function_argument = T::c_function_argument(),
    c_mut_function_argument = T::c_mut_function_argument())
}

pub fn primitive_type_metatype<T: Type>() -> String {
    format!(r#"
local __const_c_typename_{self_typename} = ffi.typeof("const {c_typename}[?]")
local __c_function_argument_{self_typename} = ffi.typeof("{c_function_argument}[?]")
local __c_mut_function_argument_{self_typename} = ffi.typeof("{c_mut_function_argument}[?]")
"#,
            self_typename = T::typename(),
            c_typename = T::c_typename(),
            c_function_argument = T::c_function_argument(),
            c_mut_function_argument = T::c_mut_function_argument())
}

pub fn ptr_type_create_pointer<T: IntoRawConversion>() -> String {
    T::function()
}

pub fn ptr_type_create_array<T: IntoRawConversion>() -> String {
    format!(
        r#"function(value)
    local result = {{}}
    for i, value in pairs(value) do
        local tmp = invoke(value, {function})
        result[i] = tmp[0]
    end
    return __const_c_typename_{typename}(#result, result)
end"#,
        function = T::function(),
        typename = <T as Type>::typename())
}

pub fn immediate_type_create_array<T: IntoRawConversion>() -> String {
    format!(
        r#"function(value)
    local result = {{}}
    for i, value in pairs(value) do
        local tmp = invoke(value, {function})
        result[i] = tmp
    end
    return __const_c_typename_{typename}(#result, result)
end"#,
        function = T::function(),
        typename = <T as Type>::typename())
}

fn primitive_type_create_pointer<T: IntoRawConversion>() -> String {
    format!(r#"function(value)
    return __const_c_typename_{typename}(1, {{ value }})
end"#,
            typename = <T as Type>::typename())
}

fn primitive_type_create_array<T: IntoRawConversion>() -> String {
    format!(
        r#"function(value)
    return __const_c_typename_{typename}(#value, value)
end"#,
        typename = T::typename())
}

impl<T: Type + 'static> Type for Option<T> {
    fn typename() -> String {
        format!("Option_{}", T::typename())
    }
    fn typedeclaration() -> String {
        format!(
            r#"typedef struct {{
    const {c_typename} *ptr;
}} {self_typename};"#,
            c_typename = <T as Type>::c_typename(),
            self_typename = Self::typename())
    }
    fn dependencies() -> Dependencies {
        make_dependencies::<T>()
    }
    fn c_function_argument() -> String {
        format!("const {}*", Self::c_typename())
    }
    fn c_mut_function_argument() -> String {
        format!("{}*", Self::c_typename())
    }
    fn metatype() -> String {
        ptr_type_metatype::<Self>()
    }
}

impl<T: FromRawConversion + 'static> FromRawConversion for Option<T> {
    fn function() -> String {
        format!(
            r#"function(value)
    return value.ptr ~= nil and invoke(value.ptr[0], {function}) or nil
end"#,
            function = T::function())
    }
    fn gc() -> bool {
        true
    }
}

impl<T: IntoRawConversion + 'static> IntoRawConversion for Option<T> {
    fn function() -> String {
        format!(r#"
function(value)
    return __typename_{self_typename}(value ~= nil and invoke(value, {create_pointer}) or nil)
end
"#,
        self_typename = < Self as Type >::typename(),
        create_pointer = <T as IntoRawConversion>::create_pointer())
    }
    fn create_pointer() -> String {
        ptr_type_create_pointer::<Self>()
    }
    fn create_array() -> String {
        panic!("Array of Option<T> are unreliable and have been disabled");
    }
}

impl<T: Type + 'static> Type for Vec<T> {
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
            c_typename = <T as Type>::c_typename(),
            self_typename = Self::typename())
    }
    fn dependencies() -> Dependencies {
        make_dependencies::<T>()
    }
    fn c_function_argument() -> String {
        format!("const {}*", <Self as Type>::c_typename())
    }
    fn c_mut_function_argument() -> String {
        format!("{}*", <Self as Type>::c_typename())
    }
    fn metatype() -> String {
        ptr_type_metatype::<Self>()
    }
}

impl<T: FromRawConversion + 'static> FromRawConversion for Vec<T> {
    fn function() -> String {
        format!(
            r#"function(value)
    local ret = {{}}
    local len = tonumber(value.len)
    for i = 1,len do
        ret[i] = invoke(value.ptr[i - 1], {function})
    end
    return ret
end"#,
            function = T::function())
    }
    fn gc() -> bool {
        true
    }
}

impl<T: IntoRawConversion + 'static> IntoRawConversion for Vec<T> {
    fn function() -> String {
        format!(r#"
function(value)
    if type(value) == "string" then
        return __typename_{self_typename}(value, #value)
    else
        return __typename_{self_typename}(invoke(value, {create_array}), #value, 0)
    end
end
"#,
                self_typename = < Self as Type >::typename(),
                create_array = <T as IntoRawConversion>::create_array())
    }
    fn create_pointer() -> String {
        ptr_type_create_pointer::<Self>()
    }
    fn create_array() -> String {
        immediate_type_create_array::<Self>()
    }
}

impl Type for String {
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
    fn metatype() -> String {
        primitive_type_metatype::<Self>()
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
                fn metatype() -> String {
                    primitive_type_metatype::<Self>()
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
                fn typename() -> String {
                    format!("Slice_{}", stringify!($typ))
                }

                fn typedeclaration() -> String {
                    format!(r#"typedef struct {{
    const {c_name} *ptr;
    size_t len;
}} {self_typename};"#,
                        c_name = stringify!($typ),
                        self_typename = Self::typename())
                }
                fn c_function_argument() -> String {
                    format!("const {}*", <Self as Type>::c_typename())
                }
                fn c_mut_function_argument() -> String {
                    // Mutable not supported
                    Self::c_function_argument()
                }
                fn metatype() -> String {
                    ptr_type_metatype::<Self>()
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

use ::libc::{
    int8_t,
    int16_t,
    int32_t,
    int64_t,
    uint8_t,
    uint16_t,
    uint32_t,
    uint64_t,
    ssize_t,
    size_t
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
        return __typename_{self_typename}(__c_function_argument_{typename}(#result, result), #result)
    end
end"#,
            self_typename = <Self as Type>::typename(),
            typename = <u8 as Type>::typename())
    }
    fn create_pointer() -> String {
        ptr_type_create_pointer::<Self>()
    }
    fn create_array() -> String {
        ptr_type_create_array::<Self>()
    }
}

impl<'a> Type for &'a str {
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
    fn metatype() -> String {
        primitive_type_metatype::<Self>()
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
