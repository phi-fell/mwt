mod args;

use args::Args;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

fn do_parse(mut args: Args, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let non_mut_version = args.convert_fn(false, input.clone());
    let mut_version = args.convert_fn(true, input);
    let expanded = quote! {
        #non_mut_version
        #mut_version
    };
    TokenStream::from(expanded)
}

/// Creates two copies of a function, replacing `&Mwt<T>`, and all occurrences of `mwt` in identifiers
#[proc_macro_attribute]
pub fn mwt(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    do_parse(args, input)
}

/// Creates two copies of a function, replacing `&MaybeMut<T>`, and all occurrences of `maybe_mut` in identifiers
#[proc_macro_attribute]
pub fn maybe_mut(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut args = parse_macro_input!(args as Args);
    args.set_ident_str("maybe_mut".to_owned());
    args.set_type_str("MaybeMut".to_owned());
    args.set_type_switch_str("MutOrElse".to_owned());
    args.set_ref_str("MaybeMut".to_owned());
    do_parse(args, input)
}
