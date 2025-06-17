use crate::field::FieldNestedArg;
use crate::utils::to_type_string;
use quote::ToTokens;
use syn::{GenericArgument, Ident, PathArguments, PathSegment, Type};

#[derive(Debug, PartialEq)]
pub enum WrapperType {
    Arc,
    Box,
    Option,
    Rc,
}

#[derive(Debug)]
pub struct FieldValue {
    pub nested: bool,
    pub nested_ident: Option<FieldNestedIdent>,
    pub ty: Type,
    pub ty_string: String,
    pub wrappers: Vec<WrapperType>,
}

impl FieldValue {
    pub fn new(ty: Type, nested_arg: Option<&FieldNestedArg>) -> Self {
        let mut nested = false;
        let mut nested_ident = None;
        let mut wrappers = vec![];
        let ty_string = to_type_string(ty.to_token_stream());

        // Determine nested state
        if let Some(nested_arg) = nested_arg {
            match nested_arg {
                FieldNestedArg::Detect(state) => {
                    nested = *state;
                }
                FieldNestedArg::Ident(ident) => {
                    nested = true;
                    nested_ident = Some(FieldNestedIdent::Unknown(ident.to_owned()));

                    if !ty_string.contains(&ident.to_string()) {
                        panic!(
                            "Nested configuration identifier `{ident}` does not exist within `{ty_string}`."
                        )
                    }
                }
            };
        }

        // Extract type information
        if let Some(custom_ident) =
            extract_type_information(&ty, &mut wrappers, None, nested && nested_ident.is_none())
        {
            nested_ident = Some(custom_ident);
        }

        if nested_ident.is_none() && nested {
            panic!(
                "Unable to extract the nested configuration identifier from `{ty_string}`. Try explicitly passing the identifier with `nested = ConfigName`."
            )
        }

        let value = Self {
            nested,
            nested_ident,
            wrappers,
            ty_string,
            ty,
        };

        // dbg!(&value);

        value
    }

    pub fn is_outer_option_wrapped(&self) -> bool {
        self.wrappers
            .first()
            .is_some_and(|wrapper| *wrapper == WrapperType::Option)
    }
}

fn extract_type_information(
    ty: &Type,
    wrappers: &mut Vec<WrapperType>,
    parent_segment: Option<&PathSegment>,
    nested_ident: bool,
) -> Option<FieldNestedIdent> {
    // We don't need to traverse other types, just paths
    let Type::Path(ty_path) = ty else {
        return None;
    };

    // Extract the last segment of the path, for example `Option`,
    // instead of the full path `std::option::Option`
    let last_segment = ty_path.path.segments.last().unwrap();

    if last_segment.ident == "Option" {
        wrappers.push(WrapperType::Option);
    } else if last_segment.ident == "Arc" {
        wrappers.push(WrapperType::Arc);
    } else if last_segment.ident == "Box" {
        wrappers.push(WrapperType::Box);
    } else if last_segment.ident == "Rc" {
        wrappers.push(WrapperType::Rc);
    }

    match &last_segment.arguments {
        // We've reached the final segment
        PathArguments::None => {
            if nested_ident {
                return extract_nested_ident(&last_segment, parent_segment);
            }
        }

        // Attempt to drill deeper down
        PathArguments::AngleBracketed(args) => {
            if let Some(GenericArgument::Type(inner_ty)) = args.args.last() {
                return extract_type_information(
                    inner_ty,
                    wrappers,
                    Some(last_segment),
                    nested_ident,
                );
            }
        }

        // What to do here, anything?
        PathArguments::Parenthesized(_) => {}
    };

    None
}

fn extract_nested_ident(
    segment: &PathSegment,
    parent_segment: Option<&PathSegment>,
) -> Option<FieldNestedIdent> {
    let ident = segment.ident.to_owned();

    if let Some(parent) = parent_segment {
        let parent_id = parent.ident.to_string();

        if parent_id.ends_with("Vec") {
            return Some(FieldNestedIdent::Vec(ident));
        } else if parent_id.ends_with("Set") {
            return Some(FieldNestedIdent::Set(ident));
        } else if parent_id.ends_with("Map") {
            return Some(FieldNestedIdent::Map(ident));
        } else {
            return None;
        }
    }

    Some(FieldNestedIdent::Unknown(ident))
}

#[derive(Debug, PartialEq)]
pub enum FieldNestedIdent {
    Unknown(Ident),
    Map(Ident),
    Set(Ident),
    Vec(Ident),
}

impl FieldNestedIdent {
    pub fn get_ident(&self) -> &Ident {
        match self {
            Self::Unknown(ident) => ident,
            Self::Map(ident) => ident,
            Self::Set(ident) => ident,
            Self::Vec(ident) => ident,
        }
    }
}
