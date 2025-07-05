//! Advanced Fact Stream Processing
//!
//! This module provides sophisticated stream processing capabilities for time-series
//! and event processing, including windowed aggregations, temporal pattern matching,
//! and out-of-order event handling.

use super::types::{Fact, FactValue};
use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Represents a timestamp for stream processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Create timestamp from system time
    pub fn now() -> Self {
        Self(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64)
    }

    /// Create timestamp from milliseconds since epoch
    pub fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    /// Get milliseconds since epoch
    pub fn as_millis(&self) -> u64 {
        self.0
    }

    /// Add duration to timestamp
    pub fn add_duration(&self, duration: Duration) -> Self {
        Self(self.0 + duration.as_millis() as u64)
    }

    /// Subtract duration from timestamp
    pub fn sub_duration(&self, duration: Duration) -> Self {
        Self(self.0.saturating_sub(duration.as_millis() as u64))
    }
}

/// Window specification for stream processing
#[derive(Debug, Clone, PartialEq)]
pub enum WindowSpec {
    /// Tumbling window with fixed size
    Tumbling {
        /// Window size in milliseconds
        size: Duration,
    },
    /// Sliding window with size and advance interval
    Sliding {
        /// Window size in milliseconds
        size: Duration,
        /// How often the window advances
        advance: Duration,
    },
    /// Session window with gap timeout
    Session {
        /// Maximum gap between events to consider them in the same session
        gap_timeout: Duration,
    },
}

/// Aggregation function for windowed operations
#[derive(Debug, Clone, PartialEq)]
pub enum AggregationFunction {
    /// Count the number of facts in the window
    Count,
    /// Sum numeric field values
    Sum { field: String },
    /// Average numeric field values
    Average { field: String },
    /// Minimum field value
    Min { field: String },
    /// Maximum field value
    Max { field: String },
    /// Collect distinct values
    Distinct { field: String },
    /// Standard deviation of numeric field values
    StandardDeviation { field: String },
    /// Variance of numeric field values
    Variance { field: String },
    /// Percentile of numeric field values (0.0 to 100.0)
    Percentile { field: String, percentile: f64 },
    /// Median (50th percentile) of numeric field values
    Median { field: String },
    /// Custom aggregation with calculator expression
    Custom { field: String, expression: String },
}

/// Window instance containing facts and metadata
#[derive(Debug, Clone)]
pub struct WindowInstance {
    /// Unique identifier for this window
    pub id: String,
    /// Window start timestamp
    pub start_time: Timestamp,
    /// Window end timestamp
    pub end_time: Timestamp,
    /// Facts contained in this window
    pub facts: Vec<Fact>,
    /// Whether this window is complete and ready for processing
    pub is_complete: bool,
    /// Aggregation results cache
    pub aggregation_cache: HashMap<String, FactValue>,
}

impl WindowInstance {
    /// Create a new window instance
    pub fn new(id: String, start_time: Timestamp, end_time: Timestamp) -> Self {
        Self {
            id,
            start_time,
            end_time,
            facts: Vec::new(),
            is_complete: false,
            aggregation_cache: HashMap::new(),
        }
    }

    /// Add a fact to this window
    pub fn add_fact(&mut self, fact: Fact) {
        self.facts.push(fact);
        // Clear cache when facts are added
        self.aggregation_cache.clear();
    }

    /// Check if a timestamp falls within this window
    pub fn contains_timestamp(&self, ts: Timestamp) -> bool {
        ts >= self.start_time && ts < self.end_time
    }

