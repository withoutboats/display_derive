extern crate proc_macro;
extern crate syn;

#[macro_use] extern crate synstructure;
#[macro_use] extern crate quote;

decl_derive!([Display, attributes(display)] => display_derive);

fn display_derive(s: synstructure::Structure) -> quote::Tokens {
    #[cfg(feature = "std")]
    let display = display_body(&s).map(|display_body| {
        s.bound_impl("::std::fmt::Display", quote! {
            #[allow(unreachable_code)]
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match *self { #display_body }
                ::std::result::Result::Ok(())
            }
        })
    });

    #[cfg(not(feature = "std"))]
    let display = display_body(&s).map(|display_body| {
        s.bound_impl("::core::fmt::Display", quote! {
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

fn display_body(s: &synstructure::Structure) -> Option<quote::Tokens> {
    Some(s.each_variant(|v| {
        let msg = match find_display_msg(&v.ast().attrs) {
            Some(msg) => msg,
            None => {
                let variant_name = v.ast().ident.as_ref();
                return quote!( return write!(f, #variant_name));
            }
        };
        if msg.is_empty() {
            panic!("Expected at least one argument to display attribute");
        }

        let s = match msg[0] {
            syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ref i, ref lit)) if i == "fmt" => {
                lit.clone()
            }
            _ => panic!("Display attribute must begin `fmt = \"\"` to control the Display message."),
        };
        let args = msg[1..].iter().map(|arg| match *arg {
            syn::NestedMetaItem::Literal(syn::Lit::Int(i, _)) => {
                let bi = &v.bindings()[i as usize];
                quote!(#bi)
            }
            syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref id)) => {
                if id.as_ref().starts_with("_") {
                    if let Ok(idx) = id.as_ref()[1..].parse::<usize>() {
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

fn find_display_msg(attrs: &[syn::Attribute]) -> Option<&[syn::NestedMetaItem]> {
    let mut display_msg = None;
    for attr in attrs {
        if attr.name() == "display" {
            if display_msg.is_some() {
                panic!("Cannot have two display attributes")
            } else {
                if let syn::MetaItem::List(_, ref list)  = attr.value {
                    display_msg = Some(&list[..]);
                } else {
                    panic!("Display attribute must take a list in parentheses")
                }
            }
        }
    }
    display_msg
}
