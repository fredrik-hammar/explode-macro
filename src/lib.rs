#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse, Ident, LitByte, LitByteStr, LitChar, LitStr, Token};

/// See [crate] level documentation.
#[proc_macro]
pub fn explode(input: TokenStream) -> TokenStream {
    let input = match parse::<Input>(input) {
        Ok(input) => input,
        Err(e) => {
            return e.into_compile_error().into();
        }
    };
    let exploded = input.explode();
    quote!([ #(#exploded),* ]).into()
}

#[proc_macro]
pub fn explode_map(input: TokenStream) -> TokenStream {
    let MapInput { op, bang, input } = match parse::<MapInput>(input) {
        Ok(input) => input,
        Err(e) => {
            return e.into_compile_error().into();
        }
    };
    let exploded = input.explode();
    quote!([ #(#op #bang ( #exploded) ),* ]).into()
}

struct MapInput {
    op: Ident,
    bang: Option<Token![!]>,
    input: Input,
}

impl parse::Parse for MapInput {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let op = input.parse()?;
        let bang = input.parse()?;
        input.parse::<Token![,]>()?;
        let input = input.parse()?;
        Ok(MapInput { op, bang, input })
    }
}

enum Input {
    Ident(Ident),
    LitStr(LitStr),
    LitByteStr(LitByteStr),
}

impl parse::Parse for Input {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
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
    type Into: ToTokens + Into<Exploded>;
    fn explode(&self) -> Vec<Self::Into>;
}

impl Explode for Input {
    type Into = Exploded;

    fn explode(&self) -> Vec<Self::Into> {
        return match self {
            Input::Ident(t) => inner(t),
            Input::LitStr(t) => inner(t),
            Input::LitByteStr(t) => inner(t),
        };
        fn inner<E: Explode>(e: &E) -> Vec<Exploded> {
            e.explode().map_collect(Into::into)
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
    fn explode_in(self, span: Span) -> Vec<Self::Into>;
}

impl ExplodeIn for String {
    type Into = LitChar;
    fn explode_in(self, span: Span) -> Vec<LitChar> {
        let to_lit_char = |ch| LitChar::new(ch, span);
        self.chars().map_collect(to_lit_char)
    }
}

struct ByteString(Vec<u8>);
impl ExplodeIn for ByteString {
    type Into = LitByte;
    fn explode_in(self, span: Span) -> Vec<LitByte> {
        let to_lit_byte = |ch| LitByte::new(ch, span);
        self.0.map_collect(to_lit_byte)
    }
}

trait MapCollect: IntoIterator + Sized {
    fn map_collect<F, B, C>(self, f: F) -> C
    where
        F: FnMut(Self::Item) -> B,
        C: FromIterator<B>,
    {
        self.into_iter().map(f).collect()
    }
}
impl<T> MapCollect for T where T: IntoIterator {}
