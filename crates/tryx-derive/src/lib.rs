extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Error, Fields, GenericArgument, GenericParam, Ident,
    PathArguments, Type, parse_macro_input, parse_quote,
};

#[proc_macro_derive(Outcome, attributes(outcome))]
pub fn derive_outcome(input: TokenStream) -> TokenStream {
    expand(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let enum_ident = input.ident;
    let Data::Enum(data) = input.data else {
        return Err(Error::new_spanned(
            enum_ident,
            "Outcome can only be derived for enums",
        ));
    };

    let mut success = None;
    let mut short = None;

    for variant in data.variants {
        if has_flag(&variant.attrs, "success")? {
            if success.is_some() {
                return Err(Error::new_spanned(
                    variant.ident,
                    "multiple success variants",
                ));
            }
            success = Some((variant.ident, single_field_type(&variant.fields)?));
        } else if has_flag(&variant.attrs, "short_circuit")? {
            if short.is_some() {
                return Err(Error::new_spanned(
                    variant.ident,
                    "multiple short_circuit variants",
                ));
            }
            short = Some((variant.ident, single_field_type(&variant.fields)?));
        }
    }

    let (success_ident, output_ty) =
        success.ok_or_else(|| Error::new_spanned(&enum_ident, "missing #[outcome(success)]"))?;
    let (short_ident, residual_ty) = short
        .ok_or_else(|| Error::new_spanned(&enum_ident, "missing #[outcome(short_circuit)]"))?;
    let result_interop = result_interop(&input.attrs)?;
    let residual_ident = format_ident!("{enum_ident}Residual");
    let mut result_generics = input.generics.clone();
    result_generics
        .params
        .push(GenericParam::Type(parse_quote!(E)));
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let (result_impl_generics, _, _) = result_generics.split_for_impl();
    let residual_generic = residual_type_generic(&residual_ty);

    let result_impl = result_interop.map(|_| {
        quote! {
            impl #result_impl_generics ::core::ops::FromResidual<#residual_ident<#residual_ty>>
                for ::core::result::Result<#output_ty, E>
            where
                E: ::core::convert::From<#residual_ty>,
            {
                fn from_residual(residual: #residual_ident<#residual_ty>) -> Self {
                    ::core::result::Result::Err(residual.0.into())
                }
            }
        }
    });

    Ok(quote! {
        pub struct #residual_ident<#residual_generic>(pub #residual_generic);

        impl #impl_generics ::core::ops::Try for #enum_ident #ty_generics #where_clause {
            type Output = #output_ty;
            type Residual = #residual_ident<#residual_ty>;

            fn from_output(output: Self::Output) -> Self {
                Self::#success_ident(output)
            }

            fn branch(self) -> ::core::ops::ControlFlow<Self::Residual, Self::Output> {
                match self {
                    Self::#success_ident(value) => ::core::ops::ControlFlow::Continue(value),
                    Self::#short_ident(value) => ::core::ops::ControlFlow::Break(#residual_ident(value)),
                }
            }
        }

        impl #impl_generics ::core::ops::FromResidual<#residual_ident<#residual_ty>>
            for #enum_ident #ty_generics
            #where_clause
        {
            fn from_residual(residual: #residual_ident<#residual_ty>) -> Self {
                Self::#short_ident(residual.0)
            }
        }

        #result_impl
    })
}

fn has_flag(attrs: &[Attribute], name: &str) -> syn::Result<bool> {
    let mut found = false;
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("outcome")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(name) {
                found = true;
            }
            Ok(())
        })?;
    }
    Ok(found)
}

fn result_interop(attrs: &[Attribute]) -> syn::Result<Option<Type>> {
    let mut out = None;
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("outcome")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("result_interop") {
                let value = meta.value()?;
                out = Some(value.parse()?);
            }
            Ok(())
        })?;
    }
    Ok(out)
}

fn single_field_type(fields: &Fields) -> syn::Result<Type> {
    match fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => fields
            .unnamed
            .first()
            .map(|field| field.ty.clone())
            .ok_or_else(|| {
                Error::new_spanned(fields, "variant must have exactly one unnamed field")
            }),
        _ => Err(Error::new_spanned(
            fields,
            "variant must have exactly one unnamed field",
        )),
    }
}

fn residual_type_generic(ty: &Type) -> Ident {
    if let Type::Path(path) = ty
        && let Some(segment) = path.path.segments.last()
        && let PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(GenericArgument::Type(Type::Path(inner))) = args.args.first()
        && let Some(inner_segment) = inner.path.segments.last()
    {
        return inner_segment.ident.clone();
    }
    format_ident!("R")
}
