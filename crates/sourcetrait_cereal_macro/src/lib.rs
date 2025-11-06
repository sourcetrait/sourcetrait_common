use proc_macro;
use proc_macro2;
use quote::quote;
use syn::{self, parse::Parser};

#[proc_macro_attribute]
pub fn derived(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);
    match parse_derived(attr, item) {
        Ok(token_stream) => proc_macro::TokenStream::from(token_stream),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error())
    }
}

fn make_derives(_target: &syn::Path, _target_generics: &syn::Generics, cereal_attr: &CerealAttr) -> syn::Result<proc_macro2::TokenStream> {
    let derive_bitcode = match cereal_attr.bitcode.derive {
        true => Some(quote! { ::bitcode::Encode, ::bitcode::Decode, }),
        false => None,
    };
    let derive_debug = match cereal_attr.debug.derive {
        true => Some(quote! { ::std::fmt::Debug, }),
        false => None,
    };
    let derive_clone = match cereal_attr.clone.derive {
        true => Some(quote! { ::core::clone::Clone, }),
        false => None,
    };
    let derive_copy = match cereal_attr.copy.derive {
        true => Some(quote! { ::core::marker::Copy, }),
        false => None,
    };
    let derive_partial_eq = match cereal_attr.partial_eq.derive {
        true => Some(quote! { ::core::cmp::PartialEq, }),
        false => None,
    };
    let derive_eq = match cereal_attr.eq.derive {
        true => Some(quote! { ::core::cmp::Eq, }),
        false => None,
    };
    let derive_hash = match cereal_attr.hash.derive {
        true => Some(quote! { ::std::hash::Hash, }),
        false => None,
    };
    let derive_rkyv = match cereal_attr.rkyv.derive {
        true => Some(quote! { ::rkyv::Archive, ::rkyv::Serialize, ::rkyv::Deserialize, }),
        false => None,
    };
    let derive_serde = match cereal_attr.serde.derive {
        true => Some(quote! { ::serde::Serialize, ::serde::Deserialize, }),
        false => None,
    };
    
    let output = quote! {
        #[derive(
            #derive_debug
            #derive_clone
            #derive_copy
            #derive_partial_eq
            #derive_eq
            #derive_hash
            #derive_serde
            #derive_bitcode
            #derive_rkyv
        )]
    };
    
    Ok(output)
}

fn make_imps(target: &syn::Path, target_generics: &syn::Generics, cereal_attr: &CerealAttr) -> syn::Result<proc_macro2::TokenStream> {
    let (impl_generics, type_generics, where_clause) = target_generics.split_for_impl();
    
    let data_imp = match cereal_attr.conforms_data_trait() {
        false => None,
        true => Some(quote! {
            impl #impl_generics ::sourcetrait_cereal::Data for #target #type_generics #where_clause {}
        }),
    };
    
    let data_copy_imp = match cereal_attr.conforms_data_copy_trait() {
        false => None,
        true => Some(quote! {
            impl #impl_generics ::sourcetrait_cereal::DataCopy for #target #type_generics #where_clause {}
        }),
    };
    
    let data_eq_imp = match cereal_attr.conforms_data_eq_trait() {
        false => None,
        true => Some(quote! {
            impl #impl_generics ::sourcetrait_cereal::DataEq for #target #type_generics #where_clause {}
        }),
    };
    
    let data_copy_eq_imp = match cereal_attr.conforms_data_copy_eq_trait() {
        false => None,
        true => Some(quote! {
            impl #impl_generics ::sourcetrait_cereal::DataCopyEq for #target #type_generics #where_clause {}
        }),
    };
    
    Ok(quote! {
        #data_imp
        #data_copy_imp
        #data_eq_imp
        #data_copy_eq_imp
    })
}

fn parse_derived(attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let item = syn::parse2::<syn::DeriveInput>(item)?;
    let cereal_attr = parse_attr(attr)?;
    let target = syn::Path::from(item.ident.clone());
    let target_generics = &item.generics;
    let derives = make_derives(&target, target_generics, &cereal_attr)?;
    let imps = make_imps(&target, target_generics, &cereal_attr)?;
    
    Ok(quote! {
        #derives
        #item
        #imps
    })
}

struct CerealAttr {
    impl_data: bool,
    impl_data_copy: bool,
    impl_data_eq: bool,
    impl_data_copy_eq: bool,
    bitcode: AttrOpt,
    clone: AttrOpt,
    copy: AttrOpt,
    debug: AttrOpt,
    hash: AttrOpt,
    partial_eq: AttrOpt,
    eq: AttrOpt,
    rkyv: AttrOpt,
    serde: AttrOpt,
}

impl CerealAttr {
    fn conforms_data_trait(&self) -> bool {
        self.impl_data
        && self.clone.is && self.debug.is && self.partial_eq.is
        && self.hash.is && self.rkyv.is && self.serde.is && self.bitcode.is
    }
    
    fn conforms_data_eq_trait(&self) -> bool {
        self.impl_data_eq
        && self.conforms_data_trait()
        && self.eq.is
    }
    
