pub mod prof_log;
pub mod server_process;
pub mod synthetic_load;

use std::time::Duration;

pub const VISIBLE_PANE_COUNT: usize = 12;
pub const CLIENT_TERMINAL_COLUMNS: u16 = 200;
pub const CLIENT_TERMINAL_ROWS: u16 = 50;
pub const AGENT_STATE_REPORT_INTERVAL: Duration = Duration::from_millis(250);
pub const AGENT_STATE_TRANSITION_EVERY: usize = 12;
pub const MEASUREMENT_WINDOW: Duration = Duration::from_secs(45);
pub const PANE_SETTLE_DELAY: Duration = Duration::from_secs(3);
