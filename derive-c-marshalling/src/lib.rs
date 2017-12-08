extern crate proc_macro;
extern crate syn;
extern crate derive_c_marshalling_library;

#[proc_macro_derive(CMarshalling)]
pub fn c_marshalling(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_c_marshalling_library::c_marshalling(
        &syn::parse_derive_input(&input.to_string()).unwrap()).parse().unwrap()
}
