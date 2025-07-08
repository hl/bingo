//! Attribute macro for performance tests.

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn performance_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let _attrs = attr.to_string();

    let test_name = &input.sig.ident;

    let expanded = quote! {
        #[test]
        fn #test_name() {
            let config = bingo_core::performance_config::PerformanceConfig::detect_environment();

            // For now, we just run the test.
            #input
        }
    };

    TokenStream::from(expanded)
}
