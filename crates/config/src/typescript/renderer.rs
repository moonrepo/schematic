use super::converter::Type;
use std::fmt::Write;

#[derive(Default)]
pub enum ObjectType {
    #[default]
    Interface,
    Type,
}

#[derive(Default)]
pub struct RenderOptions {
    pub exclude_partial: bool,
    pub object_type: ObjectType,
}

pub enum Output {
    Field {
        name: String,
        type_of: Type,
        optional: bool,
    },

    Struct {
        name: String,
        fields: Vec<Output>,
    },
}

pub fn render_output(
    output: &Output,
    partial: bool,
    options: &RenderOptions,
) -> Result<String, std::fmt::Error> {
    let mut buffer = String::new();

    match output {
        Output::Field {
            name,
            type_of,
            optional,
        } => {
            if partial {
                write!(buffer, "{}?", name)?;
            } else {
                write!(buffer, "{}", name)?;
            }

            write!(buffer, ": {}", type_of)?;

            if partial || *optional {
                write!(buffer, " | null")?;
            }

            write!(buffer, ";")?;
        }
        Output::Struct { name, fields } => {
            let name = if partial {
                format!("Partial{}", name)
            } else {
                name.to_owned()
            };

            if matches!(options.object_type, ObjectType::Interface) {
                write!(buffer, "export interface {} {{\n", name)?;
            } else {
                write!(buffer, "export type {} = {{\n", name)?;
            }

            for field in fields {
                write!(buffer, "\t{}\n", render_output(field, partial, options)?)?;
            }

            write!(buffer, "}}")?;
        }
    };

    Ok(buffer)
}
