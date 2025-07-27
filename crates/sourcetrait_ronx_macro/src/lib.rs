use proc_macro;
use proc_macro2;
use quote::quote;
use syn;

#[proc_macro_derive(RonX, attributes(ronx))]
pub fn derive_ronx(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse_derive_ronx(proc_macro2::TokenStream::from(item)) {
        Ok(token_stream) => proc_macro::TokenStream::from(token_stream),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error())
    }
}

fn make_fromto_implementations(target: &syn::Path, target_generics: &syn::Generics, inlined: bool) -> syn::Result<proc_macro2::TokenStream> {
    let (impl_generics, type_generics, where_clause) = target_generics.split_for_impl();
    
    let inlined = if inlined {
        Some(quote! {
            impl #impl_generics ::sourcetrait_ronx::FromInlinedRon for #target #type_generics #where_clause {}
            impl #impl_generics ::sourcetrait_ronx::ToInlinedRon for #target #type_generics #where_clause {}
        })
    } else {
        None
    };
    
    Ok(quote! {
        impl #impl_generics ::sourcetrait_ronx::FromRon for #target #type_generics #where_clause {}
        impl #impl_generics ::sourcetrait_ronx::ToRon for #target #type_generics #where_clause {}
        #inlined
    })
}

fn parse_derive_ronx(item: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let mut input = syn::parse2::<syn::DeriveInput>(item)?;
    let target = syn::Path::from(input.ident.clone());
    let target_generics = &input.generics;
    let target_lil_ron = extract_lil_ron(&mut input.attrs)?;
    let fromto_imps = make_fromto_implementations(&target, target_generics, target_lil_ron.inlined)?;
    
    let resolver_imp = if target_lil_ron.inlined {
        Some(make_resolver_impl(&target, target_generics, &mut input.data)?)
    } else {
        None
    };
    
    Ok(quote! {
        #fromto_imps
        #resolver_imp
    })
}

#[derive(Default)]
struct LilRon {
    inlined: bool,
}

fn extract_lil_ron(item_attrs: &mut Vec<syn::Attribute>) -> syn::Result<LilRon> {
    let attrs = item_attrs.extract_if(.., |a| a.path().is_ident(LIL_RON));
    let mut lil_ron = LilRon::default();
    for attr in attrs {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(INLINED) {
                lil_ron.inlined = true;
                return Ok(());
            }
            
            Err(meta.error("unknown ronx attribute"))
        })?;
    }
    
    Ok(lil_ron)
}

fn make_resolver_impl(target: &syn::Path, target_generics: &syn::Generics, data: &mut syn::Data) -> syn::Result<proc_macro2::TokenStream> {
    let resolver_body = match data {
        syn::Data::Struct(struct_item) => {
            let mut body = vec![];
            for field in &mut struct_item.fields {
                let lil_ron = extract_lil_ron(&mut field.attrs)?;
                if lil_ron.inlined && let Some(field_ident) = field.ident.as_ref() {
                    body.push(quote! {
                        self.#field_ident.resolve_inlined_ron(state, current, config)?;
                    });
                }
            }
            
            quote! {
                #(#body)*
                Ok(())
            }
        },
        syn::Data::Enum(enum_item) => {
            let mut body = vec![];
            for variant in &mut enum_item.variants {
                let lil_ron = extract_lil_ron(&mut variant.attrs)?;
                if lil_ron.inlined && !variant.fields.is_empty() {
                    let variant_ident = &variant.ident;
                    body.push(quote! {
                        Self::#variant_ident(inner) => inner.resolve_inlined_ron(state, current, config),
                    });
                }
            }
            
            quote! {
                match self {
                    #(#body)*
                }
            }
        },
        _ => return Err(syn::Error::new(target.get_ident().expect("ident").span(), "RonX can only be derived for structs and enums")),
    };
    
    let (impl_generics, type_generics, where_clause) = target_generics.split_for_impl();
    
    Ok(quote! {
        impl #impl_generics ::sourcetrait_ronx::InlinedRonResolver for #target #type_generics #where_clause {
            fn resolve_inlined_ron(&mut self, state: &mut ::sourcetrait_ronx::InlinedRonState, current: &::std::path::Path, config: &::sourcetrait_ronx::InlinedRonConfig) -> ::sourcetrait_ronx::Result<()> {
                #resolver_body
            }
        }
    })
}

const LIL_RON: &'static str = "ronx";
const INLINED: &'static str = "inlined";

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use pretty_assertions::assert_eq;
    
    #[test]
    fn test_derive_ronx() {
        let item = quote! {
            struct TestStruct {
                foo: bool,
            }
        };
        
        let expected = quote! {
            impl ::sourcetrait_ronx::FromRon for TestStruct {}
            impl ::sourcetrait_ronx::ToRon for TestStruct {}
        };
        
        let actual = parse_derive_ronx(item).unwrap();
        assert_eq!(expected.to_string(), actual.to_string());
    }
    
    #[test]
    fn test_derive_inlined_ronx() {
        // test: inlined type
        let item = quote! {
            #[ronx(inlined)]
            struct TestStruct {
                foo: bool,
            }
        };
        
        let expected = quote! {
            impl ::sourcetrait_ronx::FromRon for TestStruct {}
            impl ::sourcetrait_ronx::ToRon for TestStruct {}
            impl ::sourcetrait_ronx::FromInlinedRon for TestStruct {}
            impl ::sourcetrait_ronx::ToInlinedRon for TestStruct {}
            
            impl ::sourcetrait_ronx::InlinedRonResolver for TestStruct {
                fn resolve_inlined_ron(&mut self, state: &mut ::sourcetrait_ronx::InlinedRonState, current: &::std::path::Path, config: &::sourcetrait_ronx::InlinedRonConfig) -> ::sourcetrait_ronx::Result<()> {
                    Ok(())
                }
            }
        };
        
        let actual = parse_derive_ronx(item).unwrap();
        assert_eq!(expected.to_string(), actual.to_string());
        
        // test: inlined fields
        let item = quote! {
            #[ronx(inlined)]
            struct TestStruct {
                foo: bool,
                #[ronx(inlined)]
                kids: Vec<InlinedRon>,
                bar: String,
                #[ronx(inlined)]
                other: OtherThing,
            }
        };
        
        let expected = quote! {
            impl ::sourcetrait_ronx::FromRon for TestStruct {}
            impl ::sourcetrait_ronx::ToRon for TestStruct {}
            impl ::sourcetrait_ronx::FromInlinedRon for TestStruct {}
            impl ::sourcetrait_ronx::ToInlinedRon for TestStruct {}
            
            impl ::sourcetrait_ronx::InlinedRonResolver for TestStruct {
                fn resolve_inlined_ron(&mut self, state: &mut ::sourcetrait_ronx::InlinedRonState, current: &::std::path::Path, config: &::sourcetrait_ronx::InlinedRonConfig) -> ::sourcetrait_ronx::Result<()> {
                    self.kids.resolve_inlined_ron(state, current, config)?;
                    self.other.resolve_inlined_ron(state, current, config)?;
                    Ok(())
                }
            }
        };
        
        let actual = parse_derive_ronx(item).unwrap();
        assert_eq!(expected.to_string(), actual.to_string());
    }
}
