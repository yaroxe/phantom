//! The Fx hasher for hot maps keyed by small integers: kept in phantom-core
//! (the scheduled ticks use it too), and reachable here under its old path.
pub use phantom_core::fast_hash::*;
