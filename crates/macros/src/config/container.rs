use crate::common::Container;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

impl Container<'_> {
    pub fn generate_default_values(&self) -> TokenStream {
        match self {
            Self::NamedStruct {
                fields: settings, ..
            } => {
                let mut setting_names = vec![];
                let mut default_values = vec![];

                for setting in settings {
                    setting_names.push(setting.name);
                    default_values.push(setting.generate_default_value());
                }

                quote! {
                    Ok(Some(Self {
                        #(#setting_names: #default_values),*
                    }))
                }
            }
            Self::UnnamedStruct {
                fields: settings, ..
            } => {
                let mut default_values = vec![];

                for setting in settings {
                    default_values.push(setting.generate_default_value());
                }

                quote! {
                    Ok(Some(Self(
                        #(#default_values),*
                    )))
                }
            }
            Self::Enum { variants } => {
                let default_variant = variants.iter().find(|v| v.is_default());

                if let Some(variant) = default_variant {
                    let default_value = variant.generate_default_value();

                    quote! {
                        Ok(Some(Self::#default_value))
                    }
                } else {
                    quote! {
                        Ok(None)
                    }
                }
            }
        }
    }

    pub fn generate_env_values(&self) -> TokenStream {
        match self {
            Self::NamedStruct {
                fields: settings, ..
            }
            | Self::UnnamedStruct {
                fields: settings, ..
            } => {
                let env_stmts = settings
                    .iter()
                    .filter_map(|s| s.generate_env_statement())
                    .collect::<Vec<_>>();

                if env_stmts.is_empty() {
                    quote! {
                        Ok(None)
                    }
                } else {
                    quote! {
                        let mut tracker = std::collections::HashSet::new();
                        let mut partial = Self::default();
                        #(#env_stmts)*
                        Ok(if tracker.is_empty() {
                            None
                        } else {
                            Some(partial)
                        })
                    }
                }
            }
            Self::Enum { .. } => {
                quote! {
                    Ok(None)
                }
            }
        }
    }

    pub fn generate_extends_from(&self) -> TokenStream {
        #[cfg(feature = "extends")]
        match self {
            Self::NamedStruct {
                fields: settings, ..
            } => {
                use quote::ToTokens;

                // Validate only 1 setting is using it
                let mut names = vec![];

                for setting in settings {
                    if setting.is_extendable() {
                        names.push(setting.get_name_raw().to_string());
                    }
                }

                if names.len() > 1 {
                    panic!(
                        "Only 1 setting may use `extend`, found: {}",
                        names.join(", ")
                    );
                }

                // Loop again and generate the necessary code
                for setting in settings {
                    if !setting.is_extendable() {
                        continue;
                    }

                    if let Some(inner_type) = setting.value_type.get_inner_type() {
                        let name = setting.get_name_raw();
                        let value = format!("{}", inner_type.to_token_stream());

                        // Janky but works!
                        match value.as_str() {
                            "String" => {
                                return quote! {
                                    self.#name
                                        .as_ref()
                                        .map(|inner| schematic::ExtendsFrom::String(inner.to_owned()))
                                };
                            }
                            "Vec<String>" | "Vec < String >" => {
                                return quote! {
                                    self.#name
                                        .as_ref()
                                        .map(|inner| schematic::ExtendsFrom::List(inner.to_owned()))
                                };
                            }
                            "ExtendsFrom"
                            | "schematic::ExtendsFrom"
                            | "schematic :: ExtendsFrom" => {
                                return quote! {
                                    self.#name.clone()
                                };
                            }
                            inner => {
                                let inner = inner.to_string();

                                panic!(
                                    "Only `String`, `Vec<String>`, or `ExtendsFrom` are supported when using `extend` for {name}. Received `{inner}`."
                                );
                            }
                        };
                    }
                }

                quote! { None }
            }
            Self::UnnamedStruct { .. } | Self::Enum { .. } => {
                quote! { None }
            }
        }

        #[cfg(not(feature = "extends"))]
        quote! { None }
    }

    pub fn generate_finalize(&self) -> TokenStream {
        match self {
            Self::NamedStruct {
                fields: settings, ..
            }
            | Self::UnnamedStruct {
                fields: settings, ..
            } => {
                let finalize_stmts = settings
                    .iter()
                    .map(|s| s.generate_finalize_statement())
                    .collect::<Vec<_>>();

                let env_statement = if cfg!(feature = "env") {
                    quote! {
                        if let Some(data) = Self::env_values()? {
                            partial.merge(context, data)?;
                        }
                    }
                } else {
                    quote! {}
                };

                quote! {
                    let mut partial = Self::default();

                    if let Some(data) = Self::default_values(context)? {
                        partial.merge(context, data)?;
                    }

                    partial.merge(context, self)?;

                    #env_statement

                    #(#finalize_stmts)*

                    Ok(partial)
                }
            }
            Self::Enum { variants } => {
                if self.has_nested() {
                    let finalize_stmts = variants
                        .iter()
                        .flat_map(|s| s.generate_finalize_statement())
                        .collect::<Vec<_>>();

                    quote! {
                        Ok(match self {
                            #(#finalize_stmts)*
                            _ => self
                        })
                    }
                } else {
                    quote! {
                        Ok(self)
                    }
                }
            }
        }
    }

    pub fn generate_merge(&self) -> TokenStream {
        match self {
            Self::NamedStruct {
                fields: settings, ..
            }
            | Self::UnnamedStruct {
                fields: settings, ..
            } => {
                let merge_stmts = settings
                    .iter()
                    .map(|s| s.generate_merge_statement())
                    .collect::<Vec<_>>();

                quote! {
                    #(#merge_stmts)*
                    Ok(())
                }
            }
            Self::Enum { variants } => {
                let merge_stmts = variants
                    .iter()
                    .filter_map(|s| s.generate_merge_statement())
                    .collect::<Vec<_>>();

                if merge_stmts.is_empty() {
                    quote! {
                        *self = next;
                        Ok(())
                    }
                } else {
                    quote! {
                        match self {
                            #(#merge_stmts)*
                            _ => {
                                *self = next;
                            }
                        };
                        Ok(())
                    }
                }
            }
        }
    }

    pub fn generate_validate(&self) -> TokenStream {
        match self {
            Self::NamedStruct {
                fields: settings, ..
            }
            | Self::UnnamedStruct {
                fields: settings, ..
            } => {
                let validate_stmts = settings
                    .iter()
                    .map(|s| s.generate_validate_statement())
                    .collect::<Vec<_>>();

                quote! {
                    #(#validate_stmts)*
                }
            }
            Self::Enum { variants } => {
                let validate_stmts = variants
                    .iter()
                    .filter_map(|s| s.generate_validate_statement())
                    .collect::<Vec<_>>();

                if validate_stmts.is_empty() {
                    quote! {}
                } else {
                    quote! {
                        match self {
                            #(#validate_stmts)*
                            _ => {}
                        };
                    }
                }
            }
        }
    }

    pub fn generate_from_partial(&self, partial_name: &Ident) -> TokenStream {
        match self {
            Self::NamedStruct {
                fields: settings, ..
            } => {
                let mut setting_names = vec![];
                let mut from_partial_values = vec![];

                for setting in settings {
                    setting_names.push(setting.name);
                    from_partial_values.push(setting.generate_from_partial_value());
                }

                quote! {
                    Self {
                        #(#setting_names: #from_partial_values),*
                    }
                }
            }
            Self::UnnamedStruct {
                fields: settings, ..
            } => {
                let mut from_partial_values = vec![];

                for setting in settings {
                    from_partial_values.push(setting.generate_from_partial_value());
                }

                quote! {
                    Self(
                        #(#from_partial_values),*
                    )
                }
            }
            Self::Enum { variants } => {
                let from_partial_values = variants
                    .iter()
                    .map(|s| s.generate_from_partial_value(partial_name))
                    .collect::<Vec<_>>();

                quote! {
                    match partial {
                        #(#from_partial_values)*
                    }
                }
            }
        }
    }

    pub fn generate_partial(
        &self,
        partial_name: &Ident,
        partial_attrs: &[TokenStream],
        is_untagged: bool,
    ) -> TokenStream {
        match self {
            Self::NamedStruct {
                fields: settings, ..
            } => {
                quote! {
                    #[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
                    #(#partial_attrs)*
                    pub struct #partial_name {
                        #(#settings)*
                    }
                }
            }
            Self::UnnamedStruct {
                fields: settings, ..
            } => {
                quote! {
                    #[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
                    #(#partial_attrs)*
                    pub struct #partial_name(
                        #(#settings)*
                    );
                }
            }
            Self::Enum { variants } => {
                let default_variant = variants
                    .iter()
                    .find(|v| v.is_default())
                    .or_else(|| variants.first());

                let default_impl = if let Some(default) = default_variant {
                    let value = default.generate_default_value();

                    quote! { Self::#value }
                } else {
                    quote! { panic!("No variant has been marked as default!"); }
                };

                if is_untagged {
                    // For untagged enums, generate custom Deserialize that collects all errors
                    let deserialize_impl =
                        self.generate_untagged_deserialize(partial_name, variants);

                    quote! {
                        #[derive(Clone, Debug, PartialEq, serde::Serialize)]
                        #(#partial_attrs)*
                        pub enum #partial_name {
                            #(#variants)*
                        }

                        impl Default for #partial_name {
                            fn default() -> Self {
                                #default_impl
                            }
                        }

                        #deserialize_impl
                    }
                } else {
                    quote! {
                        #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
                        #(#partial_attrs)*
                        pub enum #partial_name {
                            #(#variants)*
                        }

                        impl Default for #partial_name {
                            fn default() -> Self {
                                #default_impl
                            }
                        }
                    }
                }
            }
        }
    }

    fn generate_untagged_deserialize(
        &self,
        partial_name: &Ident,
        variants: &[crate::common::Variant<'_>],
    ) -> TokenStream {
        use syn::Fields;

        // Generate a deserialize attempt for each variant
        let mut variant_attempts = vec![];

        for variant in variants {
            let name = &variant.name;
            let variant_name_str = variant.get_name(Some(&variant.casing_format));

            match &variant.value.fields {
                Fields::Named(fields) => {
                    // Struct-like variant with named fields: Variant { a: Type, b: Type }
                    // We need to generate a helper struct to deserialize and then convert to the variant
                    let field_names: Vec<_> = fields
                        .named
                        .iter()
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect();
                    let field_types: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();

                    let inner_types: Vec<_> = field_types
                        .iter()
                        .map(|ty| {
                            if variant.is_nested() {
                                quote! { <#ty as schematic::Config>::Partial }
                            } else {
                                quote! { #ty }
                            }
                        })
                        .collect();

                    // Create a temporary struct type name for deserialization
                    let helper_struct_name =
                        quote::format_ident!("__{}{}Helper", partial_name, name);

                    variant_attempts.push(quote! {
                        {
                            // Define a helper struct that matches the variant's fields
                            #[derive(serde::Deserialize)]
                            struct #helper_struct_name {
                                #(#field_names: #inner_types),*
                            }

                            let deserializer = serde_content::Deserializer::new(content.clone())
                                .coerce_numbers()
                                .human_readable();
                            match <#helper_struct_name as serde::Deserialize>::deserialize(deserializer) {
                                Ok(value) => return Ok(#partial_name::#name {
                                    #(#field_names: value.#field_names),*
                                }),
                                Err(e) => errors.push((#variant_name_str, e.to_string())),
                            }
                        }
                    });
                }
                Fields::Unnamed(fields) => {
                    let field_types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();

                    if field_types.len() == 1 {
                        let ty = &field_types[0];
                        // Check if this is a nested variant
                        let inner_ty = if variant.is_nested() {
                            quote! { <#ty as schematic::Config>::Partial }
                        } else {
                            quote! { #ty }
                        };

                        variant_attempts.push(quote! {
                            {
                                let deserializer = serde_content::Deserializer::new(content.clone())
                                    .coerce_numbers()
                                    .human_readable();
                                match <#inner_ty as serde::Deserialize>::deserialize(deserializer) {
                                    Ok(value) => return Ok(#partial_name::#name(value)),
                                    Err(e) => errors.push((#variant_name_str, e.to_string())),
                                }
                            }
                        });
                    } else {
                        // Tuple variant with multiple fields
                        let inner_types: Vec<_> = field_types
                            .iter()
                            .map(|ty| {
                                if variant.is_nested() {
                                    quote! { <#ty as schematic::Config>::Partial }
                                } else {
                                    quote! { #ty }
                                }
                            })
                            .collect();

                        // Generate field accessors: value.0, value.1, value.2, etc.
                        let field_accessors: Vec<_> = (0..field_types.len())
                            .map(|i| {
                                let idx = syn::Index::from(i);
                                quote! { value.#idx }
                            })
                            .collect();

                        variant_attempts.push(quote! {
                            {
                                let deserializer = serde_content::Deserializer::new(content.clone())
                                    .coerce_numbers()
                                    .human_readable();
                                match <(#(#inner_types),*) as serde::Deserialize>::deserialize(deserializer) {
                                    Ok(value) => return Ok(#partial_name::#name(#(#field_accessors),*)),
                                    Err(e) => errors.push((#variant_name_str, e.to_string())),
                                }
                            }
                        });
                    }
                }
                Fields::Unit => {
                    // Unit variants in untagged enums are serialized as null
                    variant_attempts.push(quote! {
                        {
                            let deserializer = serde_content::Deserializer::new(content.clone())
                                .coerce_numbers()
                                .human_readable();
                            match <() as serde::Deserialize>::deserialize(deserializer) {
                                Ok(_) => return Ok(#partial_name::#name),
                                Err(e) => errors.push((#variant_name_str, e.to_string())),
                            }
                        }
                    });
                }
            }
        }

        quote! {
            impl<'de> serde::Deserialize<'de> for #partial_name {
                fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    use serde::de::Error as _;

                    // Buffer the content so we can try deserializing it multiple ways
                    let content = deserializer.deserialize_any(serde_content::ValueVisitor)?;

                    let mut errors: Vec<(&str, String)> = Vec::new();

                    #(#variant_attempts)*

                    // All variants failed, build the combined error message
                    let mut error_msg = format!("failed to parse as any variant of {}:", stringify!(#partial_name));
                    for (variant_name, error) in &errors {
                        error_msg.push_str(&format!("\n- {}: {}", variant_name, error));
                    }

                    Err(D::Error::custom(error_msg))
                }
            }
        }
    }

    #[cfg(feature = "schema")]
    pub fn generate_partial_schema(
        &self,
        config_name: &Ident,
        _partial_name: &Ident,
    ) -> TokenStream {
        quote! {
            let mut schema = #config_name::build_schema(schema);
            schematic::internal::partialize_schema(&mut schema, true);
        }
    }
}
