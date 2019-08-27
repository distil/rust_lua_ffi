#![allow(unused_imports)]

use quick_error::quick_error;
quick_error! {
    #[derive(Debug)]
    pub enum Error {
        NulError(err: std::ffi::NulError) {
            display("{}", err)
            from()
        }
        IntoStringError(err: std::ffi::IntoStringError) {
            display("{}", err)
            from()
        }
        Utf8Error(err: std::str::Utf8Error) {
            display("{}", err)
            from()
        }
    }
}

// Types with #[derive(CMarshalling)] implement this trait.
pub trait IntoRawConversion: Sized {
    type Raw: Sized;
    type Ptr: Sized;

    /// This method releases ownership `self`.
    /// A successfully returned type *must* be free'd using
    /// `FromRawConversion::from_raw` and said method only.
    ///
    /// `PtrAsReference::raw_as_ref` can be used to access the type
    /// but it will *not* return ownership.
    fn into_raw(self) -> Result<Self::Raw, Error>;

    fn into_ptr(self) -> Result<Self::Ptr, Error>;
}

// Types with #[derive(CMarshalling)] implement this trait.
pub trait FromRawConversion: Sized {
    type Raw: Sized;
    type Ptr: Sized;

    /// This method takes ownership of the `raw` object.
    /// Use `PtrAsReference::raw_as_ref` to *not* take ownership of the object.
    unsafe fn from_raw(raw: Self::Raw) -> Result<Self, Error>;

    unsafe fn from_ptr(ptr: Self::Ptr) -> Result<Self, Error>;
}

// Types with #[derive(CMarshalling)] implement this trait.
pub trait PtrAsReference: Sized {
    type Raw: Sized;
    type Ptr: Sized;

    /// This method does not take ownership of the object pointed to by `raw`.
    /// Use `FromRawConversion::from_raw` to take ownership of the pointer.
    unsafe fn raw_as_ref(raw: &Self::Raw) -> Result<Self, Error>;

    unsafe fn ptr_as_ref(ptr: Self::Ptr) -> Result<Self, Error>;
}

pub fn box_into_ptr<R, T: IntoRawConversion<Raw = R>>(value: T) -> Result<*mut T::Raw, Error> {
    value.into_raw().map(Box::new).map(Box::into_raw)
}

pub unsafe fn box_from_ptr<R, T: FromRawConversion<Raw = R>>(raw: *mut T::Raw) -> Result<T, Error> {
    T::from_raw(*Box::from_raw(raw))
}

#[repr(C)]
pub struct CVec<T> {
    pub ptr: *const T,
    pub len: usize,
    pub capacity: usize,
}

#[repr(C)]
pub struct CMutVec<T> {
    pub ptr: *mut T,
    pub len: usize,
    pub capacity: usize,
}

impl<T: IntoRawConversion> IntoRawConversion for Vec<T> {
    type Raw = CMutVec<T::Raw>;
    type Ptr = *mut Self::Raw;

    fn into_raw(self) -> Result<Self::Raw, Error> {
        let mut vec = self.into_iter()
            .map(T::into_raw)
            .collect::<Result<Vec<_>, Error>>()?;
        let mut_vec = CMutVec {
            ptr: vec.as_mut_ptr(),
            len: vec.len(),
            capacity: vec.capacity(),
        };
        std::mem::forget(vec);
        Ok(mut_vec)
    }

    fn into_ptr(self) -> Result<Self::Ptr, Error> {
        box_into_ptr(self)
    }
}

impl<T: FromRawConversion> FromRawConversion for Vec<T> {
    type Raw = CMutVec<T::Raw>;
    type Ptr = *mut Self::Raw;

    unsafe fn from_raw(raw: Self::Raw) -> Result<Self, Error> {
        Vec::from_raw_parts(raw.ptr, raw.len as usize, raw.capacity as usize)
            .into_iter()
            .map(|value| T::from_raw(value))
            .collect()
    }


    unsafe fn from_ptr(ptr: Self::Ptr) -> Result<Self, Error> {
        box_from_ptr(ptr)
    }
}

impl<T: PtrAsReference> PtrAsReference for Vec<T> {
    type Raw = CVec<T::Raw>;
    type Ptr = *const Self::Raw;

    unsafe fn raw_as_ref(raw: &Self::Raw) -> Result<Self, Error> {
        std::slice::from_raw_parts(raw.ptr, raw.len as usize)
            .into_iter()
            .map(|value| T::raw_as_ref(value))
            .collect()
    }