    /// Compute aggregation for this window
    pub fn compute_aggregation(&mut self, func: &AggregationFunction) -> anyhow::Result<FactValue> {
        let cache_key = format!("{func:?}");

        // Return cached result if available
        if let Some(cached_result) = self.aggregation_cache.get(&cache_key) {
            return Ok(cached_result.clone());
        }

        let result = match func {
            AggregationFunction::Count => FactValue::Integer(self.facts.len() as i64),
            AggregationFunction::Sum { field } => {
                let mut sum = 0.0;
                for fact in &self.facts {
                    if let Some(value) = fact.data.fields.get(field) {
                        match value {
                            FactValue::Integer(i) => sum += *i as f64,
                            FactValue::Float(f) => sum += f,
                            _ => continue,
                        }
                    }
                }
                if sum.fract() == 0.0 {
                    FactValue::Integer(sum as i64)
                } else {
                    FactValue::Float(sum)
                }
            }
            AggregationFunction::Average { field } => {
                let mut sum = 0.0;
                let mut count = 0;
                for fact in &self.facts {
                    if let Some(value) = fact.data.fields.get(field) {
                        match value {
                            FactValue::Integer(i) => {
                                sum += *i as f64;
                                count += 1;
                            }
                            FactValue::Float(f) => {
                                sum += f;
                                count += 1;
                            }
                            _ => continue,
                        }
                    }
                }
                if count > 0 {
                    FactValue::Float(sum / count as f64)
                } else {
                    FactValue::Float(0.0)
                }
            }
            AggregationFunction::Min { field } => {
                let mut min_val: Option<FactValue> = None;
                for fact in &self.facts {
                    if let Some(value) = fact.data.fields.get(field) {
                        match &min_val {
                            None => min_val = Some(value.clone()),
                            Some(current_min) => {
                                if Self::compare_values(value, current_min) < 0 {
                                    min_val = Some(value.clone());
                                }
                            }
                        }
                    }
                }
                min_val.unwrap_or(FactValue::Integer(0))
            }
            AggregationFunction::Max { field } => {
                let mut max_val: Option<FactValue> = None;
                for fact in &self.facts {
                    if let Some(value) = fact.data.fields.get(field) {
                        match &max_val {
                            None => max_val = Some(value.clone()),
                            Some(current_max) => {
                                if Self::compare_values(value, current_max) > 0 {
                                    max_val = Some(value.clone());
                                }
                            }
                        }
                    }
                }
                max_val.unwrap_or(FactValue::Integer(0))
            }
            AggregationFunction::Distinct { field } => {
                let mut distinct_values = std::collections::HashSet::new();
                for fact in &self.facts {
                    if let Some(value) = fact.data.fields.get(field) {
                        distinct_values.insert(value.clone());
                    }
                }
                FactValue::Integer(distinct_values.len() as i64)
            }
            AggregationFunction::StandardDeviation { field } => {
                let mut values = Vec::new();
                for fact in &self.facts {
                    if let Some(value) = fact.data.fields.get(field) {
                        match value {
                            FactValue::Integer(i) => values.push(*i as f64),
                            FactValue::Float(f) => values.push(*f),
                            _ => continue,
                        }
                    }
                }

                if values.is_empty() {
                    FactValue::Float(0.0)
                } else {
                    let mean = values.iter().sum::<f64>() / values.len() as f64;
                    let variance = values.iter().map(|v| (*v - mean).powi(2)).sum::<f64>()
                        / values.len() as f64;
                    FactValue::Float(variance.sqrt())
                }
            }
            AggregationFunction::Variance { field } => {
                let mut values = Vec::new();
                for fact in &self.facts {
                    if let Some(value) = fact.data.fields.get(field) {
                        match value {
                            FactValue::Integer(i) => values.push(*i as f64),
                            FactValue::Float(f) => values.push(*f),
                            _ => continue,
                        }
                    }
                }

                if values.is_empty() {
                    FactValue::Float(0.0)
                } else {
                    let mean = values.iter().sum::<f64>() / values.len() as f64;
                    let variance = values.iter().map(|v| (*v - mean).powi(2)).sum::<f64>()
                        / values.len() as f64;
                    FactValue::Float(variance)
                }
            }
            AggregationFunction::Percentile { field, percentile } => {
                let mut values = Vec::new();
                for fact in &self.facts {
                    if let Some(value) = fact.data.fields.get(field) {
                        match value {
                            FactValue::Integer(i) => values.push(*i as f64),
                            FactValue::Float(f) => values.push(*f),
                            _ => continue,
                        }
                    }
                }

                if values.is_empty() {
                    FactValue::Float(0.0)
                } else {
                    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    let rank_f = (percentile / 100.0) * (values.len() as f64 - 1.0);
                    let lower = rank_f.floor() as usize;
                    let upper = rank_f.ceil() as usize;

                    let result = if upper == lower || lower >= values.len() {
                        values[lower.min(values.len() - 1)]
                    } else {
                        let weight = rank_f - lower as f64;
                        values[lower] * (1.0 - weight) + values[upper] * weight
                    };

                    FactValue::Float(result)
                }
            }
            AggregationFunction::Median { field } => {
                let mut values = Vec::new();
                for fact in &self.facts {
                    if let Some(value) = fact.data.fields.get(field) {
                        match value {
                            FactValue::Integer(i) => values.push(*i as f64),
                            FactValue::Float(f) => values.push(*f),
                            _ => continue,
                        }
                    }
                }

                if values.is_empty() {
                    FactValue::Float(0.0)
                } else {
                    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    let mid = values.len() / 2;

                    let median = if values.len() % 2 == 0 {
                        // Even number of values - average of two middle values
                        (values[mid - 1] + values[mid]) / 2.0
                    } else {
                        // Odd number of values - middle value
                        values[mid]
                    };

                    FactValue::Float(median)
                }
            }
            AggregationFunction::Custom { field: _, expression: _ } => {
                // Custom aggregation requires calculator integration
                // Currently returns count as default implementation
                FactValue::Integer(self.facts.len() as i64)
            }
        };

        // Cache the result
        self.aggregation_cache.insert(cache_key, result.clone());
        Ok(result)
    }

