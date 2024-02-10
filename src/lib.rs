use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

/// # Example
/// ```
/// use explode::explode;
///
/// let a: [char; 5] = ['h', 'e', 'l', 'l', 'o'];
/// let b = explode!(hello);
/// assert_eq!(a, b);
/// ```
#[proc_macro]
pub fn explode(input: TokenStream) -> TokenStream {
    let input = match syn::parse::<Input>(input) {
        Ok(input) => input,
        Err(e) => {
            return e.into_compile_error().into();
        }
    };
    let str = input.to_string();
    let chars = str.chars();
    quote!([ #(#chars),* ]).into()
}

enum Input {
    Ident(Ident),
    LitStr(LitStr),
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitStr) {
            input.parse().map(Input::LitStr)
        } else if lookahead.peek(Ident) {
            input.parse().map(Input::Ident)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToString for Input {
    fn to_string(&self) -> String {
        match self {
            Input::Ident(ident) => ident.to_string(),
            Input::LitStr(lit_str) => lit_str.value(),
        }
    }
}
