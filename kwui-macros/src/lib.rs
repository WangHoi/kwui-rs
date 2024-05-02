use proc_macro::TokenStream;

mod lib_impl;

#[proc_macro_attribute]
pub fn main(attrs: TokenStream, input: TokenStream) -> TokenStream {
    lib_impl::main_impl(attrs, input)
}