    /// Compare two FactValues for ordering
    fn compare_values(a: &FactValue, b: &FactValue) -> i32 {
        match (a, b) {
            (FactValue::Integer(a), FactValue::Integer(b)) => a.cmp(b) as i32,
            (FactValue::Float(a), FactValue::Float(b)) => {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal) as i32
            }
            (FactValue::Integer(a), FactValue::Float(b)) => {
                (*a as f64).partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal) as i32
            }
            (FactValue::Float(a), FactValue::Integer(b)) => {
                a.partial_cmp(&(*b as f64)).unwrap_or(std::cmp::Ordering::Equal) as i32
            }
            (FactValue::String(a), FactValue::String(b)) => a.cmp(b) as i32,
            (FactValue::Boolean(a), FactValue::Boolean(b)) => a.cmp(b) as i32,
            _ => 0, // Incomparable types are equal
        }
    }
}

/// Stream processing engine for windowed operations
#[derive(Debug)]
pub struct StreamProcessor {
    /// Active windows organized by window spec
    windows: HashMap<String, Vec<WindowInstance>>,
    /// Window specifications by name
    window_specs: HashMap<String, WindowSpec>,
    /// Current watermark for event time processing
    watermark: Timestamp,
    /// Buffer for out-of-order events
    event_buffer: BTreeMap<Timestamp, Vec<Fact>>,
    /// Maximum allowed lateness for events
    max_lateness: Duration,
    /// Statistics for monitoring
    pub stats: StreamProcessingStats,
}

/// Statistics for stream processing monitoring
#[derive(Debug, Default, Clone)]
pub struct StreamProcessingStats {
    pub events_processed: usize,
    pub windows_created: usize,
    pub windows_completed: usize,
    pub late_events_dropped: usize,
    pub aggregations_computed: usize,
    pub watermark_updates: usize,
}

