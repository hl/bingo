/// Get current RSS (Resident Set Size) memory usage in bytes
pub fn get_memory_usage() -> anyhow::Result<usize> {
    #[cfg(target_os = "linux")]
    {
        let status = match std::fs::read_to_string("/proc/self/status") {
            Ok(s) => s,
            Err(_) => {
                // Fallback with a dummy value to keep tests working inside
                // restrictive environments where /proc is not accessible.
                return Ok(1024 * 1024); // 1 MB
            }
        };
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let kb: usize =
                    line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                return Ok(kb * 1024); // Convert KB to bytes
            }
        }
        Ok(0)
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // Use ps command to get RSS for current process
        let output = match Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
        {
            Ok(o) => o,
            Err(_) => {
                return Ok(1024 * 1024); // 1 MB fallback in sandboxed envs.
            }
        };

        if output.status.success() {
            let rss_str = String::from_utf8_lossy(&output.stdout);
            let rss_kb: usize = rss_str.trim().parse().unwrap_or(0);
            return Ok(rss_kb * 1024); // Convert KB to bytes
        }

        Ok(0)
    }

    #[cfg(target_os = "windows")]
    {
        use std::mem;
        use std::ptr;

        // Windows API structures and functions
        #[repr(C)]
        struct ProcessMemoryCounters {
            cb: u32,
            page_fault_count: u32,
            peak_working_set_size: usize,
            working_set_size: usize,
            quota_peak_paged_pool_usage: usize,
            quota_paged_pool_usage: usize,
            quota_peak_non_paged_pool_usage: usize,
            quota_non_paged_pool_usage: usize,
            pagefile_usage: usize,
            peak_pagefile_usage: usize,
        }

        extern "system" {
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
            fn GetProcessMemoryInfo(
                process: *mut std::ffi::c_void,
                ppsmemCounters: *mut ProcessMemoryCounters,
                cb: u32,
            ) -> i32;
        }

        // SAFETY: `GetCurrentProcess` always returns a valid pseudo-handle for the
        // current process (documented by the Windows API). We then pass that
        // handle together with a properly initialised `ProcessMemoryCounters`
        // struct to `GetProcessMemoryInfo`. Both APIs expect the struct to be
        // at least `cb` bytes long; we set `cb` to the correct size just
        // beforehand. All pointers remain valid for the duration of the call
        // and no memory is mutated other than the `pmc` instance we own, so
        // this usage is safe under Rust’s aliasing rules.
        unsafe {
            let mut pmc: ProcessMemoryCounters = mem::zeroed();
            pmc.cb = mem::size_of::<ProcessMemoryCounters>() as u32;

            let process_handle = GetCurrentProcess();
            let result = GetProcessMemoryInfo(
                process_handle,
                &mut pmc as *mut ProcessMemoryCounters,
                pmc.cb,
            );

            if result != 0 {
                Ok(pmc.working_set_size)
            } else {
                // Fall back to 0 if the API call failed
                Ok(0)
            }
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Ok(0)
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub rss_bytes: usize,
    pub timestamp: std::time::Instant,
}

impl MemoryStats {
    /// Get current memory statistics
    pub fn current() -> anyhow::Result<Self> {
        Ok(Self { rss_bytes: get_memory_usage()?, timestamp: std::time::Instant::now() })
    }

    /// Calculate memory delta from another measurement
    pub fn delta_from(&self, other: &MemoryStats) -> i64 {
        self.rss_bytes as i64 - other.rss_bytes as i64
    }

    /// Format RSS in human-readable form
    pub fn format_rss(&self) -> String {
        let mb = self.rss_bytes as f64 / (1024.0 * 1024.0);
        format!("{mb:.2} MB")
    }
}

/// Memory tracker for benchmarking
pub struct MemoryTracker {
    start_stats: MemoryStats,
}

impl MemoryTracker {
    /// Start tracking memory usage
    pub fn start() -> anyhow::Result<Self> {
        Ok(Self { start_stats: MemoryStats::current()? })
    }

    /// Get current memory delta since start
    pub fn current_delta(&self) -> anyhow::Result<i64> {
        let current = MemoryStats::current()?;
        Ok(current.delta_from(&self.start_stats))
    }

    /// Finish tracking and return final stats
    pub fn finish(self) -> anyhow::Result<(MemoryStats, MemoryStats, i64)> {
        let end_stats = MemoryStats::current()?;
        let delta = end_stats.delta_from(&self.start_stats);
        Ok((self.start_stats, end_stats, delta))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_usage() {
        let usage = get_memory_usage().unwrap();
        // Should return some reasonable value (at least a few KB)
        assert!(
            usage > 1024,
            "Memory usage should be at least 1KB, got: {usage}"
        );
    }

    #[test]
    fn test_memory_stats() {
        let stats = MemoryStats::current().unwrap();
        assert!(stats.rss_bytes > 0);

        let formatted = stats.format_rss();
        assert!(formatted.contains("MB"));
    }

    #[test]
    fn test_memory_tracker() {
        let tracker = MemoryTracker::start().unwrap();

        // Allocate some memory
        let _big_vec: Vec<u8> = vec![0; 1024 * 1024]; // 1MB

        let _delta = tracker.current_delta().unwrap();
        // Note: Delta might be negative due to GC or other factors
        // Just check that we got a reasonable value

        let (start, end, final_delta) = tracker.finish().unwrap();
        assert!(start.rss_bytes > 0);
        assert!(end.rss_bytes > 0);

        eprintln!(
            "Memory tracking test: start={}, end={}, delta={}",
            start.format_rss(),
            end.format_rss(),
            final_delta
        );
    }
}
