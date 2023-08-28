use clap::Parser;
use std::time::Duration;

const DEFAULT_DURATION_ONE_SEC: u64 = 1500;
const DEFAULT_BUF_CAPACITY: usize = 100;
const DEFAULT_GRAPH_CLEARANCE: f64 = 1.0;

#[derive(Parser)]
#[command(author,version,about, long_about=None)]
pub struct Config {
    #[arg(short, long, default_value_t = DEFAULT_DURATION_ONE_SEC)]
    tick_rate: u64,
    #[arg(short, long, default_value_t = DEFAULT_BUF_CAPACITY)]
    buf_capacity: usize,
    #[arg(short, long)]
    graph: bool,
    #[arg(long = "clearance", default_value_t = DEFAULT_GRAPH_CLEARANCE)]
    graph_clearance: f64,
}

impl Config {
    pub fn tick_rate(&self) -> Duration {
        Duration::from_millis(self.tick_rate)
    }

    pub fn graph_clearance(&self) -> f64 {
        self.graph_clearance
    }

    pub fn buf_capacity(&self) -> usize {
        self.buf_capacity
    }

    pub fn upper_index(&self) -> usize {
        self.buf_capacity - 1
    }

    pub fn graph(&self) -> bool {
        self.graph
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tick_rate: DEFAULT_DURATION_ONE_SEC,
            graph_clearance: DEFAULT_GRAPH_CLEARANCE,
            buf_capacity: DEFAULT_BUF_CAPACITY,
            graph: false,
        }
    }
}
