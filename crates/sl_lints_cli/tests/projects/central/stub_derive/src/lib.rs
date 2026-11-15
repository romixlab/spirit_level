//! Stands for clap's `Subcommand` derive: it accepts the same attributes and expands to nothing

use proc_macro::TokenStream;

#[proc_macro_derive(Subcommand, attributes(command))]
pub fn subcommand(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}
