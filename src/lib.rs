extern crate proc_macro;
extern crate proc_macro2;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Ident, Token, TypePath,
};

macro_rules! or_return_compile_error {
    ($e:expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => return e.to_compile_error().into(),
        }
    };
}

#[proc_macro]
pub fn field(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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

fn inner_most_type_name(ty: &TypePath) -> Result<&Ident> {
    let last_segment = ty.path.segments.last();
    let segments = last_segment.as_ref().unwrap();
    let args = &segments.arguments;

    if args.is_empty() {
        Ok(&segments.ident)
    } else if let syn::PathArguments::AngleBracketed(arguments) = &args {
        let last_arg = arguments.args.last().unwrap();
        if let syn::GenericArgument::Type(ty) = last_arg {
            match ty {
                syn::Type::Path(ty_path) => inner_most_type_name(ty_path),
                syn::Type::Reference(ty_ref) => {
                    let elem = &ty_ref.elem;
                    if let syn::Type::Path(ty) = &**elem {
                        inner_most_type_name(&ty)
                    } else {
                        error(elem, "expected type path")?
                    }
                }
                _ => error(ty, "expected type path or reference type")?,
            }
        } else {
            error(last_arg, "expected generic type argument")?
        }
    } else {
        error(
            args,
            "expected type without arguments or angle bracketed type arguments",
        )?
    }
}

fn error<T: Spanned, K>(t: T, msg: &str) -> Result<K> {
    Err(syn::Error::new(t.span(), msg))
}

impl Parse for FieldInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key_path = Punctuated::parse_separated_nonempty(input)?;
        input.parse::<Token![-]>()?;
        input.parse::<Token![>]>()?;
        let ty = input.parse::<TypePath>()?;

        let force_as_scalar = if input.peek(Token![as]) {
            input.parse::<Token![as]>()?;
            let ident = input.parse::<Ident>()?;
            if ident == "scalar" {
                true
            } else {
                error(ident, "expected `scalar`")?
            }
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

type Result<T, E = syn::Error> = std::result::Result<T, E>;

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;
    use quote::ToTokens;

    #[test]
    fn ui() {
        let t = trybuild::TestCases::new();
        t.pass("tests/compile_pass/*.rs");
    }

    #[test]
    fn test_inner_most_type() {
        let ty = syn::parse_str::<TypePath>("i32").unwrap();
        let ty = inner_most_type_name(&ty).unwrap();
        assert_eq!("i32", ty.into_token_stream().to_string());

        let ty = syn::parse_str::<TypePath>("f64").unwrap();
        let ty = inner_most_type_name(&ty).unwrap();
        assert_eq!("f64", ty.into_token_stream().to_string());

        let ty = syn::parse_str::<TypePath>("String").unwrap();
        let ty = inner_most_type_name(&ty).unwrap();
        assert_eq!("String", ty.into_token_stream().to_string());

        let ty = syn::parse_str::<TypePath>("bool").unwrap();
        let ty = inner_most_type_name(&ty).unwrap();
        assert_eq!("bool", ty.into_token_stream().to_string());

        let ty = syn::parse_str::<TypePath>("Vec<i32>").unwrap();
        let ty = inner_most_type_name(&ty).unwrap();
        assert_eq!("i32", ty.into_token_stream().to_string());

        let ty = syn::parse_str::<TypePath>("Option<Vec<Option<i32>>>").unwrap();
        let ty = inner_most_type_name(&ty).unwrap();
        assert_eq!("i32", ty.into_token_stream().to_string());

        let ty = syn::parse_str::<TypePath>("Option<Vec<Option<i32>>>").unwrap();
        let ty = inner_most_type_name(&ty).unwrap();
        assert_eq!("i32", ty.into_token_stream().to_string());

        let ty = syn::parse_str::<TypePath>("Vec<&i32>").unwrap();
        let ty = inner_most_type_name(&ty).unwrap();
        assert_eq!("i32", ty.into_token_stream().to_string());
    }
}
