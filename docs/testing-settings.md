# Testing Settings

This document details the different testing settings and environments used in the Bingo codebase.

## Performance Test Environments

The performance testing framework uses the `BINGO_PERF_ENV` environment variable to determine which performance profile to use. The following profiles are available:

-   **`local`**: The default profile for local development. Uses the strictest thresholds.
-   **`ci`**: The profile used in the CI environment. Uses relaxed thresholds to account for the slower CI runners.
-   **`benchmark`**: A profile for running benchmarks. Uses the most relaxed thresholds.
-   **`low_resource`**: A profile for low-resource environments. Uses the most lenient thresholds.
-   **`custom`**: A profile that allows you to set custom threshold multipliers.

## Environment Detection

The performance testing framework automatically detects the environment based on the following criteria:

1.  **`BINGO_PERF_ENV` environment variable**: If this variable is set, it will be used to determine the environment.
2.  **CI environment variables**: If common CI environment variables (e.g., `CI`, `GITHUB_ACTIONS`) are set, the `ci` profile will be used.
3.  **`BINGO_BENCHMARK` environment variable**: If this variable is set, the `benchmark` profile will be used.
4.  **Low-resource detection**: If the system is detected to have low resources (e.g., < 2 CPU cores, < 4GB RAM), the `low_resource` profile will be used.
5.  **Default**: If none of the above conditions are met, the `local` profile will be used.

## Dynamic Threshold Scaling

The performance testing framework also uses dynamic threshold scaling to adjust the performance thresholds based on the system's resources. This is done by comparing the current system's resources (CPU speed and memory) against a baseline and producing a scaling factor for the performance thresholds.
