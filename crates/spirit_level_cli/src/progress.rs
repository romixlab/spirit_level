//! The progress display of the long commands: drawn on stderr while they run and cleared when they end, so the
//! report on stdout is the same with or without it. `--progress` decides when.

use clap::ValueEnum;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use sl_engine::{Event, Progress};
use std::io::IsTerminal;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::JoinHandle;
use std::time::Duration;
use vc_ui::Paint;

/// When to show progress
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub(crate) enum ProgressChoice {
    /// At a terminal: not in a pipe, in CI, under `--format json`, with `NO_COLOR` or `TERM=dumb`
    #[default]
    Auto,
    /// Always: drawn at a terminal, one line per step otherwise
    Always,
    /// Never
    Never,
}

/// The global `--progress`, beside `--color`
#[derive(Clone, Copy, Debug, Default, clap::Args)]
pub(crate) struct ProgressFlag {
    /// Progress on stderr while a long command runs: auto (at a terminal, not in CI, not under `--format json`, without `NO_COLOR` or `TERM=dumb`), always or never
    #[arg(long, value_enum, value_name = "WHEN", default_value_t, global = true)]
    progress: ProgressChoice,
}

/// The choice of this run, set once by [`ProgressFlag::apply`] before a command runs
static CHOICE: AtomicU8 = AtomicU8::new(0);

impl ProgressFlag {
    /// Decides progress for the whole run
    pub(crate) fn apply(self) {
        CHOICE.store(self.progress as u8, Ordering::Relaxed);
    }
}

fn choice() -> ProgressChoice {
    match CHOICE.load(Ordering::Relaxed) {
        1 => ProgressChoice::Always,
        2 => ProgressChoice::Never,
        _ => ProgressChoice::Auto,
    }
}

/// How progress is shown
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    /// Not at all
    Off,
    /// One line per step on stderr, for a stderr that is not a terminal under `--progress always`
    Lines,
    /// Drawn and redrawn in place at the terminal, cleared at the end
    Live,
}

/// What the environment says about drawing progress
#[derive(Clone, Debug, Default)]
struct Env {
    /// stderr is a terminal
    terminal: bool,
    /// `TERM`
    term: Option<String>,
    /// `NO_COLOR` set and not empty
    no_color: bool,
    /// `CI` set and not empty
    ci: bool,
}

impl Env {
    fn read() -> Self {
        let set = |k: &str| std::env::var_os(k).is_some_and(|v| !v.is_empty());
        Self {
            terminal: std::io::stderr().is_terminal(),
            term: std::env::var("TERM").ok(),
            no_color: set("NO_COLOR"),
            ci: set("CI"),
        }
    }
}

/// The mode for a choice, the environment and whether the command prints JSON
fn mode(choice: ProgressChoice, env: &Env, json: bool) -> Mode {
    let dumb = env.term.as_deref() == Some("dumb");
    match choice {
        ProgressChoice::Always if env.terminal && !dumb => Mode::Live,
        ProgressChoice::Always => Mode::Lines,
        ProgressChoice::Auto if env.terminal && !dumb && !env.no_color && !env.ci && !json => {
            Mode::Live
        }
        ProgressChoice::Auto | ProgressChoice::Never => Mode::Off,
    }
}

/// The tick that keeps the spinners and the elapsed times moving
const TICK: Duration = Duration::from_millis(100);
/// The width of the bar of analyses done
const BAR_WIDTH: usize = 24;
/// Messages queued for the drawing thread at most; the run waits while the queue is full
const QUEUE_MAX: usize = 64;

