use crate::args::{SerdeContainerArgs, SerdeFieldArgs};
use crate::container::ContainerArgs;
use crate::field_value::FieldValue;
use crate::utils::to_type_string;
use darling::{FromAttributes, FromMeta};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::rc::Rc;
use syn::{Attribute, Expr, Field as NativeField, FieldMutability, Ident, Visibility, parse_str};

// #[setting(nested)]
#[derive(Debug)]
pub enum FieldNestedArg {
    Detect(bool),
    Ident(Ident),
}

impl FromMeta for FieldNestedArg {
    // #[setting(nested)]
    fn from_word() -> darling::Result<Self> {
        Ok(Self::Detect(true))
    }

    // #[setting(nested = true)]
    fn from_bool(value: bool) -> darling::Result<Self> {
        Ok(Self::Detect(value))
    }

    // #[setting(nested = NestedConfig)]
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Lit(lit) => Self::from_value(&lit.lit),
            Expr::Path(path) => {
                if path.path.segments.len() > 1 {
                    Err(darling::Error::custom(format!(
                        "Too many segments for `{}`, only a single identifier is allowed.",
                        to_type_string(path.to_token_stream())
                    )))
                } else {
                    Ok(Self::Ident(
                        path.path.segments.last().unwrap().ident.to_owned(),
                    ))
                }
            }
            _ => Err(darling::Error::unexpected_expr_type(expr)),
        }
        .map_err(|e| e.with_span(expr))
    }
}

// #[schema()], #[setting()]
#[derive(Debug, FromAttributes, Default)]
#[darling(default, attributes(schema, setting))]
pub struct FieldArgs {
    pub nested: Option<FieldNestedArg>,
}

#[derive(Debug)]
pub struct Field {
    pub value: FieldValue,

    // args
    pub args: FieldArgs,
    pub container_args: Rc<ContainerArgs>,
    pub serde_args: SerdeFieldArgs,
    pub serde_container_args: Rc<SerdeContainerArgs>,

    // inherited
    pub attrs: Vec<Attribute>,
    pub ident: Option<Ident>, // Named
    pub index: usize,         // Unnamed
    pub mutability: FieldMutability,
    pub vis: Visibility,
}

impl Field {
    pub fn new(
        field: NativeField,
        container_args: Rc<ContainerArgs>,
        serde_container_args: Rc<SerdeContainerArgs>,
    ) -> Self {
        let args = FieldArgs::from_attributes(&field.attrs).unwrap();
        let serde_args = SerdeFieldArgs::from_attributes(&field.attrs).unwrap();

        let field = Self {
            attrs: field.attrs,
            container_args,
            ident: field.ident,
            index: 0,
            mutability: field.mutability,
            serde_args,
            serde_container_args,
            vis: field.vis,
            value: FieldValue::new(field.ty, args.nested.as_ref()),
            args,
        };

        // dbg!(&field);

        field
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut value = self.value.ty_string.clone();

        if let Some(nested_ident) = &self.value.nested_ident {
            let ident = nested_ident.to_string();

            value = value.replace(&ident, &format!("<{ident} as schematic::Config>::Partial"));
        }

        if !self.value.is_outer_option_wrapped() {
            value = format!("Option<{value}>");
        }

        let key = self.ident.as_ref().unwrap();
        let value: TokenStream = parse_str(&value).unwrap();

        tokens.extend(quote! {
            pub #key: #value,
        });
    }
}
