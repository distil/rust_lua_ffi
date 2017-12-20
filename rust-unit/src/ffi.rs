#[derive(Clone, Debug, Eq, PartialEq, LuaMarshalling)]
pub struct A {
    string: String,
    integer: i32,
}

#[derive(Debug, Eq, PartialEq, LuaMarshalling)]
pub struct B {
    string: Option<String>,
    integer: Option<i32>,
}

#[derive(Debug, Eq, PartialEq, LuaMarshalling)]
pub struct C {
    a: Option<A>,
    b: Vec<B>,
}

#[derive(Debug, Eq, PartialEq, LuaMarshalling)]
pub struct D {
    integers: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq, LuaMarshalling)]
pub struct E {
    integers: Option<Vec<i32>>,
    ds: Vec<D>,
}

#[derive(Debug, Eq, PartialEq, LuaMarshalling)]
pub struct F {
    as_: Option<Vec<A>>,
    strings: Option<Vec<String>>,
}

#[derive(Debug, Eq, PartialEq, LuaMarshalling)]
pub struct G {
    b: bool,
    option_b: Option<bool>,
    vec_b: Vec<bool>,
}

pub mod extern_ffi {
    // Intentionally not `use` all structs to test relative names
    use super::{A, D};

    pub fn square_i8(value: i8) -> i8 {
        value * value
    }
    pub fn square_i16(value: i16) -> i16 {
        value * value
    }
    pub fn square_i32(value: i32) -> i32 {
        value * value
    }
    pub fn square_u8(value: u8) -> u8 {
        value * value
    }
    pub fn square_u16(value: u16) -> u16 {
        value * value
    }
    pub fn square_u32(value: u32) -> u32 {
        value * value
    }
    pub fn square_f32(value: f32) -> f32 {
        value * value
    }
    pub fn square_f64(value: f64) -> f64 {
        value * value
    }
    pub fn concatenate_strings(string1: String, string2: String, separator: &str) -> String {
        [string1, string2].join(separator)
    }
    pub fn concatenate_u16_slices(slice1: &[u16], slice2: &[u16]) -> Vec<u16> {
        slice1.iter().chain(slice2).cloned().collect()
    }
    pub fn concatenate_a(a1: A, a2: A, separator: &str) -> A {
        A {
            string: concatenate_strings(a1.string, a2.string, separator),
            integer: a1.integer + a2.integer,
        }
    }
    pub fn concatenate_vec_i32(vec1: Vec<i32>, vec2: Vec<i32>) -> Vec<i32> {
        vec1.into_iter().chain(vec2).collect()
    }
    pub fn concatenate_vec_a(vec1: Vec<A>, vec2: Vec<A>) -> Vec<A> {
        vec1.into_iter().chain(vec2).collect()
    }
    pub fn concatenate_vec_string(vec1: Vec<String>, vec2: Vec<String>) -> Vec<String> {
        vec1.into_iter().chain(vec2).collect()
    }
    pub fn concatenate_vec_vec_i32(vec1: Vec<Vec<i32>>, vec2: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
        vec1.into_iter().chain(vec2).collect()
    }
    pub fn concatenate_vec_vec_string(
        vec1: Vec<Vec<String>>,
        vec2: Vec<Vec<String>>
    ) -> Vec<Vec<String>> {
        vec1.into_iter().chain(vec2).collect()
    }
    pub fn option_i32_or(option1: Option<i32>, option2: Option<i32>) -> Option<i32> {
        option1.or(option2)
    }
    pub fn option_a_or(option1: Option<A>, option2: Option<A>) -> Option<A> {
        option1.or(option2)
    }
    pub fn option_string_or(
        option1: Option<String>,
        option2: Option<String>
    ) -> Option<String> {
        option1.or(option2)
    }
    pub fn option_vec_i32_or(
        option1: Option<Vec<i32>>,
        option2: Option<Vec<i32>>
    ) -> Option<Vec<i32>> {
        option1.or(option2)
    }
    pub fn option_vec_string_or(
        option1: Option<Vec<String>>,
        option2: Option<Vec<String>>
    ) -> Option<Vec<String>> {
        option1.or(option2)
    }
    pub fn option_vec_a_or(
        option1: Option<Vec<A>>,
        option2: Option<Vec<A>>
    ) -> Option<Vec<A>> {
        option1.or(option2)
    }
    pub fn option_option_i32_or(
        option1: Option<Option<i32>>,
        option2: Option<Option<i32>>
    ) -> Option<Option<i32>> {
        option1.or(option2)
    }
    pub fn option_option_string_or(
        option1: Option<Option<String>>,
        option2: Option<Option<String>>
    ) -> Option<Option<String>> {
        option1.or(option2)
    }
    pub fn option_option_a_or(
        option1: Option<Option<A>>,
        option2: Option<Option<A>>
    ) -> Option<Option<A>> {
        option1.or(option2)
    }

    pub fn make_a(string: &str, integer: i32) -> A {
        A {
            string: string.to_owned(),
            integer,
        }
    }
    pub fn make_b(string: Option<&str>, integer: Option<i32>) -> super::B {
        super::B {
            string: string.map(|string|string.to_owned()),
            integer,
        }
    }
    pub fn make_c(a: Option<A>, b: Vec<super::B>) -> super::C {
        super::C {
            a,
            b,
        }
    }
    pub fn make_d(integers: &[i32]) -> D {
        D {
            integers: integers.to_owned(),
        }
    }
    pub fn make_e(integers: Option<Vec<i32>>, ds: Vec<D>) -> super::E {
        super::E {
            integers,
            ds,
        }
    }
    pub fn make_f(as_: Option<Vec<A>>, strings: Option<Vec<String>>) -> super::F {
        super::F {
            as_,
            strings,
        }
    }

    pub fn make_g(b: bool, option_b: Option<bool>, vec_b: Vec<bool>) -> super::G {
        println!("option_b: {:?}", option_b);
        super::G {
            b,
            option_b,
            vec_b,
        }
    }

    pub fn describe(a: A, b: super::B, c: super::C, d: D, e: super::E, f: super::F) -> String {
        format!("A: {:?}, B: {:?}, C: {:?}, D: {:?}, E: {:?}, F: {:?}", a, b, c, d, e, f)
    }

    // Note: No support for return ! or ()
    pub fn i_like_to_panic() -> i32 {
        panic!("p-p-p-p-p-anic!");
    }

    pub fn u8_slice_to_string(slice: &[u8]) -> String {
        String::from_utf8(slice.to_owned()).unwrap()
    }

    pub fn u8_vec_to_string(vec: Vec<u8>) -> String {
        String::from_utf8(vec)
            .unwrap()
    }

    pub fn string_with_byte_zeros() -> String {
        println!("string_with_byte_zeros");
        "String\0containing\0null\0bytes".to_owned()
    }

    pub fn blob_string(message: &str) -> ::c_marshalling::Blob<String> {
        message.to_owned().into()
    }

    pub fn use_blob_string(string: &::c_marshalling::Blob<String>) -> String {
        string.as_ref().to_owned()
    }
}

include!(concat!(env!("OUT_DIR"), "/ffi.rs"));
