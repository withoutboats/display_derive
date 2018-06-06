extern crate proc_macro;

#[macro_use] extern crate syn;
#[macro_use] extern crate synstructure;
#[macro_use] extern crate quote;
extern crate proc_macro2;

use syn::punctuated::Punctuated;
use proc_macro2::TokenStream;

decl_derive!([Display, attributes(display)] => display_derive);

fn display_derive(s: synstructure::Structure) -> TokenStream {
    #[cfg(feature = "std")]
    let display = display_body(&s).map(|display_body| {
        s.bound_impl(quote!(::std::fmt::Display), quote! {
            #[allow(unreachable_code)]
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match *self { #display_body }
                ::std::result::Result::Ok(())
            }
        })
    });

    #[cfg(not(feature = "std"))]
    let display = display_body(&s).map(|display_body| {
        s.bound_impl(quote!(::core::fmt::Display), quote! {
            #[allow(unreachable_code)]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self { #display_body }
                ::std::result::Result::Ok(())
            }
        })
    });

    quote! {
        #display
    }
}

fn display_body(s: &synstructure::Structure) -> Option<TokenStream> {
    let mut msgs = s.variants().iter().map(|v| find_display_msg(&v.ast().attrs));
    if msgs.all(|msg| msg.is_none()) { return None; }

    Some(s.each_variant(|v| {
        let msg = find_display_msg(&v.ast().attrs).expect("All variants must have display attribute.");
        if msg.is_empty() {
            panic!("Expected at least one argument to display attribute");
        }

        let s = match msg[0] {
            syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue { ref ident, ref lit, .. })) if ident == "fmt" => {
                lit.clone()
            }
            _ => panic!("Display attribute must begin `fmt = \"\"` to control the Display message."),
        };
        let args = msg.iter().skip(1).map(|arg| match *arg {
            syn::NestedMeta::Literal(syn::Lit::Int(ref lit_int)) => {
                let bi = &v.bindings()[lit_int.value() as usize];
                quote!(#bi)
            }
            syn::NestedMeta::Meta(syn::Meta::Word(ref id)) => {
                if id.to_string().starts_with("_") {
                    if let Ok(idx) = id.to_string()[1..].parse::<usize>() {
                        let bi = &v.bindings()[idx];
                        return quote!(#bi)
                    }
                }
                for bi in v.bindings() {
                    if bi.ast().ident.as_ref() == Some(id) {
                        return quote!(#bi);
                    }
                }
                panic!("Couldn't find a field with this name!");
            }
            _ => panic!("Invalid argument to display attribute!"),
        });

        quote! {
            return write!(f, #s #(, #args)*)
        }
    }))
}

fn find_display_msg(attrs: &[syn::Attribute]) -> Option<Punctuated<syn::NestedMeta, Token![,]>> {
    let mut display_msg = None;

    for attr in attrs {
        if attr.path == parse_quote!(display) {
            if display_msg.is_some() {
                panic!("Cannot have two display attributes")
            } else {
                if let Some(syn::Meta::List(syn::MetaList { nested, .. })) = attr.interpret_meta() {
                    display_msg = Some(nested);
                } else {
                    panic!("Display attribute must take a list in parentheses")
                }
            }
        }
    }
    display_msg
}
