extern crate proc_macro;

use proc_macro::TokenStream;

use extended_einsum_macros_internal::ein_internal;

#[proc_macro]
pub fn ein(input: TokenStream) -> TokenStream {
    let output =
        ein_internal(proc_macro2::TokenStream::from(input));

    proc_macro::TokenStream::from(output)
}
