use proc_macro;
use proc_macro2;
use quote::quote;
use syn;

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

fn parse_macro(macro_kind: MacroKind, _attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let item: syn::ItemFn = syn::parse2(item)?;
    let std_ident = macro_kind.std_ident();

    // append a #[test] or #[bench] if there isn't one
    let test_attr = if let Some(_)= item.attrs.iter().find(|attr| attr.path().is_ident(std_ident)) {
        quote!{}
    } else {
        macro_kind.std_attr()
    };

    // append a #[named] if there isn't one
    let named_attr = if let Some(_)= item.attrs.iter().find(|attr| attr.path().is_ident("named")) {
        quote!{}
    } else {
        quote!{ #[named] }
    };

    let output = quote! {
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
        let attr = quote::quote!{
            #[tested]
        };

        let item = quote::quote!{
            fn test_one() {}
        };

        parse_macro(MacroKind::Tested, attr, item).unwrap();
    }
    
    #[test]
    fn test_benched() {
        let attr = quote::quote!{
            #[benched]
        };

        let item = quote::quote!{
            fn bench_one() {}
        };

        parse_macro(MacroKind::Benched, attr, item).unwrap();
    }
}
