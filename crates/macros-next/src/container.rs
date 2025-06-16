use crate::args::{SerdeContainerArgs, SerdeRenameArg};
use crate::field::Field;
use crate::variant::Variant;
use darling::FromDeriveInput;
use std::rc::Rc;
use syn::{Attribute, Data, DeriveInput, ExprPath, Fields, Ident, Visibility};

// #[config()], #[schematic()]
#[derive(Default, FromDeriveInput)]
#[darling(
    default,
    attributes(config, schematic),
    supports(struct_named, enum_any)
)]
pub struct ContainerArgs {
    // config
    pub allow_unknown_fields: bool,
    pub context: Option<ExprPath>,
    // pub partial: PartialAttr, // TODO
    #[cfg(feature = "env")]
    pub env_prefix: Option<String>,

    // serde
    pub rename: Option<SerdeRenameArg>,
    pub rename_all: Option<SerdeRenameArg>,
    pub rename_all_fields: Option<SerdeRenameArg>,
    // pub serde: SerdeMeta, // TODO
}

pub struct Container {
    pub args: Rc<ContainerArgs>,
    pub inner: ContainerInner,
    pub serde_args: Rc<SerdeContainerArgs>,

    // inherited
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub vis: Visibility,
}

impl Container {
    pub fn from(input: DeriveInput) -> Self {
        let args = Rc::new(ContainerArgs::from_derive_input(&input).unwrap_or_default());
        let serde_args = Rc::new(SerdeContainerArgs::from_derive_input(&input).unwrap_or_default());

        let inner = match input.data {
            Data::Struct(data) => match data.fields {
                Fields::Named(fields) => ContainerInner::NamedStruct {
                    fields: fields
                        .named
                        .into_iter()
                        .map(|data| Field::new(data, args.clone(), serde_args.clone()))
                        .collect(),
                },
                Fields::Unnamed(fields) => ContainerInner::UnnamedStruct {
                    fields: fields
                        .unnamed
                        .into_iter()
                        .enumerate()
                        .map(|(index, data)| {
                            let mut field = Field::new(data, args.clone(), serde_args.clone());
                            field.index = index;
                            field
                        })
                        .collect(),
                },
                Fields::Unit => {
                    panic!("Unit structs are not supported.");
                }
            },
            Data::Enum(data) => {
                let all_unit = data
                    .variants
                    .iter()
                    .all(|variant| matches!(variant.fields, Fields::Unit));
                let variants = data
                    .variants
                    .into_iter()
                    .map(|data| Variant::new(data, args.clone(), serde_args.clone()))
                    .collect::<Vec<_>>();

                if all_unit {
                    ContainerInner::UnitEnum { variants }
                } else {
                    ContainerInner::Enum { variants }
                }
            }
            Data::Union(_) => {
                panic!("Unions are not supported.");
            }
        };

        Self {
            args,
            attrs: input.attrs,
            ident: input.ident,
            inner,
            serde_args,
            vis: input.vis,
        }
    }
}

pub enum ContainerInner {
    NamedStruct { fields: Vec<Field> },
    UnnamedStruct { fields: Vec<Field> },
    Enum { variants: Vec<Variant> },
    UnitEnum { variants: Vec<Variant> },
}
