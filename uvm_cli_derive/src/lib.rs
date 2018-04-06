extern crate proc_macro;
extern crate proc_macro2;

#[macro_use]
extern crate quote;

extern crate serde_derive_internals as internals;
extern crate syn;

use proc_macro::TokenStream;
use syn::DeriveInput;
use quote::Tokens;

#[proc_macro_derive(UVMOptions)]
pub fn uvm_options_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    match uvm::expand_uvm_options(&input) {
        Ok(expanded) => expanded.into(),
        Err(msg) => panic!(msg),
    }
}

mod uvm {
    use internals::ast::{Container, Data, Field, Style, Variant};
    use internals::{attr, Ctxt};
    use proc_macro::TokenStream;
    use syn::DeriveInput;
    use syn;
    use quote::Tokens;

    pub fn expand_uvm_options(input: &DeriveInput) -> Result<Tokens, String> {
        let ctxt = Ctxt::new();
        let cont = Container::from_ast(&ctxt, input);
        precondition(&ctxt, &cont);
        try!(ctxt.check());

        let name = &input.ident;

        if let Some(ref fields) = retrieve_fields(input) {
            let fields = match *fields {
                &syn::Fields::Named(ref fields) => fields.named.iter().collect(),
                _ => Vec::new(),
            };
        };

        let implementation_block = quote! {
            impl Options for #name {
                fn verbose(&self) -> bool {
                    false
                }
            }
        };

        let generated = quote! {
            use Options;
            #implementation_block
        };

        Ok(generated)
    }

    fn retrieve_fields(input: &DeriveInput) -> Option<&syn::Fields> {
        if let syn::Data::Struct(ref data) = input.data {
            return Some(&data.fields);
        }
        None
    }

    fn precondition(cx: &Ctxt, cont: &Container) {
        match cont.attrs.identifier() {
            attr::Identifier::No => {}
            attr::Identifier::Field => {
                cx.error("field identifiers cannot be serialized");
            }
            attr::Identifier::Variant => {
                cx.error("variant identifiers cannot be serialized");
            }
        }
    }
}
