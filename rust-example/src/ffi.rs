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

#[derive(Debug, Eq, PartialEq, Default, LuaMarshalling)]
pub struct F {
    thing: Option<Vec<Vec<A>>>,
}

impl F {
    fn describe(&self) -> String {
        self.thing
            .as_ref()
            .and_then(|things: &Vec<Vec<A>>| things.first())
            .and_then(|things: &Vec<A>| things.first())
            .map(|thing: &A| format!("{:#?}", thing))
            .unwrap_or_else(|| "Nothing here".into())
    }
}

#[derive(Debug, Eq, PartialEq, Default, LuaMarshalling)]
pub struct G {
    thing: Option<Option<A>>,
}

impl G {
    fn a(&self) -> Option<&A> {
        self.thing.as_ref().and_then(|inner| inner.as_ref())
    }

    fn string(&self) -> &str {
        self.a().map(|a| a.string.as_ref()).unwrap_or_default()
    }

    fn integer(&self) -> i32 {
        self.a().map(|a| a.integer).unwrap_or_default()
    }
}

pub mod extern_ffi {
    use super::{A, D, F, G};

    pub fn make_a(string: &str, integer: i32) -> A {
        A {
            string: string.to_owned(),
            integer,
        }
    }

    pub fn make_b(string: Option<&str>, integer: Option<i32>) -> super::B {
        super::B {
            string: string.map(|string| string.to_owned()),
            integer,
        }
    }

    pub fn make_c(a: Option<A>, b: Vec<super::B>) -> super::C {
        super::C { a, b }
    }

    pub fn make_d(integers: &[i32]) -> D {
        D {
            integers: integers.to_owned(),
        }
    }

    pub fn make_e(integers: Option<Vec<i32>>, ds: Vec<D>) -> super::E {
        super::E { integers, ds }
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

    pub fn make_f(a: Option<A>) -> F {
        F {
            thing: match a {
                Some(A { string, integer }) => Some(vec![vec![A { string, integer }]]),
                None => None,
            },
        }
    }

    pub fn length_f(f: F) -> usize {
        f.thing.map(|things| things.len()).unwrap_or_default()
    }

    pub fn describe_first_f(f: F) -> String {
        f.describe()
    }

    pub fn make_g(a: Option<A>) -> G {
        match a {
            Some(A { string, integer }) => G {
                thing: Some(Some(A { string, integer })),
            },
            None => G { thing: None },
        }
    }

    pub fn g_get_a(g: G) -> Option<A> {
        g.thing.and_then(|inner| inner)
    }

    pub fn string_g(g: G) -> String {
        g.string().into()
    }

    pub fn integer_g(g: G) -> i32 {
        g.integer()
    }
}

include!(concat!(env!("OUT_DIR"), "/ffi.rs"));
