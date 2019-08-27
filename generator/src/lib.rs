#![recursion_limit = "256"]

use std::fs::File;
use std::io::Read;

use quote::quote;

fn function_declarations(
    functions: &[parser::Function<impl quote::ToTokens, impl quote::ToTokens>],
    uses: impl Iterator<Item = impl quote::ToTokens>,
    library_name: &str,
    ffi_load_using_cpath: bool,
) -> impl quote::ToTokens + std::fmt::Display {
    let extern_lua_ffi_c_header_functions = functions.iter().map(|function| {
        let ident = function.ident.to_string();
        let ret = &function.ret;
        let argument_declaration: Vec<_> = function
            .args
            .iter()
            .map(|arg| {
                let typ = &arg.typ;
                quote! {
                    <#typ as lua_marshalling::Type>::c_function_argument()
                }
            })
            .chain(std::iter::once(quote! {
                    format!("{}*", <#ret as lua_marshalling::Type>::c_mut_function_argument())
                }))
            .collect();
        quote! {
                    format!(r#"int32_t {ident}(
        {argument_declaration});"#,
                        ident=#ident,
                        argument_declaration=[#(#argument_declaration),*].join(",\n    ")),
                    format!("int32_t __gc_{ident}(
        {argument_declaration});",
                        ident=#ident,
                        argument_declaration=<#ret as lua_marshalling::Type>
                            ::c_mut_function_argument())
                }
    });

    use quote::ToTokens;
    let extern_lua_function_wrappers = functions
        .iter()
        .map(|function|{
            let ident = function.ident.to_string();
            let argument_declaration: Vec<_> = function.args
                .iter()
                .map(|arg| arg.ident.to_token_stream().to_string())
                .collect();
            let argument_passing: Vec<_> = function.args
                .iter()
                .map(|arg| {
                    let ident = arg.ident.to_token_stream().to_string();
                    let typ = &arg.typ;
                    quote!{
                        format!(
                            "({function})({ident})",
                            ident=#ident,
                            function=<#typ as lua_marshalling::IntoRawConversion>::function())
                    }
                })
                .collect();
            let argument_declaration = argument_declaration.join(",\n    ");

            let ret = &function.ret;

            quote!{
                format!(r#"function M.{ident}(
    {argument_declaration})
    local __typeof = __c_mut_function_argument_{typename}
    local __ret_ptr = __typeof(1, {{}})
    local status = rust.{ident}(
        {argument_passing}
    )
    if status ~= 0 then
        error("{ident} failed with status "..status)
    end
    local __ret = __ret_ptr[0]
    {gc}
    local f = {function}
    return f(__ret)
end
"#,
                    ident = #ident,
                    argument_declaration = #argument_declaration,
                    typename = <#ret as lua_marshalling::Type>::typename(),
                    argument_passing = {
                        let mut argument_passing: Vec<String> = [#(#argument_passing),*].to_vec();
                        argument_passing.push("__ret_ptr".to_owned());
                        argument_passing
                    }.join(",\n    "),
                    gc = if <#ret as lua_marshalling::FromRawConversion>::gc() {
                        format!("ffi.gc(__ret, rust.__gc_{})", #ident)
                    } else {
                        "".to_owned()
                    },
                    function = <#ret as lua_marshalling::FromRawConversion>::function()
                )
            }
        });

    let extern_lua_unique_types = functions.iter().map(|function| {
        let args = function.args.iter().map(|arg| {
            let typ = &arg.typ;
            quote! {
                ::lua_marshalling::make_dependencies::<#typ>()
            }
        });

        let ret = &function.ret;
        quote! {
            #(#args,)*
            ::lua_marshalling::make_dependencies::<#ret>(),
        }
    });

    let ffi_load_expression = if ffi_load_using_cpath {
        format!(
            "ffi.load(
                package.searchpath('lib{library_name}', package.cpath)
                or package.searchpath('{library_name}', package.cpath)
                or '{library_name}')",
            library_name = library_name,
        )
    } else {
        format!("ffi.load('{library_name}')", library_name = library_name)
    };

    quote! {
            #[doc(hidden)]
            pub mod lua_bootstrap {
                #(#uses)*

                #[no_mangle]
                pub extern "C" fn __lua_bootstrap() -> *mut libc::c_char {
                    let unique_types: lua_marshalling::Dependencies =
                        [ #(#extern_lua_unique_types)* ]
                            .into_iter()
                            .flat_map(|value| value.into_iter()
                                .map(|(k, v)| (k.clone(), v.clone())))
                            .collect();
                    let sorted_types =
                        lua_marshalling::dependency_sorted_type_descriptions(&unique_types);

                    std::ffi::CString::new(
                        [
                            r#"-- Code generated by Rust Lua interface. DO NOT EDIT.

    local ffi = require("ffi")

    ffi.cdef[[
    "#.to_owned(),
                            sorted_types
                                .iter()
                                .map(|dependencies| (dependencies.typedeclaration)())
                                .collect::<Vec<_>>()
                                .join("\n"),
                            {
                                let functions: Vec<String> =
                                    vec![#(#extern_lua_ffi_c_header_functions),*];
                                functions
                            }.join("\n"),
                            format!(r#"
    ]]

    local rust = {ffi_load_expression}

    local M = {{}}

    "#, ffi_load_expression = #ffi_load_expression),
                            sorted_types
                                .iter()
                                .map(|dependencies| (dependencies.metatype)())
                                .collect::<Vec<_>>()
                                .join("\n"),
                            #(#extern_lua_function_wrappers,)*
                            r#"
    return M
    "#.to_owned()
                        ].join("\n"))
                            .ok()
                            .map(std::ffi::CString::into_raw)
                            .unwrap_or_else(std::ptr::null_mut)
                }

                #[no_mangle]
                pub unsafe extern "C" fn __free_lua_bootstrap(bootstrap: *mut libc::c_char) {
                    if bootstrap != std::ptr::null_mut() {
                        std::ffi::CString::from_raw(bootstrap);
                    }
                }
            }
        }
}

pub fn generate(
    file_name: &std::path::Path,
    library_name: &str,
    ffi_load_using_cpath: bool,
) -> String {
    let mut input = String::new();
    let input = {
        let mut file = File::open(file_name).unwrap();
        file.read_to_string(&mut input).unwrap();
        &input
    };
    let file = syn::parse_file(input).unwrap();
    let items = parser::extern_ffi_mod(&file).expect("ffi module");
    let uses = parser::uses(items.iter()).collect::<Vec<_>>();
    let functions = parser::functions(items.iter().cloned())
        .collect::<Vec<_>>();
    let declarations = parser::function_declarations(
        functions.iter(), uses.iter()).to_string();
    let bootstrap = function_declarations(
        &functions, uses.iter(), library_name, ffi_load_using_cpath).to_string();

    format!(
        r#"// Code generated by Rust Lua interface. DO NOT EDIT.
{}
{}
"#,
        declarations, bootstrap
    )
}
