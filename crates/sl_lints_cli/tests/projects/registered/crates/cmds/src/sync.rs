//! `big sync`: registered here, through a bare `submit!`

use inventory::submit;

submit! { Command::new("sync") }
