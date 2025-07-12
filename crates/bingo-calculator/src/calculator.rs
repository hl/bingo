use crate::built_in::{
    add::AddCalculator, multiply::MultiplyCalculator, percentage_add::PercentageAddCalculator,
    percentage_deduct::PercentageDeductCalculator,
    proportional_allocator::ProportionalAllocatorCalculator,
    time_between_datetime::TimeBetweenDatetimeCalculator,
};
use crate::plugin::CalculationResult;
use crate::plugin_manager::PluginManager;
use bingo_types::FactValue;

pub struct Calculator {
    plugin_manager: PluginManager,
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Calculator {
    pub fn new() -> Self {
        let mut plugin_manager = PluginManager::new();
        plugin_manager.register(Box::new(AddCalculator));
        plugin_manager.register(Box::new(MultiplyCalculator));
        plugin_manager.register(Box::new(PercentageAddCalculator));
        plugin_manager.register(Box::new(PercentageDeductCalculator));
        plugin_manager.register(Box::new(ProportionalAllocatorCalculator));
        plugin_manager.register(Box::new(TimeBetweenDatetimeCalculator));
        Self { plugin_manager }
    }

    pub fn calculate(
        &self,
        calculator_name: &str,
        args: &std::collections::HashMap<String, &FactValue>,
    ) -> CalculationResult {
        if let Some(plugin) = self.plugin_manager.get(calculator_name) {
            plugin.calculate(args)
        } else {
            Err(format!("calculator '{calculator_name}' not found"))
        }
    }
}
