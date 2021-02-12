#![recursion_limit = "128"]
use quote::*;

pub fn c_marshalling(derive_input: &syn::DeriveInput) -> quote::Tokens {
    let ident = &derive_input.ident;
    let marshal_typename: syn::Ident = syn::parse_str(&format!("__c_{}", ident)).unwrap();
    let mut_marshal_typename: syn::Ident = syn::parse_str(&format!("__c_mut_{}", ident)).unwrap();

    match derive_input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => {
            let marshal_type_field_declarations = fields.named.iter().map(|field| {
                let ident = &field.ident.as_ref().unwrap();
                let ty = &field.ty;
                quote! { #ident: <#ty as c_marshalling::PtrAsReference>::Raw }
            });
            let mut_marshal_type_field_declarations = fields.named.iter().map(|field| {
                let ident = &field.ident.as_ref().unwrap();
                let ty = &field.ty;
                quote! { #ident: <#ty as c_marshalling::FromRawConversion>::Raw }
            });
            let into_raw_field_initializers = fields.named.iter().map(|field| {
                let ident = &field.ident.as_ref().unwrap();
                quote! { #ident: self.#ident.into_raw()? }
            });
            let from_raw_field_initializers = fields.named.iter().map(|field| {
                let ident = &field.ident.as_ref().unwrap();
                quote! { #ident: c_marshalling::FromRawConversion::from_raw(raw.#ident)? }
            });
            let raw_as_ref_field_initializers = fields.named.iter().map(|field| {
                let ident = &field.ident.as_ref().unwrap();
                quote! { #ident: c_marshalling::PtrAsReference::raw_as_ref(&raw.#ident)? }
            });

            quote! {

                #[doc(hidden)]
                #[allow(non_snake_case)]
                #[repr(C)]
                pub struct #marshal_typename {
                    #(#marshal_type_field_declarations),*
                }

                #[doc(hidden)]
                #[allow(non_snake_case)]
                #[repr(C)]
                pub struct #mut_marshal_typename {
                    #(#mut_marshal_type_field_declarations),*
                }

                impl c_marshalling::IntoRawConversion for #ident {
                    type Raw = #mut_marshal_typename;
                    type Ptr = *mut Self::Raw;

                    fn into_raw(self) -> Result<Self::Raw, c_marshalling::Error> {
                        Ok(Self::Raw {
                            #(#into_raw_field_initializers),*
                        })
                    }

                    fn into_ptr(self) -> Result<Self::Ptr, c_marshalling::Error> {
                        c_marshalling::box_into_ptr(self)
                    }
                }

                impl c_marshalling::FromRawConversion for #ident {
                    type Raw = #mut_marshal_typename;
                    type Ptr = *mut Self::Raw;

                    unsafe fn from_raw(raw: #mut_marshal_typename)
                        -> Result<Self, c_marshalling::Error> {
                        Ok(Self {
                            #(#from_raw_field_initializers),*
                        })
                    }

                    unsafe fn from_ptr(raw: Self::Ptr) -> Result<Self, c_marshalling::Error> {
                        c_marshalling::box_from_ptr(raw)
                    }
                }

                impl c_marshalling::PtrAsReference for #ident {
                    type Raw = #marshal_typename;
                    type Ptr = *const Self::Raw;

                    unsafe fn raw_as_ref(raw: &#marshal_typename)
                        -> Result<Self, c_marshalling::Error> {
                        Ok(Self {
                            #(#raw_as_ref_field_initializers),*
                        })
                    }

                    unsafe fn ptr_as_ref(raw: Self::Ptr) -> Result<Self, c_marshalling::Error> {
                       Self::raw_as_ref(&*raw)
                    }
                }
            }
        }
        syn::Data::Struct(::syn::DataStruct {
            fields: syn::Fields::Unnamed(..),
            ..
        }) => panic!("Tuple-struct type not supported"),
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Unit,
            ..
        }) => panic!("Unit type not supported"),
        syn::Data::Enum(_) => panic!("Enum type not supported"),
        syn::Data::Union(_) => panic!("Union type not supported"),
    }
}
