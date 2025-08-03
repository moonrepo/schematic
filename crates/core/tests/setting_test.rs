use schematic_core::container::Container;
use schematic_core::field::Field;
use schematic_core::field_value::Layer;
use syn::{Ident, parse_quote};

fn get_field<'a>(fields: &'a [&'a Field], key: &str) -> &'a Field {
    fields
        .iter()
        .find(|field| field.ident.as_ref().is_some_and(|ident| ident == key))
        .unwrap()
}

fn get_field_nested_ident(field: &Field) -> &Ident {
    field.value.nested_ident.as_ref().unwrap()
}

mod setting_field {
    use super::*;

    #[test]
    fn basic_args() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(exclude, extend, required)]
                a: String,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.exclude);
        assert!(field.args.extend);
        assert!(field.args.required);
    }

    #[test]
    fn extracts_layers() {
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
        assert_eq!(field.value.layers, vec![]);

        // b
        let field = get_field(&fields, "b");
        assert_eq!(field.value.ty_string, "Option<bool>");
        assert_eq!(field.value.layers, vec![Layer::Option]);

        // c
        let field = get_field(&fields, "c");
        assert_eq!(field.value.ty_string, "Arc<bool>");
        assert_eq!(field.value.layers, vec![Layer::Arc]);

        // d
        let field = get_field(&fields, "d");
        assert_eq!(field.value.ty_string, "Box<bool>");
        assert_eq!(field.value.layers, vec![Layer::Box]);

        // e
        let field = get_field(&fields, "e");
        assert_eq!(field.value.ty_string, "Rc<bool>");
        assert_eq!(field.value.layers, vec![Layer::Rc]);

        // f
        let field = get_field(&fields, "f");
        assert_eq!(field.value.ty_string, "Option<Arc<bool>>");
        assert_eq!(field.value.layers, vec![Layer::Option, Layer::Arc]);

        // g
        let field = get_field(&fields, "g");
        assert_eq!(field.value.ty_string, "Option<Box<bool>>");
        assert_eq!(field.value.layers, vec![Layer::Option, Layer::Box]);

        // h
        let field = get_field(&fields, "h");
        assert_eq!(field.value.ty_string, "Option<Rc<bool>>");
        assert_eq!(field.value.layers, vec![Layer::Option, Layer::Rc]);

        // i
        let field = get_field(&fields, "i");
        assert_eq!(field.value.ty_string, "Arc<Option<bool>>");
        assert_eq!(field.value.layers, vec![Layer::Arc, Layer::Option]);

        // j
        let field = get_field(&fields, "j");
        assert_eq!(field.value.ty_string, "Box<Option<bool>>");
        assert_eq!(field.value.layers, vec![Layer::Box, Layer::Option]);

        // k
        let field = get_field(&fields, "k");
        assert_eq!(field.value.ty_string, "Rc<Option<bool>>");
        assert_eq!(field.value.layers, vec![Layer::Rc, Layer::Option]);

        // l
        let field = get_field(&fields, "l");
        assert_eq!(field.value.ty_string, "Option<Arc<Option<bool>>>");
        assert_eq!(
            field.value.layers,
            vec![Layer::Option, Layer::Arc, Layer::Option]
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
                e: Vec<SmallVec<String>>,
            }
        });
        let fields = container.inner.get_fields();

        // a
        let field = get_field(&fields, "a");
        assert_eq!(field.value.ty_string, "Vec<String>");
        assert_eq!(field.value.layers, vec![Layer::Vec("Vec".into())]);
        assert!(field.value.nested_ident.is_none());

        // b
        let field = get_field(&fields, "b");
        assert_eq!(field.value.ty_string, "Option<Vec<String>>");
        assert_eq!(
            field.value.layers,
            vec![Layer::Option, Layer::Vec("Vec".into())]
        );
        assert!(field.value.nested_ident.is_none());

        // c
        let field = get_field(&fields, "c");
        assert_eq!(field.value.ty_string, "SmallVec<String>");
        assert_eq!(field.value.layers, vec![Layer::Vec("SmallVec".into())]);
        assert!(field.value.nested_ident.is_none());

        // d
        let field = get_field(&fields, "d");
        assert_eq!(field.value.ty_string, "Vec<Option<String>>");
        assert_eq!(
            field.value.layers,
            vec![Layer::Vec("Vec".into()), Layer::Option]
        );
        assert!(field.value.nested_ident.is_none());

        // e
        let field = get_field(&fields, "e");
        assert_eq!(field.value.ty_string, "Vec<SmallVec<String>>");
        assert_eq!(
            field.value.layers,
            vec![Layer::Vec("Vec".into()), Layer::Vec("SmallVec".into())]
        );
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
                e: Vec<SmallVec<NestedConfig>>,
            }
        });
        let fields = container.inner.get_fields();

        // a
        let field = get_field(&fields, "a");
        assert_eq!(field.value.ty_string, "Vec<NestedConfig>");
        assert_eq!(field.value.layers, vec![Layer::Vec("Vec".into())]);
        assert_eq!(get_field_nested_ident(field).to_string(), "NestedConfig");

        // b
        let field = get_field(&fields, "b");
        assert_eq!(field.value.ty_string, "Option<Vec<NestedConfig>>");
        assert_eq!(
            field.value.layers,
            vec![Layer::Option, Layer::Vec("Vec".into())]
        );
        assert_eq!(get_field_nested_ident(field).to_string(), "NestedConfig");

        // c
        let field = get_field(&fields, "c");
        assert_eq!(field.value.ty_string, "SmallVec<NestedConfig>");
        assert_eq!(field.value.layers, vec![Layer::Vec("SmallVec".into())]);
        assert_eq!(get_field_nested_ident(field).to_string(), "NestedConfig");

        // d
        let field = get_field(&fields, "d");
        assert_eq!(field.value.ty_string, "Vec<Option<CustomNestedConfig>>");
        assert_eq!(
            field.value.layers,
            vec![Layer::Vec("Vec".into()), Layer::Option]
        );
        assert_eq!(
            get_field_nested_ident(field).to_string(),
            "CustomNestedConfig"
        );

        // e
        let field = get_field(&fields, "e");
        assert_eq!(field.value.ty_string, "Vec<SmallVec<NestedConfig>>");
        assert_eq!(
            field.value.layers,
            vec![Layer::Vec("Vec".into()), Layer::Vec("SmallVec".into())]
        );
        assert_eq!(get_field_nested_ident(field).to_string(), "NestedConfig");
    }

    #[test]
    fn extracts_set_types() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                a: HashSet<String>,
                b: Option<HashSet<String>>,
                c: BTreeSet<String>,
                d: HashSet<Option<String>>,
                e: HashSet<FxHashSet<String>>,
            }
        });
        let fields = container.inner.get_fields();

        // a
        let field = get_field(&fields, "a");
        assert_eq!(field.value.ty_string, "HashSet<String>");
        assert_eq!(field.value.layers, vec![Layer::Set("HashSet".into())]);
        assert!(field.value.nested_ident.is_none());

        // b
        let field = get_field(&fields, "b");
        assert_eq!(field.value.ty_string, "Option<HashSet<String>>");
        assert_eq!(
            field.value.layers,
            vec![Layer::Option, Layer::Set("HashSet".into())]
        );
        assert!(field.value.nested_ident.is_none());

        // c
        let field = get_field(&fields, "c");
        assert_eq!(field.value.ty_string, "BTreeSet<String>");
        assert_eq!(field.value.layers, vec![Layer::Set("BTreeSet".into())]);
        assert!(field.value.nested_ident.is_none());

        // d
        let field = get_field(&fields, "d");
        assert_eq!(field.value.ty_string, "HashSet<Option<String>>");
        assert_eq!(
            field.value.layers,
            vec![Layer::Set("HashSet".into()), Layer::Option]
        );
        assert!(field.value.nested_ident.is_none());

        // e
        let field = get_field(&fields, "e");
        assert_eq!(field.value.ty_string, "HashSet<FxHashSet<String>>");
        assert_eq!(
            field.value.layers,
            vec![Layer::Set("HashSet".into()), Layer::Set("FxHashSet".into())]
        );
        assert!(field.value.nested_ident.is_none());
    }

    #[test]
    fn extracts_set_types_nested() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(nested)]
                a: HashSet<NestedConfig>,
                #[setting(nested)]
                b: Option<HashSet<NestedConfig>>,
                #[setting(nested)]
                c: BTreeSet<NestedConfig>,
                #[setting(nested = CustomNestedConfig)]
                d: HashSet<Option<CustomNestedConfig>>,
                #[setting(nested)]
                e: HashSet<FxHashSet<NestedConfig>>,
            }
        });
        let fields = container.inner.get_fields();

        // a
        let field = get_field(&fields, "a");
        assert_eq!(field.value.ty_string, "HashSet<NestedConfig>");
        assert_eq!(field.value.layers, vec![Layer::Set("HashSet".into())]);
        assert_eq!(get_field_nested_ident(field).to_string(), "NestedConfig");

        // b
        let field = get_field(&fields, "b");
        assert_eq!(field.value.ty_string, "Option<HashSet<NestedConfig>>");
        assert_eq!(
            field.value.layers,
            vec![Layer::Option, Layer::Set("HashSet".into())]
        );
        assert_eq!(get_field_nested_ident(field).to_string(), "NestedConfig");

        // c
        let field = get_field(&fields, "c");
        assert_eq!(field.value.ty_string, "BTreeSet<NestedConfig>");
        assert_eq!(field.value.layers, vec![Layer::Set("BTreeSet".into())]);
        assert_eq!(get_field_nested_ident(field).to_string(), "NestedConfig");

        // d
        let field = get_field(&fields, "d");
        assert_eq!(field.value.ty_string, "HashSet<Option<CustomNestedConfig>>");
        assert_eq!(
            field.value.layers,
            vec![Layer::Set("HashSet".into()), Layer::Option]
        );
        assert_eq!(
            get_field_nested_ident(field).to_string(),
            "CustomNestedConfig"
        );

        // e
        let field = get_field(&fields, "e");
        assert_eq!(field.value.ty_string, "HashSet<FxHashSet<NestedConfig>>");
        assert_eq!(
            field.value.layers,
            vec![Layer::Set("HashSet".into()), Layer::Set("FxHashSet".into())]
        );
        assert_eq!(get_field_nested_ident(field).to_string(), "NestedConfig");
    }

    #[test]
    fn extracts_map_types() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                a: HashMap<String, String>,
                b: Option<HashMap<String, String>>,
                c: BTreeMap<usize, String>,
                d: HashMap<String, Option<String>>,
                e: HashMap<String, FxHashMap<String, String>>,
            }
        });
        let fields = container.inner.get_fields();

        // a
        let field = get_field(&fields, "a");
        assert_eq!(field.value.ty_string, "HashMap<String, String>");
        assert_eq!(field.value.layers, vec![Layer::Map("HashMap".into())]);
        assert!(field.value.nested_ident.is_none());

        // b
        let field = get_field(&fields, "b");
        assert_eq!(field.value.ty_string, "Option<HashMap<String, String>>");
        assert_eq!(
            field.value.layers,
            vec![Layer::Option, Layer::Map("HashMap".into())]
        );
        assert!(field.value.nested_ident.is_none());

        // c
        let field = get_field(&fields, "c");
        assert_eq!(field.value.ty_string, "BTreeMap<usize, String>");
        assert_eq!(field.value.layers, vec![Layer::Map("BTreeMap".into())]);
        assert!(field.value.nested_ident.is_none());

        // d
        let field = get_field(&fields, "d");
        assert_eq!(field.value.ty_string, "HashMap<String, Option<String>>");
        assert_eq!(
            field.value.layers,
            vec![Layer::Map("HashMap".into()), Layer::Option]
        );
        assert!(field.value.nested_ident.is_none());

        // e
        let field = get_field(&fields, "e");
        assert_eq!(
            field.value.ty_string,
            "HashMap<String, FxHashMap<String, String>>"
        );
        assert_eq!(
            field.value.layers,
            vec![Layer::Map("HashMap".into()), Layer::Map("FxHashMap".into())]
        );
        assert!(field.value.nested_ident.is_none());
    }

    #[test]
    fn extracts_map_types_nested() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(nested)]
                a: HashMap<String, NestedConfig>,
                #[setting(nested)]
                b: Option<HashMap<String, NestedConfig>>,
                #[setting(nested)]
                c: BTreeMap<usize, NestedConfig>,
                #[setting(nested = CustomNestedConfig)]
                d: HashMap<String, Option<CustomNestedConfig>>,
                #[setting(nested)]
                e: HashMap<String, FxHashMap<String, NestedConfig>>,
            }
        });
        let fields = container.inner.get_fields();

        // a
        let field = get_field(&fields, "a");
        assert_eq!(field.value.ty_string, "HashMap<String, NestedConfig>");
        assert_eq!(field.value.layers, vec![Layer::Map("HashMap".into())]);
        assert_eq!(get_field_nested_ident(field).to_string(), "NestedConfig");

        // b
        let field = get_field(&fields, "b");
        assert_eq!(
            field.value.ty_string,
            "Option<HashMap<String, NestedConfig>>"
        );
        assert_eq!(
            field.value.layers,
            vec![Layer::Option, Layer::Map("HashMap".into())]
        );
        assert_eq!(get_field_nested_ident(field).to_string(), "NestedConfig");

        // c
        let field = get_field(&fields, "c");
        assert_eq!(field.value.ty_string, "BTreeMap<usize, NestedConfig>");
        assert_eq!(field.value.layers, vec![Layer::Map("BTreeMap".into())]);
        assert_eq!(get_field_nested_ident(field).to_string(), "NestedConfig");

        // d
        let field = get_field(&fields, "d");
        assert_eq!(
            field.value.ty_string,
            "HashMap<String, Option<CustomNestedConfig>>"
        );
        assert_eq!(
            field.value.layers,
            vec![Layer::Map("HashMap".into()), Layer::Option]
        );
        assert_eq!(
            get_field_nested_ident(field).to_string(),
            "CustomNestedConfig"
        );

        // e
        let field = get_field(&fields, "e");
        assert_eq!(
            field.value.ty_string,
            "HashMap<String, FxHashMap<String, NestedConfig>>"
        );
        assert_eq!(
            field.value.layers,
            vec![Layer::Map("HashMap".into()), Layer::Map("FxHashMap".into())]
        );
        assert_eq!(get_field_nested_ident(field).to_string(), "NestedConfig");
    }
}