    unsafe fn ptr_as_ref(ptr: Self::Ptr) -> Result<Self, Error> {
        Self::raw_as_ref(&*ptr)
    }
}

impl IntoRawConversion for String {
    type Raw = *mut libc::c_char;
    type Ptr = Self::Raw;

    fn into_raw(self) -> Result<Self::Raw, Error> {
        Ok(std::ffi::CString::new(self)?.into_raw())
    }

    fn into_ptr(self) -> Result<Self::Ptr, Error> {
        self.into_raw()
    }
}

impl FromRawConversion for String {
    type Raw = *mut libc::c_char;
    type Ptr = Self::Raw;

    unsafe fn from_raw(raw: Self::Raw) -> Result<Self, Error> {
        Ok(std::ffi::CString::from_raw(raw).into_string()?)
    }

    unsafe fn from_ptr(ptr: Self::Ptr) -> Result<Self, Error> {
        Self::from_raw(ptr)
    }
}

impl PtrAsReference for String {
    type Raw = *mut libc::c_char;
    type Ptr = Self::Raw;

    unsafe fn raw_as_ref(raw: &Self::Raw) -> Result<Self, Error> {
        Ok(std::ffi::CStr::from_ptr(*raw).to_str()?.to_owned())
    }

    unsafe fn ptr_as_ref(ptr: Self::Ptr) -> Result<Self, Error> {
        Self::raw_as_ref(&ptr)
    }
}

impl<'a> PtrAsReference for &'a str {
    type Raw = *mut libc::c_char;
    type Ptr = Self::Raw;

    unsafe fn raw_as_ref(raw: &Self::Raw) -> Result<Self, Error> {
        Ok(std::ffi::CStr::from_ptr(*raw).to_str()?)
    }

    unsafe fn ptr_as_ref(ptr: Self::Ptr) -> Result<Self, Error> {
        Self::raw_as_ref(&ptr)
    }
}

#[repr(C)]
pub struct COption<T> {
    pub ptr: *const T,
}

#[repr(C)]
pub struct CMutOption<T> {
    pub ptr: *mut T,
}

impl<T: IntoRawConversion> IntoRawConversion for Option<T> {
    type Raw = CMutOption<T::Raw>;
    type Ptr = *mut Self::Raw;

    fn into_raw(self) -> Result<Self::Raw, Error> {
        Ok(CMutOption {
            ptr: if let Some(value) = self {
                box_into_ptr(value)?
            } else {
                std::ptr::null_mut()
            },
        })
    }

    fn into_ptr(self) -> Result<Self::Ptr, Error> {
        box_into_ptr(self)
    }
}

impl<T: FromRawConversion> FromRawConversion for Option<T> {
    type Raw = CMutOption<T::Raw>;
    type Ptr = *mut Self::Raw;

    unsafe fn from_raw(raw: Self::Raw) -> Result<Self, Error> {
        Ok(if !raw.ptr.is_null() {
            Some(box_from_ptr(raw.ptr)?)
        } else {
            None
        })
    }

    unsafe fn from_ptr(ptr: Self::Ptr) -> Result<Self, Error> {
        box_from_ptr(ptr)
    }
}

impl<T: PtrAsReference> PtrAsReference for Option<T> {
    type Raw = COption<T::Raw>;
    type Ptr = *const Self::Raw;

    unsafe fn raw_as_ref(raw: &Self::Raw) -> Result<Self, Error> {
        if let Some(value) = raw.ptr.as_ref() {
            Ok(Some(T::raw_as_ref(value)?))
        } else {
            Ok(None)
        }
    }

    unsafe fn ptr_as_ref(ptr: Self::Ptr) -> Result<Self, Error> {
        Self::raw_as_ref(&*ptr)
    }
}

#[repr(C)]
pub struct CResult<T, E> {
    pub ok: *const T,
    pub err: *const E,
}

#[repr(C)]
pub struct CMutResult<T, E> {
    pub ok: *mut T,
    pub err: *mut E,
}

impl<T: IntoRawConversion, E: IntoRawConversion> IntoRawConversion for Result<T, E> {
    type Raw = CMutResult<T::Raw, E::Raw>;
    type Ptr = *mut Self::Raw;

    fn into_raw(self) -> Result<Self::Raw, Error> {
        Ok(match self {
            Ok(value) => CMutResult { ok: box_into_ptr(value)?, err: std::ptr::null_mut() },
            Err(value) => CMutResult { ok: std::ptr::null_mut(), err: box_into_ptr(value)? },
        })
    }

