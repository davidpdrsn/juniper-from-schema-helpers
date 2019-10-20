use super::{inner_most_type_name, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    Ident, Token, TypePath,
};

pub fn expand(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = or_return_compile_error!(syn::parse::<Input>(input));
    or_return_compile_error!(input.expand()).into()
}

#[derive(Debug)]
struct Input {
    name: Ident,
    ty: TypePath,
}

impl Input {
    fn expand(&self) -> Result<TokenStream> {
        let name = &self.name;
        let resolver_method_name = format_ident!("field_{}", self.name);
        let inner_ty = inner_most_type_name(&self.ty)?;
        let ty = &self.ty;

        let code = quote! {
            fn #resolver_method_name(
                &self,
                executor: &juniper::Executor<'_, Context>,
                trail: &juniper_from_schema::QueryTrail<'_, #inner_ty, juniper_from_schema::Walked>,
            ) -> juniper::FieldResult<&#ty> {
                Ok(self.#name.try_unwrap()?)
            }
        };

        Ok(code)
    }
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![->]>()?;
        let ty = input.parse::<TypePath>()?;

        Ok(Input { name, ty })
    }
}
