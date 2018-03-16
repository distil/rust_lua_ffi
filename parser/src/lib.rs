#[macro_use]
extern crate quote;
extern crate syn;


pub fn extern_ffi_mod(file: &::syn::File) -> Option<&[::syn::Item]> {
    file.items
        .iter()
        .filter_map(|item| match *item {
            syn::Item::Mod(ref m) if AsRef::<str>::as_ref(&m.ident) == stringify!(extern_ffi) => {
                m.content.as_ref().map(|t| &t.1[..])
            }
            _ => None,
        })
        .next()
}


pub fn uses(items: &[::syn::Item]) -> Vec<::quote::Tokens> {
    items
        .iter()
        .filter_map(|item| if let ::syn::Item::Use(ref view_path) = *item {
            Some(quote!(#view_path))
        } else {
            None
        })
        .collect()
}

pub struct Argument {
    pub ident: syn::Ident,
    pub typ: ::quote::Tokens,
}

pub struct Function {
    pub ident: syn::Ident,
    pub args: Vec<Argument>,
    pub ret: ::quote::Tokens,
}

pub fn functions(items: &[::syn::Item]) -> Vec<Function> {
    items
        .iter()
        .filter_map(|item| if let ::syn::Item::Fn(ref fn_decl) = *item {
            Some((&fn_decl.ident, &fn_decl.decl.inputs, &fn_decl.decl.output))
        } else {
            None
        })
        .map(|(ident, args, output)| {
            let args: Vec<_> = args.iter()
                .map(|arg| {
                    let (name, ty_arg) = match *arg {
                        ::syn::FnArg::Captured(ref cap) => match cap.pat {
                            ::syn::Pat::Ident(ref pat) => (&pat.ident, &cap.ty),
                            _ => panic!("Unknown identifier"),
                        },
                        _ => panic!("Unknown identifier"),
                    };
                    let typ = match *ty_arg {
                        ::syn::Type::Reference(::syn::TypeReference {
                            elem: ref ty,
                            mutability: None,
                            ..
                        }) => match **ty {
                            ::syn::Type::Path(ref path) => {
                                quote! { &#path }
                            }
                            ::syn::Type::Slice(ref ty) => {
                                if let ::syn::Type::Path(ref path) = *ty.elem {
                                    quote! { &[#path] }
                                } else {
                                    panic!(
                                        "Slice: Function arguments can only be immutable \
                                         reference or immediate"
                                    )
                                }
                            }
                            _ => panic!(
                                "Reference: Function arguments can only be immutable \
                                 reference or immediate"
                            ),
                        },
                        ::syn::Type::Path(ref path) => {
                            quote! { #path }
                        }
                        _ => panic!(
                            "Function arguments can only be immutable reference or immediate"
                        ),
                    };
                    Argument {
                        ident: name.clone(),
                        typ,
                    }
                })
                .collect();
            Function {
                ident: ident.clone(),
                args,
                ret: match *output {
                    ::syn::ReturnType::Default => quote! { () },
                    ::syn::ReturnType::Type(_, ref ty) => if let ::syn::Type::Path(ref path) = **ty
                    {
                        quote! { #path }
                    } else {
                        panic!("Function return type can only be immediate")
                    },
                },
            }
        })
        .collect()
}

pub fn function_declarations(functions: &[Function], uses: &[::quote::Tokens]) -> ::quote::Tokens {
    let extern_c_ffi_functions = functions.iter().map(|function| {
        let argument_declaration = function.args.iter().map(|arg| {
            let ident = &arg.ident;
            let typ = &arg.typ;
            quote! { #ident: <#typ as ::c_marshalling::PtrAsReference>::Ptr }
        });
        let argument_passing = function.args.iter().map(|arg| {
            let ident = &arg.ident;
            let typ = &arg.typ;
            quote! {
                <#typ as ::c_marshalling::PtrAsReference>::ptr_as_ref(#ident)?
            }
        });
        let gc_ident =
            ::syn::parse_str::<::syn::Path>(&format!("__gc_{}", function.ident)).unwrap();
        let ret = &function.ret;
        let ident = &function.ident;
        quote! {
                #[no_mangle]
                pub unsafe extern "C" fn #ident(
                        #(#argument_declaration,)*
                        __output: *mut <#ret as ::c_marshalling::IntoRawConversion>::Ptr) -> u32 {
                    ::std::panic::catch_unwind(|| -> Result<u32, ::c_marshalling::Error> {
                        *__output = <#ret as ::c_marshalling::IntoRawConversion >::into_ptr(
                            super::extern_ffi::#ident(#(#argument_passing),*)
                        )?;
                        Ok(0)
                    }).unwrap_or(Ok(2)).unwrap_or(1)
                }

                #[no_mangle]
                pub unsafe extern "C" fn #gc_ident(
                        output: <#ret as ::c_marshalling::IntoRawConversion>::Ptr) -> u32 {
                    <#ret as ::c_marshalling::FromRawConversion >::from_ptr(output)
                        .is_err() as u32
                }
        }
    });

    quote! {
        #[doc(hidden)]
        pub mod extern_c_ffi {
            #(#uses)*

            #(#extern_c_ffi_functions) *
        }
    }
}