impl StreamProcessor {
    /// Create a new stream processor
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            window_specs: HashMap::new(),
            watermark: Timestamp::from_millis(0),
            event_buffer: BTreeMap::new(),
            max_lateness: Duration::from_secs(60), // 1 minute default
            stats: StreamProcessingStats::default(),
        }
    }

    /// Configure maximum lateness for out-of-order events
    pub fn set_max_lateness(&mut self, lateness: Duration) {
        self.max_lateness = lateness;
    }

    /// Register a window specification
    pub fn register_window(&mut self, name: String, spec: WindowSpec) {
        self.window_specs.insert(name.clone(), spec);
        self.windows.insert(name, Vec::new());
    }

    /// Process a fact through the stream processor
    pub fn process_fact(
        &mut self,
        fact: Fact,
        event_time: Option<Timestamp>,
    ) -> anyhow::Result<Vec<WindowInstance>> {
        self.stats.events_processed += 1;

        // Extract or assign event time
        let timestamp = event_time.unwrap_or_else(|| {
            // Try to extract timestamp from fact
            self.extract_timestamp_from_fact(&fact).unwrap_or_else(Timestamp::now)
        });

        // Check if event is too late
        if timestamp.add_duration(self.max_lateness) < self.watermark {
            self.stats.late_events_dropped += 1;
            return Ok(Vec::new());
        }

        // Add to event buffer for processing
        self.event_buffer.entry(timestamp).or_default().push(fact);

        // Process buffered events up to current watermark
        self.process_buffered_events()
    }

    /// Update watermark and trigger window completion
    pub fn update_watermark(
        &mut self,
        new_watermark: Timestamp,
    ) -> anyhow::Result<Vec<WindowInstance>> {
        if new_watermark > self.watermark {
            self.watermark = new_watermark;
            self.stats.watermark_updates += 1;

            // Process any buffered events that are now ready
            self.process_buffered_events()
        } else {
            Ok(Vec::new())
        }
    }

    /// Process buffered events up to the current watermark
    fn process_buffered_events(&mut self) -> anyhow::Result<Vec<WindowInstance>> {
        let mut completed_windows = Vec::new();

        // Process events in timestamp order up to watermark
        let cutoff_time = self.watermark;
        let ready_timestamps: Vec<Timestamp> =
            self.event_buffer.range(..=cutoff_time).map(|(ts, _)| *ts).collect();

        for timestamp in ready_timestamps {
            if let Some(facts) = self.event_buffer.remove(&timestamp) {
                for fact in facts {
                    // Collect all window specs first to avoid borrowing conflicts
                    let window_specs: Vec<(String, WindowSpec)> = self
                        .window_specs
                        .iter()
                        .map(|(name, spec)| (name.clone(), spec.clone()))
                        .collect();

                    // Process each window spec
                    for (window_name, window_spec) in window_specs {
                        let windows_for_spec = self.windows.get_mut(&window_name).unwrap();
                        Self::assign_fact_to_windows_static(
                            fact.clone(),
                            timestamp,
                            &window_spec,
                            windows_for_spec,
                            &mut self.stats,
                        )?;
                    }
                }
            }
        }

        // Check for completed windows
        for windows_for_spec in self.windows.values_mut() {
            let mut i = 0;
            while i < windows_for_spec.len() {
                if windows_for_spec[i].end_time <= self.watermark
                    && !windows_for_spec[i].is_complete
                {
                    windows_for_spec[i].is_complete = true;
                    self.stats.windows_completed += 1;
                    completed_windows.push(windows_for_spec[i].clone());
                }
                i += 1;
            }
        }

        Ok(completed_windows)
    }

    /// Assign a fact to appropriate windows based on window specification (static version)
    fn assign_fact_to_windows_static(
        fact: Fact,
        timestamp: Timestamp,
        window_spec: &WindowSpec,
        windows: &mut Vec<WindowInstance>,
        stats: &mut StreamProcessingStats,
    ) -> anyhow::Result<()> {
        match window_spec {
            WindowSpec::Tumbling { size } => {
                Self::assign_to_tumbling_window_static(fact, timestamp, *size, windows, stats)?;
            }
            WindowSpec::Sliding { size, advance } => {
                Self::assign_to_sliding_windows_static(
                    fact, timestamp, *size, *advance, windows, stats,
                )?;
            }
            WindowSpec::Session { gap_timeout } => {
                Self::assign_to_session_window_static(
                    fact,
                    timestamp,
                    *gap_timeout,
                    windows,
                    stats,
                )?;
            }
        }
        Ok(())
    }

    /// Assign fact to tumbling window (static version)
    fn assign_to_tumbling_window_static(
        fact: Fact,
        timestamp: Timestamp,
        window_size: Duration,
        windows: &mut Vec<WindowInstance>,
        stats: &mut StreamProcessingStats,
    ) -> anyhow::Result<()> {
        let window_size_millis = window_size.as_millis() as u64;
        let window_start = Timestamp::from_millis(
            (timestamp.as_millis() / window_size_millis) * window_size_millis,
        );
        let window_end = window_start.add_duration(window_size);

        // Find existing window or create new one
        if let Some(window) = windows.iter_mut().find(|w| w.start_time == window_start) {
            window.add_fact(fact);
        } else {
            let window_id = format!(
                "tumbling_{}_{}",
                window_start.as_millis(),
                window_end.as_millis()
            );
            let mut new_window = WindowInstance::new(window_id, window_start, window_end);
            new_window.add_fact(fact);
            windows.push(new_window);
            stats.windows_created += 1;
        }

        Ok(())
    }

    /// Assign fact to sliding windows (static version)
    fn assign_to_sliding_windows_static(
        fact: Fact,
        timestamp: Timestamp,
        window_size: Duration,
        advance: Duration,
        windows: &mut Vec<WindowInstance>,
        stats: &mut StreamProcessingStats,
    ) -> anyhow::Result<()> {
        let _window_size_millis = window_size.as_millis() as u64;
        let advance_millis = advance.as_millis() as u64;

        // Find all sliding windows that should contain this timestamp
        let earliest_start = timestamp.sub_duration(window_size);
        let latest_start = timestamp;

        let mut current_start =
            Timestamp::from_millis((earliest_start.as_millis() / advance_millis) * advance_millis);

        while current_start <= latest_start {
            let window_end = current_start.add_duration(window_size);

            if current_start <= timestamp && timestamp < window_end {
                // Find existing window or create new one
                if let Some(window) = windows.iter_mut().find(|w| w.start_time == current_start) {
                    window.add_fact(fact.clone());
                } else {
                    let window_id = format!(
                        "sliding_{}_{}",
                        current_start.as_millis(),
                        window_end.as_millis()
                    );
                    let mut new_window = WindowInstance::new(window_id, current_start, window_end);
                    new_window.add_fact(fact.clone());
                    windows.push(new_window);
                    stats.windows_created += 1;
                }
            }

            current_start = current_start.add_duration(advance);
        }

        Ok(())
    }

    /// Assign fact to session window (static version)
    fn assign_to_session_window_static(
        fact: Fact,
        timestamp: Timestamp,
        gap_timeout: Duration,
        windows: &mut Vec<WindowInstance>,
        stats: &mut StreamProcessingStats,
    ) -> anyhow::Result<()> {
        // Find session window that can be extended or create new one
        let mut assigned = false;

        for window in windows.iter_mut() {
            // Check if this fact can extend an existing session
            if !window.is_complete && timestamp >= window.start_time {
                // Find the actual last event time in the session by checking all facts
                let last_event_time = window
                    .facts
                    .iter()
                    .filter_map(|f| f.data.fields.get("timestamp"))
                    .filter_map(|v| {
                        if let FactValue::Integer(ts) = v {
                            Some(*ts as u64)
                        } else {
                            None
                        }
                    })
                    .max()
                    .map(Timestamp::from_millis)
                    .unwrap_or(window.start_time);

                // Check if gap between this event and last event is within timeout
                if timestamp.as_millis()
                    <= last_event_time.as_millis() + gap_timeout.as_millis() as u64
                {
                    // Extend the session window
                    window.end_time = timestamp.add_duration(gap_timeout);
                    window.add_fact(fact.clone());
                    assigned = true;
                    break;
                }
            }
        }

        if !assigned {
            // Create new session window
            let window_id = format!("session_{}", timestamp.as_millis());
            let window_start = timestamp;
            let window_end = timestamp.add_duration(gap_timeout);
            let mut new_window = WindowInstance::new(window_id, window_start, window_end);
            new_window.add_fact(fact);
            windows.push(new_window);
            stats.windows_created += 1;
        }

        Ok(())
    }

    /// Extract timestamp from fact data
    fn extract_timestamp_from_fact(&self, fact: &Fact) -> Option<Timestamp> {
        // Try common timestamp field names
        let timestamp_fields = ["timestamp", "time", "event_time", "created_at"];

        for field_name in &timestamp_fields {
            if let Some(value) = fact.data.fields.get(*field_name) {
                match value {
                    FactValue::Integer(millis) => {
                        return Some(Timestamp::from_millis(*millis as u64));
                    }
                    _ => continue,
                }
            }
        }

        None
    }

    /// Compute aggregation for a specific window
    pub fn compute_window_aggregation(
        &mut self,
        window_name: &str,
        aggregation: &AggregationFunction,
    ) -> anyhow::Result<Vec<(String, FactValue)>> {
        let mut results = Vec::new();

        if let Some(windows) = self.windows.get_mut(window_name) {
            for window in windows.iter_mut() {
                if window.is_complete {
                    let result = window.compute_aggregation(aggregation)?;
                    results.push((window.id.clone(), result));
                    self.stats.aggregations_computed += 1;
                }
            }
        }

        Ok(results)
    }

    /// Get all completed windows for a window specification
    pub fn get_completed_windows(&self, window_name: &str) -> Vec<&WindowInstance> {
        self.windows
            .get(window_name)
            .map(|windows| windows.iter().filter(|w| w.is_complete).collect())
            .unwrap_or_default()
    }

    /// Clean up old completed windows to free memory
    pub fn cleanup_old_windows(&mut self, retention_period: Duration) {
        let cutoff_time = self.watermark.sub_duration(retention_period);

        for windows in self.windows.values_mut() {
            windows.retain(|window| window.end_time >= cutoff_time || !window.is_complete);
        }
    }

    /// Get current watermark
    pub fn get_watermark(&self) -> Timestamp {
        self.watermark
    }

    /// Get statistics
    pub fn get_stats(&self) -> &StreamProcessingStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = StreamProcessingStats::default();
    }
}

