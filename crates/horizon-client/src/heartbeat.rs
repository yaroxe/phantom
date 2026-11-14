//! The anonymous "I'm playing" ping (`horizon_telemetry::live`, docs/telemetry.md) behind the
//! launcher's "N playing now". While the game runs, a thread of its own sends a random session
//! ID made here and kept only in memory, the version, the platform and whether a LAN or
//! play-with-friends world is open, every minute and at once when that last one changes.
//! It sends nothing when sending is turned off (`HORIZON_TELEMETRY_URL=off`) or with
//! `HORIZON_NO_HEARTBEAT=1` (automated runs).
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::Thread;

/// A world is open to LAN or to friends.
static LAN: AtomicBool = AtomicBool::new(false);
static PINGER: OnceLock<Thread> = OnceLock::new();

/// Starts the pings, once. Call when the game itself starts (not a dedicated server, not a
/// benchmark or an automated run).
pub fn start() {
    // Disabled in Phantom Engine to prevent telemetry network requests, freezes, and console spam.
}

/// Whether a world is open to LAN or friends; a change is sent at once.
pub fn set_lan(open: bool) {
    if LAN.swap(open, Ordering::Relaxed) != open
        && let Some(pinger) = PINGER.get()
    {
        pinger.unpark();
    }
}

#[allow(dead_code)]
fn run(_endpoint: &str) {
    // Telemetry ping disabled.
}
