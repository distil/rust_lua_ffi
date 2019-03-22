#![recursion_limit = "128"]

extern crate derive_c_marshalling_library;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

fn lua_marshalling(derive_input: &::syn::DeriveInput) -> ::quote::Tokens {
    let ident = &derive_input.ident;

    match derive_input.data {
        ::syn::Data::Struct(::syn::DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => {
            let lua_c_struct_fields = fields.named.iter().map(|field| {
                let ident = &field.ident.as_ref().unwrap().to_string();
                let ty = &field.ty;
                quote! {
                    format!("    const {typename} {ident};",
                        typename=<#ty as ::lua_marshalling::Type>::c_typename(),
                        ident=#ident)
                }
            });
            let lua_table_field_initializers = fields.named.iter().map(|field| {
                let ident = &field.ident.as_ref().unwrap().to_string();
                let ty = &field.ty;
                quote! {
                    format!("{ident} = ({function})(value.{ident})",
                        ident = #ident,
                        function = <#ty as ::lua_marshalling::FromRawConversion>::function())
                }
            });
            let lua_c_struct_field_initializers = fields.named.iter().map(|field| {
                let ident = &field.ident.as_ref().unwrap().to_string();
                let ty = &field.ty;
                quote! { format!("({function})(value.{ident})",
                        ident = #ident,
                        function = <#ty as ::lua_marshalling::IntoRawConversion>::function())
                }
            });
            let lua_dependencies = fields.named.iter().map(|field| {
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
                {fields}
            }} {self_typename};
            "#,
                            fields = fields.join("\n"),
                            self_typename = Self::typename())
                    }
                    fn dependencies() -> ::lua_marshalling::Dependencies {
                        let mut dependencies = ::lua_marshalling::Dependencies::new();
                        #(#lua_dependencies)*
                        dependencies
                    }
                    fn c_function_argument() -> String {
                        format!("const {}*", Self::c_typename())
                    }
                    fn c_mut_function_argument() -> String {
                        format!("{}*", Self::typename())
                    }
                    fn metatype() -> String {
                        ::lua_marshalling::ptr_type_metatype::<Self>()
                    }
                }

                impl ::lua_marshalling::FromRawConversion for #ident {
                    fn function() -> String {
                        format!(
                                                    r#"function(value)
                return {{
                    {}
                }}
            end"#,
                            &[
                                #(#lua_table_field_initializers),*
                            ].join(", "))
                    }
                    fn gc() -> bool {
                        true
                    }
                }

                impl ::lua_marshalling::IntoRawConversion for #ident {
                    fn function() -> String {
                        let fields: &[String] = &[
                            #(#lua_c_struct_field_initializers),*
                        ];
                                                format!(r#"function(value)
                return __typename_{self_typename}(
                    {fields}
                )
            end"#,
                            self_typename = <Self as ::lua_marshalling::Type>::typename(),
                            fields = fields.join(",\n    "))
                    }
                    fn create_pointer() -> String {
                        ::lua_marshalling::ptr_type_create_pointer::<Self>()
                    }
                    fn create_array() -> String {
                        ::lua_marshalling::immediate_type_create_array::<Self>()
                    }
                }
            }
        }
        _ => panic!("Only non-tuple struct supported"),
    }
}

#[proc_macro_derive(LuaMarshalling)]
pub fn derive_lua_marshalling(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let c = derive_c_marshalling_library::c_marshalling(&syn::parse(input.clone()).unwrap());
    let lua = lua_marshalling(&syn::parse(input).unwrap());
    let quote = quote! {
        #lua

        #c
    };
    quote.into()
}
