use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_quote, ItemTrait, Result};

/// When added to a trait declaration, generates the impls required to find a resource that implements a specific trait.
#[proc_macro_attribute]
pub fn trait_resource(attr: TokenStream, item: TokenStream) -> TokenStream {
    impl_trait_resource(attr, item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn impl_trait_resource(arg: TokenStream, item: TokenStream) -> Result<TokenStream2> {
    let _ = arg;
    let trait_definition = syn::parse::<ItemTrait>(item)?;
    let trait_name = trait_definition.ident.clone();

    let mut impl_generics_list = vec![];
    let mut trait_generics_list = vec![];
    let where_clause = trait_definition.generics.where_clause.clone();

    for param in &trait_definition.generics.params {
        impl_generics_list.push(param.clone());
        match param {
            syn::GenericParam::Type(param) => {
                let ident = &param.ident;
                trait_generics_list.push(quote! { #ident });
            }
            syn::GenericParam::Lifetime(param) => {
                let ident = &param.lifetime;
                trait_generics_list.push(quote! { #ident });
            }
            syn::GenericParam::Const(param) => {
                let ident = &param.ident;
                trait_generics_list.push(quote! { #ident });
            }
        }
    }

    let impl_generics = quote! { <#( #impl_generics_list ,)*> };
    let trait_generics = quote! { <#( #trait_generics_list ,)*> };

    let trait_object = quote! { dyn #trait_name #trait_generics };

    let my_crate = proc_macro_crate::crate_name("bevy-trait-resource").unwrap();
    let my_crate = match my_crate {
        proc_macro_crate::FoundCrate::Itself => quote! { crate },
        proc_macro_crate::FoundCrate::Name(x) => {
            let ident = quote::format_ident!("{x}");
            quote! { #ident }
        }
    };

    let imports = quote! { #my_crate::imports };

    let trait_resource = quote! { #my_crate::TraitResource };

    let mut marker_impl_generics_list = impl_generics_list.clone();
    marker_impl_generics_list.push(parse_quote!(__Resource: #trait_name #trait_generics + #imports::Resource));

    let marker_impl_generics = quote! { <#( #marker_impl_generics_list ,)*> };

    let marker_impl_code = quote! {
        impl #impl_generics #trait_resource for #trait_object #where_clause {}

        impl #marker_impl_generics #my_crate::TraitResourceMarker::<#trait_object> for (__Resource,)
        #where_clause
        {
            type Covered = __Resource;
            fn cast(ptr: *mut u8) -> *mut #trait_object {
                ptr as *mut __Resource as *mut _
            }
        }
    };

    Ok(quote! {
        #trait_definition
        #marker_impl_code
    })
}
