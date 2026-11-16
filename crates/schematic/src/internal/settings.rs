use crate::config::{Config, ConfigSetting};

/// Attach the settings of a nested config to its setting. A config that
/// holds its own type, directly or through another config, would otherwise
/// collect settings forever, so a config already being collected on this
/// thread is left without them.
pub fn with_nested_settings<T: Config>(setting: ConfigSetting) -> ConfigSetting {
    use std::cell::RefCell;

    thread_local! {
        static COLLECTING: RefCell<Vec<&'static str>> = const { RefCell::new(vec![]) };
    }

    let name = std::any::type_name::<T>();

    if COLLECTING.with_borrow(|stack| stack.contains(&name)) {
        return setting;
    }

    COLLECTING.with_borrow_mut(|stack| stack.push(name));

    let nested = T::settings();

    COLLECTING.with_borrow_mut(|stack| stack.pop());

    setting.nested(nested)
}
