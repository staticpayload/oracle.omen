//! Resource limits for WASM execution.

use serde::{Deserialize, Serialize};

/// Resource limits for WASM tools
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum fuel (instructions)
    pub max_fuel: u64,

    /// Maximum memory pages (64KB each)
    pub max_memory_pages: u32,

    /// Maximum table elements
    pub max_table_elements: u32,

    /// Execution timeout in milliseconds
    pub timeout_ms: u64,

    /// Maximum output size in bytes
    pub max_output_bytes: usize,
}

impl ResourceLimits {
    /// Create new resource limits
    pub fn new(
        max_fuel: u64,
        max_memory_pages: u32,
        timeout_ms: u64,
    ) -> Self {
        Self {
            max_fuel,
            max_memory_pages,
            max_table_elements: 1024,
            timeout_ms,
            max_output_bytes: 1024 * 1024, // 1MB default
        }
    }

    /// Create minimal limits for testing
    pub fn minimal() -> Self {
        Self {
            max_fuel: 10_000,
            max_memory_pages: 1,
            max_table_elements: 10,
            timeout_ms: 100,
            max_output_bytes: 1024,
        }
    }

    /// Create generous limits
    pub fn generous() -> Self {
        Self {
            max_fuel: 10_000_000,
            max_memory_pages: 64, // 4MB
            max_table_elements: 4096,
            timeout_ms: 30_000,
            max_output_bytes: 10 * 1024 * 1024, // 10MB
        }
    }

    /// Get memory limit in bytes
    pub fn max_memory_bytes(&self) -> usize {
        (self.max_memory_pages as usize) * 65536
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::new(1_000_000, 16, 5000)
    }
}

/// Fuel cost for various operations
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FuelCosts {
    pub base_opcost: u64,
    pub memory_opcost: u64,
    pub table_opcost: u64,
    pub host_call_cost: u64,
}

impl FuelCosts {
    /// Standard fuel costs (wasmi default)
    pub fn standard() -> Self {
        Self {
            base_opcost: 1,
            memory_opcost: 10,
            table_opcost: 10,
            host_call_cost: 100,
        }
    }

    /// Conservative fuel costs (for safety)
    pub fn conservative() -> Self {
        Self {
            base_opcost: 2,
            memory_opcost: 20,
            table_opcost: 20,
            host_call_cost: 200,
        }
    }
}

impl Default for FuelCosts {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limits() {
        let limits = ResourceLimits::new(1000000, 16, 5000);
        assert_eq!(limits.max_fuel, 1000000);
        assert_eq!(limits.max_memory_pages, 16);
        assert_eq!(limits.max_memory_bytes(), 16 * 65536);
        assert_eq!(limits.timeout_ms, 5000);
    }

    #[test]
    fn test_minimal_limits() {
        let limits = ResourceLimits::minimal();
        assert_eq!(limits.max_fuel, 10_000);
        assert_eq!(limits.max_memory_pages, 1);
        assert_eq!(limits.max_output_bytes, 1024);
    }

    #[test]
    fn test_fuel_costs() {
        let costs = FuelCosts::standard();
        assert_eq!(costs.base_opcost, 1);
        assert_eq!(costs.host_call_cost, 100);
    }
}
