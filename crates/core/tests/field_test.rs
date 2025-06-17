use schematic_core::container::Container;
use schematic_core::field::Field;
use schematic_core::field_value::WrapperType;
use syn::{Ident, parse_quote};

fn get_field<'a>(fields: &'a [&'a Field], key: &str) -> &'a Field {
    fields
        .iter()
        .find(|field| field.ident.as_ref().is_some_and(|ident| ident == key))
        .unwrap()
}

fn get_field_nested_ident(field: &Field) -> &Ident {
    field.value.nested_ident.as_ref().unwrap().get_ident()
}

mod field {
    use super::*;

    #[test]
    fn extracts_wrappers() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                a: bool,
                b: Option<bool>,
                c: Arc<bool>,
                d: Box<bool>,
                e: Rc<bool>,
                f: Option<Arc<bool>>,
                g: Option<Box<bool>>,
                h: Option<Rc<bool>>,
                i: Arc<Option<bool>>,
                j: Box<Option<bool>>,
                k: Rc<Option<bool>>,
                l: Option<Arc<Option<bool>>>,
            }
        });
        let fields = container.inner.get_fields();

        // a
        let field = get_field(&fields, "a");
        assert_eq!(field.value.ty_string, "bool");
        assert_eq!(field.value.wrappers, vec![]);

        // b
        let field = get_field(&fields, "b");
        assert_eq!(field.value.ty_string, "Option<bool>");
        assert_eq!(field.value.wrappers, vec![WrapperType::Option]);

        // c
        let field = get_field(&fields, "c");
        assert_eq!(field.value.ty_string, "Arc<bool>");
        assert_eq!(field.value.wrappers, vec![WrapperType::Arc]);

        // d
        let field = get_field(&fields, "d");
        assert_eq!(field.value.ty_string, "Box<bool>");
        assert_eq!(field.value.wrappers, vec![WrapperType::Box]);

        // e
        let field = get_field(&fields, "e");
        assert_eq!(field.value.ty_string, "Rc<bool>");
        assert_eq!(field.value.wrappers, vec![WrapperType::Rc]);

        // f
        let field = get_field(&fields, "f");
        assert_eq!(field.value.ty_string, "Option<Arc<bool>>");
        assert_eq!(
            field.value.wrappers,
            vec![WrapperType::Option, WrapperType::Arc]
        );

        // g
        let field = get_field(&fields, "g");
        assert_eq!(field.value.ty_string, "Option<Box<bool>>");
        assert_eq!(
            field.value.wrappers,
            vec![WrapperType::Option, WrapperType::Box]
        );

        // h
        let field = get_field(&fields, "h");
        assert_eq!(field.value.ty_string, "Option<Rc<bool>>");
        assert_eq!(
            field.value.wrappers,
            vec![WrapperType::Option, WrapperType::Rc]
        );

        // i
        let field = get_field(&fields, "i");
        assert_eq!(field.value.ty_string, "Arc<Option<bool>>");
        assert_eq!(
            field.value.wrappers,
            vec![WrapperType::Arc, WrapperType::Option]
        );

        // j
        let field = get_field(&fields, "j");
        assert_eq!(field.value.ty_string, "Box<Option<bool>>");
        assert_eq!(
            field.value.wrappers,
            vec![WrapperType::Box, WrapperType::Option]
        );

        // k
        let field = get_field(&fields, "k");
        assert_eq!(field.value.ty_string, "Rc<Option<bool>>");
        assert_eq!(
            field.value.wrappers,
            vec![WrapperType::Rc, WrapperType::Option]
        );

        // l
        let field = get_field(&fields, "l");
        assert_eq!(field.value.ty_string, "Option<Arc<Option<bool>>>");
        assert_eq!(
            field.value.wrappers,
            vec![WrapperType::Option, WrapperType::Arc, WrapperType::Option]
        );
    }

    #[test]
    fn extracts_vec_types() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                a: Vec<String>,
                b: Option<Vec<String>>,
                c: SmallVec<String>,
                d: Vec<Option<String>>,
                e: Vec<Vec<String>>,
            }
        });
        let fields = container.inner.get_fields();

        // a
        let field = get_field(&fields, "a");
        assert_eq!(field.value.ty_string, "Vec<String>");
        assert!(field.value.nested_ident.is_none());

        // b
        let field = get_field(&fields, "b");
        assert_eq!(field.value.ty_string, "Option<Vec<String>>");
        assert!(field.value.nested_ident.is_none());

        // c
        let field = get_field(&fields, "c");
        assert_eq!(field.value.ty_string, "SmallVec<String>");
        assert!(field.value.nested_ident.is_none());

        // d
        let field = get_field(&fields, "d");
        assert_eq!(field.value.ty_string, "Vec<Option<String>>");
        assert!(field.value.nested_ident.is_none());

        // e
        let field = get_field(&fields, "e");
        assert_eq!(field.value.ty_string, "Vec<Vec<String>>");
        assert!(field.value.nested_ident.is_none());
    }

    #[test]
    fn extracts_vec_types_nested() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(nested)]
                a: Vec<NestedConfig>,
                #[setting(nested)]
                b: Option<Vec<NestedConfig>>,
                #[setting(nested)]
                c: SmallVec<NestedConfig>,
                #[setting(nested = CustomNestedConfig)]
                d: Vec<Option<CustomNestedConfig>>,
                #[setting(nested)]
                e: Vec<Vec<NestedConfig>>,
            }
        });
        let fields = container.inner.get_fields();

        // a
        let field = get_field(&fields, "a");
        assert_eq!(field.value.ty_string, "Vec<NestedConfig>");
        assert_eq!(get_field_nested_ident(&field).to_string(), "NestedConfig");

        // b
        let field = get_field(&fields, "b");
        assert_eq!(field.value.ty_string, "Option<Vec<NestedConfig>>");
        assert_eq!(get_field_nested_ident(&field).to_string(), "NestedConfig");

        // c
        let field = get_field(&fields, "c");
        assert_eq!(field.value.ty_string, "SmallVec<NestedConfig>");
        assert_eq!(get_field_nested_ident(&field).to_string(), "NestedConfig");

        // d
        let field = get_field(&fields, "d");
        assert_eq!(field.value.ty_string, "Vec<Option<CustomNestedConfig>>");
        assert_eq!(
            get_field_nested_ident(&field).to_string(),
            "CustomNestedConfig"
        );

        // e
        let field = get_field(&fields, "e");
        assert_eq!(field.value.ty_string, "Vec<Vec<NestedConfig>>");
        assert_eq!(get_field_nested_ident(&field).to_string(), "NestedConfig");
    }
}
