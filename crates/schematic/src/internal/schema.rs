use schematic_types::{Schema, SchemaBuilder, SchemaField, SchemaType, Schematic, StructType};

/// Build the schema of a config's partial, from the schema of the config.
pub fn build_partial_schema<T: Schematic>(schema: SchemaBuilder) -> Schema {
    let mut schema = T::build_schema(schema);
    partialize_schema(&mut schema, true);
    schema
}

/// The schema of a unit variant with a tag, `{ "tag": "name" }`.
pub fn tagged_unit(tag: &str, field: SchemaField) -> Schema {
    Schema::structure(StructType::new([(tag.to_owned(), field)]))
}

/// The schema of an internally tagged variant, `{ "tag": "name", ...value }`.
///
/// A named struct is shared by every type that holds it, so the tag can't be
/// added to it without claiming the tag for all of them. The variant is a
/// struct of the tag and the value flattened instead, which renderers
/// reference by name, like `{ type: 'git' } & GitSource`. A struct of its
/// own, such as a named variant's fields, holds the tag itself.
pub fn internally_tagged(mut schema: Schema, tag: &str, field: SchemaField) -> Schema {
    let SchemaType::Struct(inner) = &schema.ty else {
        return schema;
    };

    let Some(name) = schema.name.clone() else {
        schema.add_field(tag, field);

        return schema;
    };

    // Serde reads the keys besides the tag as the value, so the variant
    // accepts unknown keys when the value does
    let allow_unknown_fields = inner.allow_unknown_fields;

    // A flattened value has no key of its own, so it's keyed by its type,
    // which the tag can't share
    let mut key = name;

    while key == tag {
        key.insert(0, '_');
    }

    let mut value = SchemaField::new(schema);
    value.flatten = true;

    let mut structure = StructType::new([(tag.to_owned(), field), (key, value)]);
    structure.allow_unknown_fields = allow_unknown_fields;

    Schema::structure(structure)
}

/// The schema of an adjacently tagged variant,
/// `{ "tag": "name", "content": value }`. The value keeps its description, as
/// a named type's schema is shared by every field that holds it.
pub fn adjacently_tagged(tag: &str, field: SchemaField, content: &str, value: Schema) -> Schema {
    Schema::structure(StructType::new([
        (tag.to_owned(), field),
        (content.to_owned(), SchemaField::new(value)),
    ]))
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
                    // A literal is the tag an internally tagged variant adds to
                    // the struct it holds, which serde requires of every layer
                    if matches!(field.schema.ty, SchemaType::Literal(_)) {
                        continue;
                    }

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