/// What the drawing thread is told
enum Draw {
    /// A step before or after the analyses
    Step(String),
    /// The number of analyses that run
    Planned(usize),
    /// An analysis started, with its label
    Started(&'static str, String),
    /// An analysis finished
    Finished(&'static str),
}

/// The progress of one command: built when it starts, cleared when dropped
pub(crate) struct Display {
    /// The command, in front of each line (`spl check`)
    title: String,
    paint: Paint,
    /// The queue to the thread that draws at the terminal, and that thread
    live: Option<(SyncSender<Draw>, JoinHandle<()>)>,
    lines: bool,
    /// Analyses started, for the step lines
    started: AtomicUsize,
    /// Analyses planned
    total: AtomicUsize,
}

impl Display {
    /// The display of the command `title`; `json`: the command prints JSON
    pub(crate) fn new(title: &str, json: bool) -> Self {
        let mode = mode(choice(), &Env::read(), json);
        let paint = Paint::stderr();
        let live = (mode == Mode::Live).then(|| {
            let (tx, rx) = sync_channel(QUEUE_MAX);
            let prefix = paint.id(title);
            (tx, std::thread::spawn(move || draw(prefix, &rx)))
        });
        Self {
            title: title.to_owned(),
            paint,
            live,
            lines: mode == Mode::Lines,
            started: AtomicUsize::new(0),
            total: AtomicUsize::new(0),
        }
    }

    fn send(&self, draw: Draw) {
        if let Some((tx, _)) = &self.live {
            // the drawing thread only ends when this display is dropped: nothing is lost
            tx.send(draw).ok();
        }
    }

    /// A step of a command the engine does not run (`measuring the pages`)
    pub(crate) fn step(&self, what: &str) {
        self.send(Draw::Step(what.to_owned()));
        if self.lines {
            eprintln!("{}: {what}", self.title);
        }
    }

    fn planned(&self, analyses: usize) {
        self.total.store(analyses, Ordering::Relaxed);
        self.send(Draw::Planned(analyses));
    }

    fn started(&self, analysis: &'static str, tools: &[&'static str]) {
        let n = self.started.fetch_add(1, Ordering::Relaxed) + 1;
        let what = self.analysis(analysis, tools);
        if self.lines {
            let total = self.total.load(Ordering::Relaxed);
            eprintln!("{}: [{n}/{total}] {what}", self.title);
        }
        self.send(Draw::Started(analysis, what));
    }

    /// An analysis and the tools it runs: `clippy (cargo-clippy)`
    fn analysis(&self, analysis: &str, tools: &[&str]) -> String {
        let tools: Vec<&str> = tools.iter().copied().filter(|t| *t != analysis).collect();
        if tools.is_empty() {
            analysis.to_owned()
        } else {
            format!(
                "{analysis} {}",
                self.paint.dim(&format!("({})", tools.join(", ")))
            )
        }
    }
}

impl Progress for Display {
    fn event(&self, event: Event<'_>) {
        match event {
            Event::Loading => self.step("loading the project"),
            Event::FindingTools => self.step("finding the tools"),
            Event::Planned { analyses } => self.planned(analyses),
            Event::Started { analysis, tools } => self.started(analysis, tools),
            Event::Finished { analysis } => self.send(Draw::Finished(analysis)),
            Event::Waivers => self.step("reading the waivers"),
        }
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        if let Some((tx, thread)) = self.live.take() {
            drop(tx);
            // the thread clears what it drew before it ends; a panic there spoils stderr only, never the report
            thread.join().ok();
        }
    }
}

/// Draws at the terminal until the queue closes, then clears every line: the run's line, and under it one line per
/// analysis running, so analyses that run at once show at once
fn draw(prefix: String, queue: &Receiver<Draw>) {
    let multi = MultiProgress::with_draw_target(ProgressDrawTarget::stderr());
    let main = multi.add(ProgressBar::new_spinner().with_style(phase_style()));
    main.set_prefix(prefix);
    main.enable_steady_tick(TICK);
    let mut running: Vec<(&'static str, ProgressBar)> = Vec::new();
    for d in queue {
        match d {
            Draw::Step(what) => main.set_message(what),
            Draw::Planned(analyses) => {
                main.set_style(bar_style());
                main.set_length(analyses as u64);
                main.set_message("");
            }
            Draw::Started(analysis, label) => {
                let bar = multi.add(
                    ProgressBar::new_spinner()
                        .with_style(running_style())
                        .with_message(label),
                );
                bar.enable_steady_tick(TICK);
                running.push((analysis, bar));
            }
            Draw::Finished(analysis) => {
                if let Some(i) = running.iter().position(|(a, _)| *a == analysis) {
                    let (_, bar) = running.remove(i);
                    bar.finish_and_clear();
                    multi.remove(&bar);
                }
                main.inc(1);
            }
        }
    }
    for (_, bar) in running {
        bar.finish_and_clear();
    }
    main.finish_and_clear();
    // a failed clear leaves a line on stderr only; the report on stdout is whole
    multi.clear().ok();
}

/// Before the analyses are known: a spinner and the step
fn phase_style() -> ProgressStyle {
    ProgressStyle::with_template("{spinner} {prefix}: {msg} {elapsed}")
        .expect("the template is valid")
}

/// While the analyses run: how many of how many are done
fn bar_style() -> ProgressStyle {
    ProgressStyle::with_template(&format!(
        "{{spinner}} {{prefix}}: [{{bar:{BAR_WIDTH}}}] {{pos}}/{{len}} analyses {{elapsed}} {{msg}}"
    ))
    .expect("the template is valid")
    .progress_chars("=> ")
}

/// One analysis running, indented under the run, with its own time
fn running_style() -> ProgressStyle {
    ProgressStyle::with_template("  {spinner} {msg} {elapsed}").expect("the template is valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(terminal: bool) -> Env {
        Env {
            terminal,
            term: Some("xterm-256color".to_owned()),
            ..Env::default()
        }
    }

    #[test]
    fn auto_draws_only_at_a_plain_terminal() {
        let auto = |e: &Env, json| mode(ProgressChoice::Auto, e, json);
        assert_eq!(auto(&env(true), false), Mode::Live);
        assert_eq!(auto(&env(false), false), Mode::Off);
        assert_eq!(auto(&env(true), true), Mode::Off);
        let mut e = env(true);
        e.no_color = true;
        assert_eq!(auto(&e, false), Mode::Off);
        let mut e = env(true);
        e.ci = true;
        assert_eq!(auto(&e, false), Mode::Off);
        let mut e = env(true);
        e.term = Some("dumb".to_owned());
        assert_eq!(auto(&e, false), Mode::Off);
    }

    #[test]
    fn always_writes_lines_without_a_terminal() {
        assert_eq!(mode(ProgressChoice::Always, &env(true), true), Mode::Live);
        assert_eq!(
            mode(ProgressChoice::Always, &env(false), false),
            Mode::Lines
        );
        let mut e = env(true);
        e.term = Some("dumb".to_owned());
        assert_eq!(mode(ProgressChoice::Always, &e, false), Mode::Lines);
    }

    #[test]
    fn never_is_off() {
        assert_eq!(mode(ProgressChoice::Never, &env(true), false), Mode::Off);
    }
}
