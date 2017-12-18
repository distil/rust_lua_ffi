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
                        format!("    {typename} {ident};\n",
                            typename=<#ty as ::lua_marshalling::Type>::c_typename(),
                            ident=#ident)
                    }
                });
            let mut lua_table_field_initializers: Vec<_> = fields
                .iter()
                .map(|field| {
                    let ident = &field.ident.as_ref().unwrap().to_string();
                    let ty = &field.ty;
                    quote! {
                        format!("{ident} = invoke(value.{ident}, {ty})",
                            ident = #ident,
                            ty = <#ty as ::lua_marshalling::FromRawConversion>::function())
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
{fields}}} {self_typename};"#,
                            fields = fields.join(" "),
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
    return readonlytable {{
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
                        "function(value) return value.__c_ptr__ end".to_string()
                    }
                    fn create_pointer() -> String {
                        ::lua_marshalling::ptr_type_create_pointer::<Self>()
                    }
                    fn create_array() -> String {
                        ::lua_marshalling::ptr_type_create_array::<Self>()
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
