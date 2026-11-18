//! What a run is doing, reported while it happens: the caller draws it (the `spl` binary on a terminal), the engine
//! only reports.

/// One step of a run
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event<'a> {
    /// The project is read: cargo metadata, manifests
    Loading,
    /// The external tools the enabled lints run are found and their versions read
    FindingTools,
    /// The analyses to run are known: each has an enabled lint whose needs are met
    Planned {
        /// How many will run
        analyses: usize,
    },
    /// An analysis started; several may run at once
    Started {
        /// Its name
        analysis: &'static str,
        /// The external tools its lints run (`cargo-clippy`), empty for a native analysis
        tools: &'a [&'static str],
    },
    /// An analysis finished
    Finished {
        /// Its name
        analysis: &'static str,
    },
    /// The waivers of the sources, manifests and commits are read and matched to the findings
    Waivers,
}

/// Receives the events of a run. Analyses may run on several threads, so it is `Sync`.
pub trait Progress: Sync {
    /// One event, in the order it happened on its thread
    fn event(&self, event: Event<'_>);
}

/// Reports nothing
#[derive(Clone, Copy, Debug, Default)]
pub struct Silent;

impl Progress for Silent {
    fn event(&self, _: Event<'_>) {}
}

impl<F: Fn(Event<'_>) + Sync> Progress for F {
    fn event(&self, event: Event<'_>) {
        self(event);
    }
}