    fn into_ptr(self) -> Result<Self::Ptr, Error> {
        box_into_ptr(self)
    }
}

impl<T: FromRawConversion, E: FromRawConversion> FromRawConversion for Result<T, E> {
    type Raw = CMutResult<T::Raw, E::Raw>;
    type Ptr = *mut Self::Raw;

    unsafe fn from_raw(raw: Self::Raw) -> Result<Self, Error> {
        Ok(if !raw.ok.is_null() {
            Ok(box_from_ptr(raw.ok)?)
        } else {
            Err(box_from_ptr(raw.err)?)
        })
    }

    unsafe fn from_ptr(ptr: Self::Ptr) -> Result<Self, Error> {
        box_from_ptr(ptr)
    }
}

impl<T: PtrAsReference, E: PtrAsReference> PtrAsReference for Result<T, E> {
    type Raw = CResult<T::Raw, E::Raw>;
    type Ptr = *const Self::Raw;

    unsafe fn raw_as_ref(raw: &Self::Raw) -> Result<Self, Error> {
        if let Some(value) = raw.ok.as_ref() {
            Ok(Ok(T::raw_as_ref(value)?))
        } else if let Some(value) = raw.err.as_ref() {
            Ok(Err(E::raw_as_ref(value)?))
        } else {
            unreachable!()
        }
    }

    unsafe fn ptr_as_ref(ptr: Self::Ptr) -> Result<Self, Error> {
        Self::raw_as_ref(&*ptr)
    }
}

#[repr(C)]
pub struct CSlice<T> {
    pub ptr: *const T,
    pub len: usize,
}

macro_rules! primitive_marshalled_type {
    ($($typ:ty )*) => {
        $(
            impl IntoRawConversion for $typ {
                type Raw = Self;
                type Ptr = Self::Raw;

                fn into_raw(self) -> Result<Self::Raw, Error> {
                    Ok(self)
                }

                fn into_ptr(self) -> Result<Self::Ptr, Error> {
                    Ok(self)
                }
            }

            impl FromRawConversion for $typ {
                type Raw = Self;
                type Ptr = Self::Raw;

                unsafe fn from_raw(raw: Self::Raw) -> Result<Self, Error> {
                    Ok(raw)
                }

                unsafe fn from_ptr(ptr: Self::Ptr) -> Result<Self, Error> {
                    Ok(ptr)
                }
            }

            impl PtrAsReference for $typ {
                type Raw = Self;
                type Ptr = Self::Raw;

                unsafe fn raw_as_ref(raw: &Self::Raw) -> Result<Self, Error> {
                    Ok(raw.clone())
                }

                unsafe fn ptr_as_ref(ptr: Self::Ptr) -> Result<Self, Error> {
                    Ok(ptr)
                }
            }

            impl<'a> PtrAsReference for &'a [$typ] {
                type Raw = CSlice<$typ>;
                type Ptr = *const Self::Raw;

                unsafe fn raw_as_ref(raw: &Self::Raw) -> Result<Self, Error> {
                    Ok(std::slice::from_raw_parts(raw.ptr, raw.len as usize))
                }

                unsafe fn ptr_as_ref(ptr: Self::Ptr) -> Result<Self, Error> {
                    Self::raw_as_ref(&*ptr)
                }
            }
        )*

    };
}

primitive_marshalled_type!(
    i8
    i16
    i32
    i64
    u8
    u16
    u32
    u64
    f32
    f64
    isize
    usize
);

impl IntoRawConversion for bool {
    type Raw = i8;
    type Ptr = Self::Raw;

    fn into_raw(self) -> Result<Self::Raw, Error> {
        Ok(self as Self::Raw)
    }

    fn into_ptr(self) -> Result<Self::Ptr, Error> {
        Ok(self as Self::Ptr)
    }
}

impl FromRawConversion for bool {
    type Raw = i8;
    type Ptr = Self::Raw;

    unsafe fn from_raw(raw: Self::Raw) -> Result<Self, Error> {
        Ok(raw != 0)
    }

    unsafe fn from_ptr(ptr: Self::Ptr) -> Result<Self, Error> {
        Ok(ptr != 0)
    }
}

impl PtrAsReference for bool {
    type Raw = i8;
    type Ptr = Self::Raw;

    unsafe fn raw_as_ref(raw: &Self::Raw) -> Result<Self, Error> {
        Ok(*raw != 0)
    }

    unsafe fn ptr_as_ref(ptr: Self::Ptr) -> Result<Self, Error> {
        Ok(ptr != 0)
    }
}
