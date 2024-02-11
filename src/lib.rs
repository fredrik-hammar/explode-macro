use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Ident, LitByte, LitByteStr, LitChar, LitStr};

/// # Example
/// ```
/// use explode::explode;
///
/// let expected: [char; 5] = ['h', 'e', 'l', 'l', 'o'];
/// assert_eq!(expected, explode!(hello));
/// assert_eq!(expected, explode!("hello"));
///
/// let expected: [u8; 5] = [b'h', b'e', b'l', b'l', b'o'];
/// assert_eq!(expected, explode!(b"hello"));
/// ```
#[proc_macro]
pub fn explode(input: TokenStream) -> TokenStream {
    let input = match syn::parse::<Input>(input) {
        Ok(input) => input,
        Err(e) => {
            return e.into_compile_error().into();
        }
    };
    let exploded = input.explode();
    quote!([ #(#exploded),* ]).into()
}

enum Input {
    Ident(Ident),
    LitStr(LitStr),
    LitByteStr(LitByteStr),
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitStr) {
            input.parse().map(Input::LitStr)
        } else if lookahead.peek(Ident) {
            input.parse().map(Input::Ident)
        } else if lookahead.peek(LitByteStr) {
            input.parse().map(Input::LitByteStr)
        } else {
            Err(lookahead.error())
        }
    }
}

trait Explode {
    type Into: ToTokens;
    fn explode(&self) -> Vec<Self::Into>;
}

impl Explode for Input {
    type Into = Exploded;

    fn explode(&self) -> Vec<Self::Into> {
        match self {
            Input::Ident(ident) => ident.explode().into_iter().map(Into::into).collect(),
            Input::LitStr(lit_str) => lit_str.explode().into_iter().map(Into::into).collect(),
            Input::LitByteStr(lit_byte_str) => {
                lit_byte_str.explode().into_iter().map(Into::into).collect()
            }
        }
    }
}

enum Exploded {
    LitChar(LitChar),
    LitByte(LitByte),
}

impl ToTokens for Exploded {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append(match self {
            Exploded::LitChar(v) => v.token(),
            Exploded::LitByte(v) => v.token(),
        });
    }
}

impl From<LitChar> for Exploded {
    fn from(value: LitChar) -> Self {
        Exploded::LitChar(value)
    }
}

impl From<LitByte> for Exploded {
    fn from(value: LitByte) -> Self {
        Exploded::LitByte(value)
    }
}

impl Explode for Ident {
    type Into = LitChar;
    fn explode(&self) -> Vec<LitChar> {
        self.to_string().explode_in(self.span())
    }
}

impl Explode for LitStr {
    type Into = LitChar;
    fn explode(&self) -> Vec<LitChar> {
        self.value().explode_in(self.span())
    }
}

impl Explode for LitByteStr {
    type Into = LitByte;
    fn explode(&self) -> Vec<LitByte> {
        ByteString(self.value()).explode_in(self.span())
    }
}

trait ExplodeIn {
    type Into: ToTokens;
    fn explode_in(&self, span: Span) -> Vec<Self::Into>;
}

impl ExplodeIn for String {
    type Into = LitChar;
    fn explode_in(&self, span: Span) -> Vec<LitChar> {
        let to_lit_char = |ch| LitChar::new(ch, span);
        self.chars().map(to_lit_char).collect()
    }
}

struct ByteString(Vec<u8>);
impl ExplodeIn for ByteString {
    type Into = LitByte;
    fn explode_in(&self, span: Span) -> Vec<LitByte> {
        let to_lit_byte = |&ch| LitByte::new(ch, span);
        self.0.iter().map(to_lit_byte).collect()
    }
}
