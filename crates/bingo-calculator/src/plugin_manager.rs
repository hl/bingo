use crate::plugin::CalculatorPlugin;
use std::collections::HashMap;

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn CalculatorPlugin>>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        Self { plugins: HashMap::new() }
    }

    pub fn register(&mut self, plugin: Box<dyn CalculatorPlugin>) {
        self.plugins.insert(plugin.name().to_string(), plugin);
    }

    pub fn get(&self, name: &str) -> Option<&dyn CalculatorPlugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }
}
