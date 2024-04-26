use std::rc::Rc;

use proc_macro2::TokenStream;
use quote::ToTokens;

#[derive(Clone)]
pub struct WithTokens<T> {
    pub tokens: Rc<Box<dyn ToTokens>>,
    pub inner: T,
}

impl<T: std::fmt::Debug> std::fmt::Debug for WithTokens<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WithTokens")
            .field("tokens", &"...")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T> WithTokens<T> {
    pub fn new(tokens: Box<dyn quote::ToTokens>, val: T) -> Self {
        Self {
            tokens: Rc::new(tokens),
            inner: val,
        }
    }

    pub fn map<F, O: FnOnce(&T) -> F>(&self, op: O) -> WithTokens<F> {
        WithTokens {
            tokens: self.tokens.clone(),
            inner: op(&self.inner),
        }
    }
}

impl<T> ToTokens for WithTokens<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.tokens.to_tokens(tokens)
    }
}

macro_rules! spanned_err {
    ($span:expr, $msg:tt) => {{
        Err(::syn::Error::new_spanned($span, format!($msg)))
    }};
    ($span:tt, $msg:tt, $($arg:tt)*) => {{
        Err(::syn::Error::new_spanned($span, format!($msg, $($arg)*)))
    }};
}
pub(crate) use spanned_err;

macro_rules! spanned_error {
    ($span:expr, $msg:tt) => {{
        ::syn::Error::new_spanned($span, format!($msg))
    }};
    ($span:tt, $msg:tt, $($arg:tt)*) => {{
        ::syn::Error::new_spanned($span, format!($msg, $($arg)*))
    }};
}
pub(crate) use spanned_error;

macro_rules! attach_spanned_error {
    ($x:tt, $span:expr, $msg:tt) => {{
        {
            $x.combine(::syn::Error::new_spanned($span, format!($msg)));
            $x
        }
    }};
    ($x:tt, $span:tt, $msg:tt, $($arg:tt)*) => {{
        {
            $x.combine(::syn::Error::new_spanned($span, format!($msg, $($arg)*)));
            $x
        }
    }};
}
pub(crate) use attach_spanned_error;
use syn::Ident;

pub fn prefix_ident(p: &str, i: &Ident) -> Ident {
    syn::Ident::new(&format!("{p}{}", i), i.span())
}
