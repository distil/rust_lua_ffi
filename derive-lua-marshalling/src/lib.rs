extern crate proc_macro;
extern crate syn;
extern crate derive_c_marshalling_library;
#[macro_use]
extern crate quote;

fn lua_marshalling(derive_input: &::syn::DeriveInput) -> ::quote::Tokens {
    let ident = &derive_input.ident;

    match derive_input.body {
        ::syn::Body::Struct(::syn::VariantData::Struct(ref fields)) => {
            let lua_c_struct_fields = fields
                .iter()
                .map(|field| {
                    let ident = &field.ident.as_ref().unwrap().to_string();
                    let ty = &field.ty;
                    quote! {
                        format!("    {} {};\n",
                            <#ty as ::lua_marshalling::Type>::c_typename(),
                            #ident)
                    }
                });
            let mut lua_table_field_initializers: Vec<_> = fields
                .iter()
                .map(|field| {
                    let ident = &field.ident.as_ref().unwrap().to_string();
                    let ty = &field.ty;
                    quote! {
                        format!("{} = invoke(value.{}, {})",
                            #ident,
                            #ident,
                            <#ty as ::lua_marshalling::FromRawConversion>::function())
                    }
                })
                .collect();
            lua_table_field_initializers.push(
                quote! {
                    "__c_ptr__ = value".to_owned()
                });
            let lua_dependencies = fields
                .iter()
                .map(|field| {
                    let ty = &field.ty;
                    quote! {
                        dependencies.extend(::lua_marshalling::make_dependencies::<#ty>());
                    }
                });

            quote! {
                impl ::lua_marshalling::Type for #ident {
                    fn typename() -> String {
                        stringify!(#ident).to_string()
                    }
                    fn typedeclaration() -> String {
                        let fields: &[String] = &[
                            #(#lua_c_struct_fields),*
                        ];
                        format!(r#"
typedef struct {{
{}}} {};"#,
                            fields.join(" "),
                            Self::typename())
                    }
                    fn metatype() -> String {
                        "".to_owned()
                    }
                    fn dependencies() -> ::lua_marshalling::Dependencies {
                        let mut dependencies = ::lua_marshalling::Dependencies::new();
                        #(#lua_dependencies)*
                        dependencies
                    }
                }

                impl ::lua_marshalling::FromRawConversion for #ident {
                    fn function() -> String {
                        format!(
                            r#"function(value)
    return readonlytable {{
        {}
    }}
end"#,
                            &[
                                #(#lua_table_field_initializers),*
                            ].join(", "))
                    }
                    fn c_mut_function_argument() -> String {
                        format!("{}*", <Self as ::lua_marshalling::Type>::typename())
                    }
                    fn gc() -> bool {
                        true
                    }
                }

                impl ::lua_marshalling::IntoRawConversion for #ident {
                    fn function() -> String {
                        "function(value) return value.__c_ptr__ end".to_string()
                    }
                    fn c_function_argument() -> String {
                        format!("const {}*", <Self as ::lua_marshalling::Type>::c_typename())
                    }
                    fn to_pointer() -> String {
                        ::lua_marshalling::ptr_type_to_pointer::<Self>()
                    }
                    fn to_array() -> String {
                        ::lua_marshalling::ptr_type_to_array::<Self>()
                    }
                }
            }
        },
        _ => panic!("Only non-tuple struct supported")
    }
}

#[proc_macro_derive(LuaMarshalling)]
pub fn derive_lua_marshalling(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let c = derive_c_marshalling_library::c_marshalling(
        &syn::parse_derive_input(&input.to_string()).unwrap());
    let lua = lua_marshalling(&syn::parse_derive_input(&input.to_string()).unwrap());
    let quote = quote! {
        #lua

        #c
    };
    quote.parse().unwrap()
}
