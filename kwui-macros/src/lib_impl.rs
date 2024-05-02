use quote::quote;
use syn::{parse_macro_input, ItemFn};

pub(crate) fn main_impl(_attrs: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if cfg!(not(target_os = "android")) {
        input.into()
    } else {
        let item_fn = parse_macro_input!(input as ItemFn);      
        quote! {
            (#item_fn.attrs)*
            #[no_mangle]
            pub extern "C" fn kwui_main(argc: std::os::raw::c_int, argv: *mut *mut std::os::raw::c_char) -> std::os::raw::c_int {
                let _ = #item_fn();
                0
            }
        }.into()
        // item_fn.into_token_stream().into()
    }
}
