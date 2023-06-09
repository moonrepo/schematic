use super::converter::Type;
use std::fmt::Write;

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

pub fn render_output(output: &Output, partial: bool) -> Result<String, std::fmt::Error> {
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

            if *optional {
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

            write!(buffer, "export interface {} {{\n", name)?;

            for field in fields {
                write!(buffer, "\t{}\n", render_output(field, partial)?)?;
            }

            write!(buffer, "}}")?;
        }
    };

    Ok(buffer)
}
