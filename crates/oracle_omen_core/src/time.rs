//! Deterministic time injection.
//!
//! All time must be injected, never from system clock.
//! Uses logical time (monotonically increasing counters) for determinism.

use core::fmt;

/// Logical timestamp - deterministic and injectable
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct LogicalTime {
    /// Run ID - identifies a specific execution run
    pub run_id: u64,

    /// Sequence number - monotonically increasing within a run
    pub sequence: u64,
}

impl LogicalTime {
    /// Create a new logical time
    #[must_use]
    pub const fn new(run_id: u64, sequence: u64) -> Self {
        Self { run_id, sequence }
    }

    /// Create initial time (sequence 0)
    #[must_use]
    pub const fn initial(run_id: u64) -> Self {
        Self { run_id, sequence: 0 }
    }

    /// Get the next logical time
    #[must_use]
    pub const fn next(&self) -> Self {
        Self {
            run_id: self.run_id,
            sequence: self.sequence + 1,
        }
    }

    /// Get the run ID
    #[must_use]
    pub const fn run_id(&self) -> u64 {
        self.run_id
    }

    /// Get the sequence number
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
}

impl fmt::Display for LogicalTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T({}:{})", self.run_id, self.sequence)
    }
}

/// Time source trait - allows injection for testing
pub trait TimeSource: Send + Sync {
    /// Get the current logical time
    fn now(&self) -> LogicalTime;

    /// Increment and get the next time
    fn tick(&self) -> LogicalTime;
}

/// Standard time source using atomic counters
#[derive(Debug)]
pub struct StandardTimeSource {
    run_id: u64,
    sequence: core::sync::atomic::AtomicU64,
}

impl StandardTimeSource {
    /// Create a new time source
    #[must_use]
    pub const fn new(run_id: u64) -> Self {
        Self {
            run_id,
            sequence: core::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Create with run_id = 0
    #[must_use]
    pub const fn zero() -> Self {
        Self::new(0)
    }
}

impl TimeSource for StandardTimeSource {
    fn now(&self) -> LogicalTime {
        let seq = self.sequence.load(core::sync::atomic::Ordering::SeqCst);
        LogicalTime::new(self.run_id, seq)
    }

    fn tick(&self) -> LogicalTime {
        let seq = self.sequence.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        LogicalTime::new(self.run_id, seq)
    }
}

/// Mock time source for testing - allows explicit time control
#[derive(Debug)]
pub struct MockTimeSource {
    run_id: u64,
    time: core::sync::atomic::AtomicU64,
}

impl MockTimeSource {
    /// Create a new mock time source
    #[must_use]
    pub const fn new(run_id: u64, initial: u64) -> Self {
        Self {
            run_id,
            time: core::sync::atomic::AtomicU64::new(initial),
        }
    }

    /// Set the current time explicitly
    pub fn set(&self, sequence: u64) {
        self.time.store(sequence, core::sync::atomic::Ordering::SeqCst);
    }

    /// Advance by a specific amount
    pub fn advance(&self, delta: u64) -> LogicalTime {
        let seq = self.time.fetch_add(delta, core::sync::atomic::Ordering::SeqCst);
        LogicalTime::new(self.run_id, seq)
    }
}

impl TimeSource for MockTimeSource {
    fn now(&self) -> LogicalTime {
        let seq = self.time.load(core::sync::atomic::Ordering::SeqCst);
        LogicalTime::new(self.run_id, seq)
    }

    fn tick(&self) -> LogicalTime {
        let seq = self.time.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        LogicalTime::new(self.run_id, seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logical_time_ordering() {
        let t1 = LogicalTime::new(1, 10);
        let t2 = LogicalTime::new(1, 11);
        let t3 = LogicalTime::new(1, 10);

        assert!(t2 > t1);
        assert_eq!(t1, t3);
    }

    #[test]
    fn test_logical_time_next() {
        let t = LogicalTime::initial(5);
        assert_eq!(t.sequence(), 0);
        assert_eq!(t.next().sequence(), 1);
        assert_eq!(t.next().next().sequence(), 2);
    }

    #[test]
    fn test_standard_time_source() {
        let source = StandardTimeSource::new(42);
        assert_eq!(source.now().sequence(), 0);
        assert_eq!(source.tick().sequence(), 0);
        assert_eq!(source.now().sequence(), 1);
        assert_eq!(source.tick().sequence(), 1);
    }
}
