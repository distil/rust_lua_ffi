#[macro_use]
extern crate quote;
extern crate syn;
extern crate syntex_syntax as syntax;

pub fn extern_ffi_mod<'a>(
    krate: &'a ::syntax::ast::Crate,
) -> Option<&'a Vec<::syntax::ptr::P<::syntax::ast::Item>>> {
    use syntax::ast::ItemKind::Mod;

    krate
        .module
        .items
        .iter()
        .find(|item| item.ident.name.as_str() == stringify!(extern_ffi))
        .and_then(|item| {
            if let Mod(::syntax::ast::Mod {
                inner: _,
                ref items,
            }) = item.node
            {
                Some(items)
            } else {
                None
            }
        })
}

pub fn uses(items: &Vec<::syntax::ptr::P<::syntax::ast::Item>>) -> Vec<::quote::Tokens> {
    items
        .iter()
        .filter_map(|item| {
            if let ::syntax::ast::ItemKind::Use(ref view_path) = item.node {
                match view_path.node {
                    ::syntax::ast::ViewPath_::ViewPathSimple(ref ident, ref path) => {
                        let ident = ::syn::parse_ident(&*ident.name.as_str()).unwrap();
                        let path = ::syn::parse_path(&path.to_string()).unwrap();
                        Some(quote! {
                            #path as #ident
                        })
                    }
                    ::syntax::ast::ViewPath_::ViewPathGlob(ref path) => {
                        let path = ::syn::parse_path(&path.to_string()).unwrap();
                        Some(quote! {
                            #path
                        })
                    }
                    ::syntax::ast::ViewPath_::ViewPathList(ref path, ref path_list_items) => {
                        let path = ::syn::parse_path(&path.to_string()).unwrap();
                        let path_list_items = path_list_items.iter().map(|path_list_item| {
                            let rename = ::syn::parse_ident(&*path_list_item
                                .node
                                .rename
                                .unwrap_or(path_list_item.node.name)
                                .name
                                .as_str())
                                .unwrap();
                            let ident = ::syn::parse_ident(
                                &*path_list_item.node.name.name.as_str(),
                            ).unwrap();
                            quote!{
                                #ident as #rename
                            }
                        });
                        Some(quote! {
                            #path::{#(#path_list_items),*}
                        })
                    }
                }
            } else {
                None
            }
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

pub fn functions(items: &Vec<::syntax::ptr::P<::syntax::ast::Item>>) -> Vec<Function> {
    items
        .iter()
        .filter_map(|item| {
            if let ::syntax::ast::ItemKind::Fn(ref fn_decl, _, _, _, _, _) = item.node {
                Some((&item.ident.name, &fn_decl.inputs, fn_decl.output.clone()))
            } else {
                None
            }
        })
        .map(|(ref name, ref args, ref output)| {
            let ident = ::syn::parse_ident(&*name.as_str()).unwrap();
            let args: Vec<_> = args.iter()
                .map(|arg| {
                    let name = match arg.pat.node {
                        ::syntax::ast::PatKind::Ident(
                            _,
                            syntax::ast::SpannedIdent {
                                node: syntax::ast::Ident { ref name, ctxt: _ },
                                span: _,
                            },
                            None,
                        ) => name,
                        _ => panic!("Unknown identifier"),
                    };
                    let typ = match arg.ty.node {
                        ::syntax::ast::TyKind::Rptr(
                            _,
                            ::syntax::ast::MutTy {
                                ref ty,
                                mutbl: ::syntax::ast::Mutability::Immutable,
                            },
                        ) => match ty.node {
                            ::syntax::ast::TyKind::Path(_, ref path) => {
                                let path = ::syn::parse_path(&path.to_string()).unwrap();
                                quote! { &#path }
                            }
                            ::syntax::ast::TyKind::Slice(ref ty) => {
                                if let ::syntax::ast::TyKind::Path(_, ref path) = ty.node {
                                    let path = ::syn::parse_path(&path.to_string()).unwrap();
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
                        ::syntax::ast::TyKind::Path(_, ref path) => {
                            let path = ::syn::parse_path(&path.to_string()).unwrap();
                            quote! { #path }
                        }
                        _ => panic!(
                            "Function arguments can only be immutable reference or immediate"
                        ),
                    };
                    Argument {
                        ident: ::syn::parse_ident(&*name.as_str()).unwrap(),
                        typ,
                    }
                })
                .collect();
            Function {
                ident,
                args,
                ret: match output {
                    &::syntax::ast::FunctionRetTy::Default(_) => quote! { () },
                    &::syntax::ast::FunctionRetTy::Ty(ref ty) => {
                        if let ::syntax::ast::TyKind::Path(_, ref path) = ty.node {
                            let path = &::syn::parse_path(&path.to_string()).unwrap();
                            quote! { #path }
                        } else {
                            panic!("Function return type can only be immediate")
                        }
                    }
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
        let gc_ident = ::syn::parse_path(&format!("__gc_{}", function.ident)).unwrap();
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
            #(use #uses;)*

            #(#extern_c_ffi_functions) *
        }
    }
}
