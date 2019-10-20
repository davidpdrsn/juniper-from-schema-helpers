use super::{inner_most_type_name, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Token, TypePath,
};

pub fn expand(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = or_return_compile_error!(syn::parse::<FieldInput>(input));
    or_return_compile_error!(input.expand()).into()
}

struct FieldInput {
    key_path: Punctuated<Ident, Token![.]>,
    ty: TypePath,
    force_as_scalar: bool,
}

impl FieldInput {
    fn expand(&self) -> Result<TokenStream> {
        if self.is_scalar()? || self.force_as_scalar {
            Ok(self.expand_to_scalar())
        } else {
            Ok(self.expand_to_type()?)
        }
    }

    fn expand_to_scalar(&self) -> TokenStream {
        let resolver_method_name = self.resolver_method_name();
        let ty = &self.ty;
        let key_path = &self.key_path;

        quote! {
            fn #resolver_method_name(
                &self,
                _: &juniper::Executor<'_, Context>,
            ) -> juniper::FieldResult<&#ty> {
                Ok(&self.#key_path)
            }
        }
    }

    fn expand_to_type(&self) -> Result<TokenStream> {
        let resolver_method_name = self.resolver_method_name();
        let ty = &self.ty;
        let key_path = &self.key_path;
        let inner_ty = inner_most_type_name(&self.ty)?;

        let code = quote! {
            fn #resolver_method_name(
                &self,
                _: &juniper::Executor<'_, Context>,
                _: &juniper_from_schema::QueryTrail<'_, #inner_ty, juniper_from_schema::Walked>,
            ) -> juniper::FieldResult<&#ty> {
                Ok(&self.#key_path)
            }
        };
        Ok(code)
    }

    fn resolver_method_name(&self) -> Ident {
        format_ident!("field_{}", self.key_path.last().unwrap())
    }

    fn is_scalar(&self) -> Result<bool> {
        let inner = inner_most_type_name(&self.ty)?;
        match inner.to_string().as_str() {
            "String" | "i32" | "f64" | "bool" | "ID" => Ok(true),
            _ => Ok(false),
        }
    }
}

mod kw {
    syn::custom_keyword!(scalar);
}

impl Parse for FieldInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key_path = Punctuated::parse_separated_nonempty(input)?;
        input.parse::<Token![-]>()?;
        input.parse::<Token![>]>()?;
        let ty = input.parse::<TypePath>()?;

        let force_as_scalar = if input.peek(Token![as]) {
            input.parse::<Token![as]>()?;
            input.parse::<kw::scalar>()?;
            true
        } else {
            false
        };

        Ok(FieldInput {
            key_path,
            ty,
            force_as_scalar,
        })
    }
}
