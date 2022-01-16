use crate::plugin::{EventContext, PluginHandler, Plugin};

pub struct PluginContext;

impl PluginHandler for Plugin<PluginContext> {
}

pub fn load_window_mapper_plugin() -> Plugin<PluginContext> {
    Plugin {
        context: PluginContext {},
    }
}
