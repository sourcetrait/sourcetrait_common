use proc_macro;
use proc_macro2;
use quote::{quote, ToTokens};
use syn::{self, parse::Parse, spanned::Spanned, Meta, MetaList};

#[derive(Clone, Copy)]
enum MacroKind {
    Tested,
    Benched
}

impl MacroKind {
    fn std_ident(&self) -> &'static str {
        match self {
            MacroKind::Tested => "test",
            MacroKind::Benched => "bench"
        }
    }
    
    fn std_attr(&self) -> proc_macro2::TokenStream {
        match self {
            MacroKind::Tested => quote! { #[test] },
            MacroKind::Benched => quote! { #[bench] }, 
        }
    }
}

#[derive(Debug)]
struct MacroAttr {
    tokio: Option<TokioMeta>,
}

impl MacroAttr {
    fn cfg_tokens(&self) -> Option<proc_macro2::TokenStream> {
        match &self.tokio {
            Some(tokio_meta) => tokio_meta.cfg_tokens(),
            None => None,
        }
    }
}

impl quote::ToTokens for MacroAttr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(tokio_meta) = &self.tokio {
            tokio_meta.to_tokens(tokens);
        }
    }
}


#[derive(Debug, Default)]
struct TokioMeta {
    cfg_unstable: bool, 
    inner_meta: proc_macro2::TokenStream,
}

impl TokioMeta {
    const TOKIO: &'static str = "tokio";
    const UNSTABLE: &'static str = "unstable";

    fn parse_list(list: MetaList) -> syn::Result<Self> {
        let mut cfg_unstable = None;
        let mut inner_meta: Vec<_> = vec![];
        
        let list = list.parse_args_with(syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)?;
        for meta in list {
            match meta {
                Meta::Path(p) if p.is_ident(Self::UNSTABLE) => {
                    if cfg_unstable.is_some() {
                        return Err(syn::Error::new(p.span(), "duplicate argument"));
                    }
                    
                    cfg_unstable = Some(true);
                },
                Meta::Path(p) => { inner_meta.push(p.to_token_stream()); },
                Meta::NameValue(nv) => { inner_meta.push(nv.to_token_stream()); },
                Meta::List(l) => { inner_meta.push(l.to_token_stream()); },
            }
        }
        
        Ok(Self {
            cfg_unstable: cfg_unstable.unwrap_or(false),
            inner_meta: quote! { #(#inner_meta),* }
        })
    }
    
    fn cfg_tokens(&self) -> Option<proc_macro2::TokenStream> {
        match self.cfg_unstable {
            true => Some(quote! { #[cfg(tokio_unstable)] }),
            false => None,
        }
    }
}

impl quote::ToTokens for TokioMeta {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        
        let inner = &self.inner_meta;
        match inner.is_empty() {
            true => tokens.extend(quote! { #[tokio::test] }),
            //false => tokens.extend(quote! { #[tokio::test] }),
            false => tokens.extend(quote! { #[tokio::test(#inner)] }),
        }
    }
}

impl Parse for MacroAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self { tokio: None });
        }
        
        #[allow(unused_assignments)]
        let mut tokio = None;
        match input.parse::<syn::Meta>()? {
            syn::Meta::Path(path) => {
                let name = path.get_ident()
                    .ok_or_else(|| input.error("expected ident"))?
                    .to_string();
                
                match name.as_str() {
                    TokioMeta::TOKIO => { tokio = Some(TokioMeta::default()); },
                    _ => return Err(input.error("unknown identified")),
                }
            },
            syn::Meta::List(list) => tokio = Some(TokioMeta::parse_list(list)?),
            syn::Meta::NameValue(_) => return Err(input.error("expected arguments")),
        }
        
        Ok(Self {
            tokio,
        })
    }
}

#[proc_macro_attribute]
pub fn tested(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse_macro(MacroKind::Tested, proc_macro2::TokenStream::from(attr), proc_macro2::TokenStream::from(item)) {
        Ok(token_stream) => proc_macro::TokenStream::from(token_stream),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error())
    }
}

#[proc_macro_attribute]
pub fn benched(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse_macro(MacroKind::Benched, proc_macro2::TokenStream::from(attr), proc_macro2::TokenStream::from(item)) {
        Ok(token_stream) => proc_macro::TokenStream::from(token_stream),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error())
    }
}

fn parse_macro(macro_kind: MacroKind, attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let attr: MacroAttr = syn::parse2(attr)?;
    let item: syn::ItemFn = syn::parse2(item)?;
    let std_ident = macro_kind.std_ident();
    
    // append a #[test] or #[bench] if there isn't one
    let needs_attr = {
        item.attrs.iter()
            .find(|attr| attr.path().is_ident(std_ident)).is_none()
        && attr.tokio.is_none()
    };
    
    let test_attr = match needs_attr {
        true => macro_kind.std_attr(),
        false => quote!{},
    };

    // append a #[named] if there isn't one
    let named_attr = if let Some(_)= item.attrs.iter().find(|attr| attr.path().is_ident("named")) {
        quote!{}
    } else {
        quote!{ #[named] }
    };
    
    let cfg = attr.cfg_tokens();

    let output = quote! {
        #cfg
        #attr
        #named_attr
        #test_attr
        #item
    };

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tested() {
        let attr = quote::quote!{};

        let item = quote::quote!{
            fn test_one() {}
        };

        parse_macro(MacroKind::Tested, attr, item).unwrap();
    }
    
    #[test]
    fn test_tested_tokio() {
        let attr = quote::quote!{ tokio };

        let item = quote::quote!{
            fn test_one() {}
        };

        parse_macro(MacroKind::Tested, attr, item).unwrap();
    }
    
    #[test]
    fn test_benched() {
        let attr = quote::quote!{};

        let item = quote::quote!{
            fn bench_one() {}
        };

        parse_macro(MacroKind::Benched, attr, item).unwrap();
    }
}
