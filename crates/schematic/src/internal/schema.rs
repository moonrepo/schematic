use schematic_types::{Schema, SchemaBuilder, Schematic};

/// Build the schema of a config's partial, from the schema of the config.
pub fn build_partial_schema<T: Schematic>(schema: SchemaBuilder) -> Schema {
    let mut schema = T::build_schema(schema);
    partialize_schema(&mut schema, true);
    schema
}

/// Turn the schema of a config into the schema of its partial: the settings
/// of each struct marked `partial` become optional and nullable, and the
/// struct is renamed to its partial. `force_partial` treats `schema` itself
/// as marked.
pub fn partialize_schema(schema: &mut Schema, force_partial: bool) {
    use schematic_types::*;

    let mut update_name = |update: bool| {
        if update {
            if let Some(name) = &schema.name {
                if !name.starts_with("Partial") {
                    schema.name = Some(format!("Partial{name}"));
                }
            }
        }
    };

    match &mut schema.ty {
        SchemaType::Array(inner) => {
            partialize_schema(&mut inner.items_type, false);
        }
        SchemaType::Object(inner) => {
            partialize_schema(&mut inner.key_type, false);
            partialize_schema(&mut inner.value_type, false);
        }
        SchemaType::Struct(inner) => {
            if inner.partial || force_partial {
                update_name(true);

                for field in inner.fields.values_mut() {
                    // Once nullable, the default of a named type within is no
                    // longer reported, as it belongs to the type, so the
                    // default it had as a setting is kept on the field
                    if field.default.is_none() && !field.schema.is_nullable() {
                        field.default = field.schema.get_default().cloned();
                    }

                    field.optional = true;
                    field.nullable = true;
                    field.schema.nullify();

                    partialize_schema(&mut field.schema, true);
                }
            } else {
                for field in inner.fields.values_mut() {
                    partialize_schema(&mut field.schema, false);
                }
            }
        }
        // The tag is always written, so only the value can be a partial
        SchemaType::Tagged(inner) => {
            if let Some(value) = &mut inner.value {
                partialize_schema(value, force_partial);
            }
        }
        SchemaType::Tuple(inner) => {
            for item in inner.items_types.iter_mut() {
                partialize_schema(item, false);
            }
        }
        SchemaType::Union(inner) => {
            update_name(inner.partial || force_partial);

            for variant in inner.variants_types.iter_mut() {
                partialize_schema(variant, false);
            }
        }
        // A cycle within a nested config resolves to a reference, which must
        // follow the type it points at, otherwise it names a type that was
        // never rendered.
        SchemaType::Reference { name, partial }
            if (*partial || force_partial) && !name.starts_with("Partial") =>
        {
            *name = format!("Partial{name}");
        }
        _ => {}
    };
}
