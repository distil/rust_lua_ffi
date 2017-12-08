#[derive(Debug, Eq, PartialEq, LuaMarshalling)]
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

pub mod extern_ffi {
    use super::{A, D};

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

    pub fn describe(a: A, b: super::B, c: super::C, d: D, e: super::E) -> String {
        format!("A: {:?}, B: {:?}, C: {:?}, D: {:?}, E: {:?}", a, b, c, d, e)
    }

    pub fn random_short() -> i16 {
        3
    }

    // Note: No support for return ! or ()
    pub fn i_like_to_panic() -> i32 {
        panic!("p-p-p-p-p-anic!");
    }
}

include!(concat!(env!("OUT_DIR"), "/ffi.rs"));