impl Default for StreamProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FactData, FactValue};
    use std::collections::HashMap;

    fn create_test_fact(id: u64, value: i64, timestamp: u64) -> Fact {
        let mut fields = HashMap::new();
        fields.insert("value".to_string(), FactValue::Integer(value));
        fields.insert(
            "timestamp".to_string(),
            FactValue::Integer(timestamp as i64),
        );

        Fact { id, external_id: None, timestamp: chrono::Utc::now(), data: FactData { fields } }
    }

    #[test]
    fn test_tumbling_window() {
        let mut processor = StreamProcessor::new();
        processor.register_window(
            "test_window".to_string(),
            WindowSpec::Tumbling { size: Duration::from_secs(10) },
        );

        // Add facts to same tumbling window
        let fact1 = create_test_fact(1, 100, 1000);
        let fact2 = create_test_fact(2, 200, 5000);
        let fact3 = create_test_fact(3, 300, 15000); // Different window

        processor.process_fact(fact1, Some(Timestamp::from_millis(1000))).unwrap();
        processor.process_fact(fact2, Some(Timestamp::from_millis(5000))).unwrap();
        processor.process_fact(fact3, Some(Timestamp::from_millis(15000))).unwrap();

        // Update watermark to complete windows
        processor.update_watermark(Timestamp::from_millis(20000)).unwrap();

        let windows = processor.get_completed_windows("test_window");
        assert_eq!(windows.len(), 2);

        // First window should have 2 facts
        let first_window = &windows[0];
        assert_eq!(first_window.facts.len(), 2);

        // Second window should have 1 fact
        let second_window = &windows[1];
        assert_eq!(second_window.facts.len(), 1);
    }

    #[test]
    fn test_aggregation_functions() {
        let mut window = WindowInstance::new(
            "test".to_string(),
            Timestamp::from_millis(0),
            Timestamp::from_millis(10000),
        );

        window.add_fact(create_test_fact(1, 10, 1000));
        window.add_fact(create_test_fact(2, 20, 2000));
        window.add_fact(create_test_fact(3, 30, 3000));

        // Test count
        let count = window.compute_aggregation(&AggregationFunction::Count).unwrap();
        assert_eq!(count, FactValue::Integer(3));

        // Test sum
        let sum = window
            .compute_aggregation(&AggregationFunction::Sum { field: "value".to_string() })
            .unwrap();
        assert_eq!(sum, FactValue::Integer(60));

        // Test average
        let avg = window
            .compute_aggregation(&AggregationFunction::Average { field: "value".to_string() })
            .unwrap();
        assert_eq!(avg, FactValue::Float(20.0));

        // Test min
        let min = window
            .compute_aggregation(&AggregationFunction::Min { field: "value".to_string() })
            .unwrap();
        assert_eq!(min, FactValue::Integer(10));

        // Test max
        let max = window
            .compute_aggregation(&AggregationFunction::Max { field: "value".to_string() })
            .unwrap();
        assert_eq!(max, FactValue::Integer(30));

        // Test standard deviation
        let stddev = window
            .compute_aggregation(&AggregationFunction::StandardDeviation {
                field: "value".to_string(),
            })
            .unwrap();
        // Standard deviation of [10, 20, 30] is sqrt(((10-20)² + (20-20)² + (30-20)²)/3) = sqrt(200/3) ≈ 8.165
        if let FactValue::Float(std_val) = stddev {
            assert!((std_val - 8.16496580927726).abs() < 0.001);
        } else {
            panic!("Expected float for standard deviation");
        }

        // Test variance
        let variance = window
            .compute_aggregation(&AggregationFunction::Variance { field: "value".to_string() })
            .unwrap();
        // Variance of [10, 20, 30] is ((10-20)² + (20-20)² + (30-20)²)/3 = 200/3 ≈ 66.667
        if let FactValue::Float(var_val) = variance {
            assert!((var_val - 66.66666666666667).abs() < 0.001);
        } else {
            panic!("Expected float for variance");
        }

        // Test percentile (75th percentile)
        let p75 = window
            .compute_aggregation(&AggregationFunction::Percentile {
                field: "value".to_string(),
                percentile: 75.0,
            })
            .unwrap();
        // 75th percentile of [10, 20, 30] should be 25.0
        if let FactValue::Float(p75_val) = p75 {
            assert!((p75_val - 25.0).abs() < 0.001);
        } else {
            panic!("Expected float for percentile");
        }

        // Test median
        let median = window
            .compute_aggregation(&AggregationFunction::Median { field: "value".to_string() })
            .unwrap();
        // Median of [10, 20, 30] should be 20.0
        if let FactValue::Float(median_val) = median {
            assert!((median_val - 20.0).abs() < 0.001);
        } else {
            panic!("Expected float for median");
        }
    }

    #[test]
    fn test_sliding_window() {
        let mut processor = StreamProcessor::new();
        processor.register_window(
            "sliding_test".to_string(),
            WindowSpec::Sliding { size: Duration::from_secs(10), advance: Duration::from_secs(5) },
        );

        // Add facts
        processor
            .process_fact(
                create_test_fact(1, 100, 1000),
                Some(Timestamp::from_millis(1000)),
            )
            .unwrap();
        processor
            .process_fact(
                create_test_fact(2, 200, 6000),
                Some(Timestamp::from_millis(6000)),
            )
            .unwrap();

        // Update watermark
        processor.update_watermark(Timestamp::from_millis(20000)).unwrap();

        let windows = processor.get_completed_windows("sliding_test");
        // Should have multiple overlapping windows
        assert!(windows.len() > 1);
    }

    #[test]
    fn test_advanced_aggregation_functions() {
        let mut window = WindowInstance::new(
            "advanced_test".to_string(),
            Timestamp::from_millis(0),
            Timestamp::from_millis(10000),
        );

        // Add test data with more variety for advanced statistics
        window.add_fact(create_test_fact(1, 5, 1000)); // Lower outlier
        window.add_fact(create_test_fact(2, 10, 2000));
        window.add_fact(create_test_fact(3, 15, 3000));
        window.add_fact(create_test_fact(4, 20, 4000));
        window.add_fact(create_test_fact(5, 25, 5000));
        window.add_fact(create_test_fact(6, 35, 6000)); // Higher outlier

        // Test distinct values
        let distinct = window
            .compute_aggregation(&AggregationFunction::Distinct { field: "value".to_string() })
            .unwrap();
        assert_eq!(distinct, FactValue::Integer(6)); // All values are distinct

        // Test standard deviation with more diverse data
        let stddev = window
            .compute_aggregation(&AggregationFunction::StandardDeviation {
                field: "value".to_string(),
            })
            .unwrap();
        // Standard deviation of [5, 10, 15, 20, 25, 35]
        // Mean = 110/6 ≈ 18.33
        // Variance = sum of (x - mean)² / n ≈ 106.67
        // StdDev = sqrt(106.67) ≈ 10.33
        if let FactValue::Float(std_val) = stddev {
            // Standard deviation of [5, 10, 15, 20, 25, 35] = 9.86...
            assert!((std_val - 9.860132971832694).abs() < 0.01);
        } else {
            panic!("Expected float for standard deviation");
        }

        // Test variance
        let variance = window
            .compute_aggregation(&AggregationFunction::Variance { field: "value".to_string() })
            .unwrap();
        if let FactValue::Float(var_val) = variance {
            // Variance of [5, 10, 15, 20, 25, 35] = 97.22...
            assert!((var_val - 97.22222222222223).abs() < 0.01);
        } else {
            panic!("Expected float for variance");
        }

        // Test different percentiles
        let p25 = window
            .compute_aggregation(&AggregationFunction::Percentile {
                field: "value".to_string(),
                percentile: 25.0,
            })
            .unwrap();
        if let FactValue::Float(p25_val) = p25 {
            assert!((p25_val - 11.25).abs() < 0.01);
        }

        let p50 = window
            .compute_aggregation(&AggregationFunction::Percentile {
                field: "value".to_string(),
                percentile: 50.0,
            })
            .unwrap();
        if let FactValue::Float(p50_val) = p50 {
            assert!((p50_val - 17.5).abs() < 0.01);
        }

        let p90 = window
            .compute_aggregation(&AggregationFunction::Percentile {
                field: "value".to_string(),
                percentile: 90.0,
            })
            .unwrap();
        if let FactValue::Float(p90_val) = p90 {
            assert!((p90_val - 30.0).abs() < 0.01);
        }

        // Test median (should be same as 50th percentile)
        let median = window
            .compute_aggregation(&AggregationFunction::Median { field: "value".to_string() })
            .unwrap();
        if let FactValue::Float(median_val) = median {
            assert!((median_val - 17.5).abs() < 0.01); // (15 + 20) / 2
        }

        // Test edge cases with empty field
        let empty_result = window
            .compute_aggregation(&AggregationFunction::StandardDeviation {
                field: "nonexistent".to_string(),
            })
            .unwrap();
        assert_eq!(empty_result, FactValue::Float(0.0));
    }

    #[test]
    fn test_late_event_handling() {
        let mut processor = StreamProcessor::new();
        processor.set_max_lateness(Duration::from_secs(5));
        processor.register_window(
            "late_test".to_string(),
            WindowSpec::Tumbling { size: Duration::from_secs(10) },
        );

        // Set watermark
        processor.update_watermark(Timestamp::from_millis(10000)).unwrap();

        // Try to add a very late event
        let late_fact = create_test_fact(1, 100, 1000);
        processor.process_fact(late_fact, Some(Timestamp::from_millis(1000))).unwrap();

        // Should be dropped due to lateness
        assert_eq!(processor.stats.late_events_dropped, 1);
    }
}
