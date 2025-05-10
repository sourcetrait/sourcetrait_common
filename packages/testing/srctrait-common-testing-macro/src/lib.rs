use proc_macro;
use proc_macro2;
use quote::quote;
use syn;

#[proc_macro_attribute]
pub fn tested(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse_tested_macro(proc_macro2::TokenStream::from(attr), proc_macro2::TokenStream::from(item)) {
        Ok(token_stream) => proc_macro::TokenStream::from(token_stream),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error())
    }
}

fn parse_tested_macro(_attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let item: syn::ItemFn = syn::parse2(item)?;

    // append a #[test] if there isn't one
    let test_attr = if let Some(_)= item.attrs.iter().find(|attr| attr.path().is_ident("test")) {
        quote!{}
    } else {
        quote!{ #[test] }
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
    fn test_attr() {
        let test_one_attr = quote::quote!{
            #[tested]
        };

        let test_one_item = quote::quote!{
            fn test_one() {}
        };

        parse_tested_macro(test_one_attr, test_one_item).unwrap();
    }
}
