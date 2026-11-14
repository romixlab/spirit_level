//! Positions, counts, sequence numbers and spans in plain numbers.

use std::time::Duration;

pub struct Ring {
    pub head_idx: u8,
    pub sample_count: u32,
    pub tx_seq: u16,
    pub n_channels: u8,
    pub poll: Duration,
}

impl Ring {
    pub fn new(head_idx: u8) -> Self {
        assert!(head_idx < 8, "the ring has 8 slots");
        Self { head_idx, sample_count: 0, tx_seq: 0, n_channels: 1, poll: Duration::ZERO }
    }
}
