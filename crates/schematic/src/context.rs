/// Pass the unit context, `()`, to a nested config that has none of its own,
/// whatever the context of the config it's nested within. This is the common
/// case for a config from a library, nested within an application's config.
///
/// ```ignore
/// #[derive(Config)]
/// #[config(context = AppContext)]
/// struct AppConfig {
///     #[setting(nested, context = schematic::context::unit)]
///     pub lib: LibConfig,
/// }
/// ```
pub fn unit<C>(_context: &C) -> &'static () {
    &()
}
