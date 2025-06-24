use crate::args::{SerdeContainerArgs, SerdeFieldArgs, SerdeRenameArg};
use crate::container::ContainerArgs;
use crate::field_value::FieldValue;
use crate::utils::{preserve_str_literal, to_type_string};
use darling::{FromAttributes, FromMeta};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::ops::Deref;
use std::rc::Rc;
use syn::{
    Attribute, Expr, ExprPath, Field as NativeField, FieldMutability, Ident, Visibility, parse_str,
};

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

// #[setting(validate)]
#[derive(Debug)]
pub struct FieldValidateArg(Expr);

impl FromMeta for FieldValidateArg {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Call(_) | Expr::Path(_) => Ok(Self(expr.to_owned())),
            _ => Err(darling::Error::unexpected_expr_type(expr)),
        }
        .map_err(|e| e.with_span(expr))
    }
}

impl Deref for FieldValidateArg {
    type Target = Expr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// #[schema()], #[setting()]
#[derive(Debug, FromAttributes, Default)]
#[darling(default, attributes(schema, setting))]
pub struct FieldArgs {
    #[darling(with = preserve_str_literal, map = "Some")]
    pub default: Option<Expr>,
    #[cfg(feature = "env")]
    pub env: Option<String>,
    #[cfg(feature = "env")]
    pub env_prefix: Option<String>,
    pub exclude: bool,
    #[cfg(feature = "extends")]
    pub extend: bool,
    pub merge: Option<ExprPath>,
    pub nested: Option<FieldNestedArg>,
    #[cfg(feature = "env")]
    pub parse_env: Option<ExprPath>,
    pub required: bool,
    pub transform: Option<ExprPath>,
    #[cfg(feature = "validate")]
    pub validate: Option<FieldValidateArg>,

    // serde
    #[darling(multiple)]
    pub alias: Vec<String>,
    pub flatten: bool,
    pub rename: Option<SerdeRenameArg>,
    pub skip: bool,
    pub skip_deserializing: bool,
    pub skip_deserializing_if: Option<String>,
    pub skip_serializing: bool,
    pub skip_serializing_if: Option<String>,
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

        field.validate_args();
        field
    }

    fn validate_args(&self) {
        #[cfg(feature = "env")]
        {
            // env
            if self.args.env.as_ref().is_some_and(|key| key.is_empty()) {
                panic!("Attribute `env` cannot be empty.");
            }

            if self.args.env.is_some() && self.args.env_prefix.is_some() {
                panic!("Cannot use `env` and `env_prefix` together.");
            }

            // env_prefix
            if self
                .args
                .env_prefix
                .as_ref()
                .is_some_and(|key| key.is_empty())
            {
                panic!("Attribute `env_prefix` cannot be empty.");
            }

            if self.args.env_prefix.is_some() && self.args.nested.is_none() {
                panic!("Cannot use `env_prefix` without `nested`.");
            }
        }

        // nested
        if self.args.nested.is_some() {
            if self.args.default.is_some() {
                panic!("Cannot use `default` with `nested`.");
            }

            #[cfg(feature = "env")]
            if self.args.env.is_some() {
                panic!("Cannot use `env` with `nested`, use `env_prefix` instead?");
            }
        }

        #[cfg(feature = "env")]
        {
            // parse_env
            if self.args.parse_env.is_some() && self.args.env.is_none() {
                panic!("Cannot use `parse_env` without `env`.");
            }
        }
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
