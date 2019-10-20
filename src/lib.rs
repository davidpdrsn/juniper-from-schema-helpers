extern crate proc_macro;
extern crate proc_macro2;

use syn::{spanned::Spanned, Ident, TypePath};

macro_rules! or_return_compile_error {
    ($e:expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => return e.to_compile_error().into(),
        }
    };
}

mod field;
mod loaded_association;

#[proc_macro]
pub fn field(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    field::expand(input)
}

#[proc_macro]
pub fn loaded_association(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    loaded_association::expand(input)
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
