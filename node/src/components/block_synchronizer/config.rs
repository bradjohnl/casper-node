use std::str::FromStr;

use datasize::DataSize;
use serde::{Deserialize, Serialize};

use casper_types::TimeDiff;

const DEFAULT_TIMEOUT: &str = "10mins";
const DEFAULT_MAX_PARALLEL_TRIE_FETCHES: u32 = 5000;
const DEFAULT_PEER_REFRESH_INTERVAL: &str = "90sec";

/// Configuration options for fetching.
#[derive(Copy, Clone, DataSize, Debug, Deserialize, Serialize)]
pub struct Config {
    timeout: TimeDiff,
    /// Maximum number of trie nodes to fetch in parallel.
    max_parallel_trie_fetches: u32,
    peer_refresh_interval: TimeDiff,
}

impl Config {
    pub(crate) fn timeout(&self) -> TimeDiff {
        self.timeout
    }

    pub(crate) fn max_parallel_trie_fetches(&self) -> u32 {
        self.max_parallel_trie_fetches
    }

    pub(crate) fn peer_refresh_interval(&self) -> TimeDiff {
        self.peer_refresh_interval
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            timeout: TimeDiff::from_str(DEFAULT_TIMEOUT).unwrap(),
            max_parallel_trie_fetches: DEFAULT_MAX_PARALLEL_TRIE_FETCHES,
            peer_refresh_interval: TimeDiff::from_str(DEFAULT_PEER_REFRESH_INTERVAL).unwrap(),
        }
    }
}