    fn conforms_data_copy_trait(&self) -> bool {
        self.impl_data_copy
        && self.conforms_data_trait()
        && self.copy.is
    }
    
    fn conforms_data_copy_eq_trait(&self) -> bool {
        self.impl_data_copy_eq
        && self.conforms_data_eq_trait()
        && self.copy.is
    }
}

struct AttrOpt {
    is: bool,
    derive: bool,
}

impl AttrOpt {
    const DERIVE: Self = Self { is: true, derive: true };
    const NOT: Self = Self { is: false, derive: false };
}

impl CerealAttr {
    const DEFAULT: Self = Self {
        impl_data: false,
        impl_data_copy: false,
        impl_data_eq: false,
        impl_data_copy_eq: false,
        bitcode: AttrOpt::DERIVE,
        clone: AttrOpt::DERIVE,
        copy: AttrOpt::NOT,
        debug: AttrOpt::DERIVE,
        hash: AttrOpt::DERIVE,
        partial_eq: AttrOpt::DERIVE,
        eq: AttrOpt::NOT,
        rkyv: AttrOpt::DERIVE,
        serde: AttrOpt::DERIVE,
    };
}

fn parse_attr(attrs: proc_macro2::TokenStream) -> syn::Result<CerealAttr> {
    const HAS: &'static str = "has";
    const NOT: &'static str = "not";
    
    let metas: syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>
        = syn::punctuated::Punctuated::parse_terminated.parse2(attrs)?;
    
    let mut cereal_attrs = CerealAttr::DEFAULT;
    for meta in metas {
        let ident = meta.path().require_ident()?;
        if ident == HAS {
            meta.require_list()?.parse_nested_meta(|meta_has| {
                let ident_has = meta_has.path.require_ident()?;
                if ident_has == "Debug" {
                    cereal_attrs.debug.derive = false;
                } else if ident_has == "Clone" {
                    cereal_attrs.clone.derive = false;
                } else if ident_has == "Copy" {
                    cereal_attrs.copy.derive = false;
                } else if ident_has == "Hash" {
                    cereal_attrs.hash.derive = false;
                } else if ident_has == "PartialEq" {
                    cereal_attrs.partial_eq.derive = false;
                } else if ident_has == "Rkyv" {
                    cereal_attrs.rkyv.derive = false;
                } else if ident_has == "Serde" {
                    cereal_attrs.serde.derive = false;
                } else if ident_has == "Bitcode" {
                    cereal_attrs.bitcode.derive = false;
                } else {
                    return Err(meta_has.error("unknown cereal attribute"));
                }
                
                Ok(())
            })?;
        } else if ident == NOT {
            meta.require_list()?.parse_nested_meta(|meta_not| {
                let ident_not = meta_not.path.require_ident()?;
                if ident_not == "Debug" {
                    cereal_attrs.debug = AttrOpt::NOT;
                } else if ident_not == "Clone" {
                    cereal_attrs.clone = AttrOpt::NOT; 
                } else if ident_not == "Copy" {
                    cereal_attrs.copy = AttrOpt::NOT;
                } else if ident_not == "Hash" {
                    cereal_attrs.hash = AttrOpt::NOT;
                } else if ident_not == "PartialEq" {
                    cereal_attrs.partial_eq = AttrOpt::NOT;
                } else if ident_not == "Rkyv" {
                    cereal_attrs.rkyv = AttrOpt::NOT;
                } else if ident_not == "Serde" {
                    cereal_attrs.serde = AttrOpt::NOT;
                } else if ident_not == "Bitcode" {
                    cereal_attrs.bitcode = AttrOpt::NOT;
                } else {
                    return Err(meta_not.error("unknown cereal attribute"));
                }
                
                Ok(())
            })?;
        } else if ident == "Eq" {
            cereal_attrs.eq = AttrOpt::DERIVE;
        } else if ident == "Copy" {
            cereal_attrs.copy = AttrOpt::DERIVE;
        } else if ident == "Data" {
            cereal_attrs.impl_data = true;
            cereal_attrs.impl_data_copy = true;
            cereal_attrs.impl_data_eq = true;
            cereal_attrs.impl_data_copy_eq = true;
        } else {
            return Err(syn::Error::new_spanned(ident, "unknown cereal attribute"));
        }
    }
    
    Ok(cereal_attrs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    //use pretty_assertions::assert_eq;
    
    #[test]
    fn test_derive_default() {
        let item = quote! {
            struct TestStruct {
                foo: bool,
            }
        };
        let attr = quote! {};
        
        parse_derived(attr, item).unwrap();
    }
    
    #[test]
    fn test_derive_has() {
        let item = quote! {
            struct TestStruct {
                foo: bool,
            }
        };
        let attr = quote! { has(Debug, PartialEq) };
        
        parse_derived(attr, item).unwrap();
    }
    
    #[test]
    fn test_derive_not() {
        let item = quote! {
            struct TestStruct {
                foo: bool,
            }
        };
        let attr = quote! { not(Debug, Serde) };
        
        parse_derived(attr, item).unwrap();
    }
}
