//! Board lints, one analysis: [`BOARD`]. Every description the `[board]` table selects is read with its inputs
//! ([`sl_board::load`]) and checked on each of its configurations (a revision, with each rework, with all of them); a
//! finding that holds in several configurations is reported once, with the ones it holds in.

mod checks;

use std::ops::Range;

use sl_analysis::{Analysis, Context, lint};
use sl_board::model::{Description, LoadError, Source};
use sl_board::{Input, Loaded};
use sl_core::{Diagnostic, LintMeta, Span};

lint!(READABLE, "F0099", "brd-1", "brd", "a board description reads, follows the schema, and the chip data and PCB it names read",
    params: [], needs: [], fix: None);
lint!(ONE_JOB, "F0100", "brd-2", "brd", "every pin of a board description has one job in each configuration",
    params: [], needs: [], fix: None);
lint!(SHORT_BOTH, "F0101", "brd-3", "brd", "of MCU pins joined on the board, at most one is used",
    params: [], needs: [], fix: None);
lint!(CHIP_KNOWS, "F0102", "brd-4", "brd", "the pins, package, peripherals and signals a board description names exist on its chip",
    params: [], needs: [], fix: None);
lint!(SIGNAL_PIN, "F0103", "brd-5", "brd", "every peripheral signal sits on a pin that can carry it",
    params: [], needs: [], fix: None);
lint!(PCB_NETS, "F0104", "brd-6", "brd", "the MCU pins are connected on the PCB as the board description says",
    params: [], needs: [], fix: None);
lint!(PCB_COVERED, "F0105", "brd-7", "brd", "every connected MCU pin of the PCB is in the board description, and no free pin is unconnected",
    params: [], needs: [], fix: None);

static LINTS: &[&LintMeta] = &[
    &READABLE,
    &ONE_JOB,
    &SHORT_BOTH,
    &CHIP_KNOWS,
    &SIGNAL_PIN,
    &PCB_NETS,
    &PCB_COVERED,
];

/// The board analysis
pub struct Board;

/// The registered instance
pub static BOARD: Board = Board;

/// One finding of a check, before it becomes a [`Diagnostic`]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Finding {
    /// The lint's id
    pub(crate) lint: &'static str,
    /// The check's name within the lint (`pin-twice`, `pcb-net`), first part of the item
    pub(crate) check: &'static str,
    /// What the item is about (a pin, a net, an entry), second part of the item
    pub(crate) subject: String,
    pub(crate) message: String,
    /// Byte range in the description; None: the configuration's PCB as a whole
    pub(crate) at: Option<Range<usize>>,
    pub(crate) help: Option<String>,
}

impl Finding {
    pub(crate) fn new(
        lint: &'static LintMeta,
        check: &'static str,
        subject: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            lint: lint.id,
            check,
            subject: subject.into(),
            message: message.into(),
            at: None,
            help: None,
        }
    }

    pub(crate) fn at(mut self, at: &Range<usize>) -> Self {
        self.at = Some(at.clone());
        self
    }

    pub(crate) fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

impl Analysis for Board {
    fn name(&self) -> &'static str {
        "board"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let project = cx.project;
        let members: Vec<&str> = project.packages.iter().map(|p| p.dir.as_str()).collect();
        let cfg = &cx.config.board;
        for file in sl_board::find(&project.root, &members, &cfg.descriptions) {
            match sl_board::load(&project.root, &file, &cfg.chip_data) {
                Ok(loaded) => check(cx, &file, &loaded),
                Err(e) => unreadable(cx, &file, &e),
            }
        }
    }
}

/// A description that cannot be read, or breaks the schema
fn unreadable(cx: &Context<'_>, file: &str, e: &LoadError) {
    match e {
        LoadError::Read { error, .. } => cx.emit(
            Diagnostic::new(
                READABLE.id,
                format!("{file}:read"),
                format!("{file}: {error}"),
            )
            .span(Span::new(file, 0))
            .help("make the file readable UTF-8 text"),
        ),
        LoadError::Invalid {
            file: src,
            diagnostics,
        } => {
            for d in diagnostics {
                let mut out = Diagnostic::new(
                    READABLE.id,
                    format!("{file}:{}:{}", d.code, d.message),
                    format!("{} [{}]", d.message, d.code),
                )
                .span(match &d.span {
                    Some(at) => span(file, src, at),
                    None => Span::new(file, 0),
                });
                out = out.help(d.help.clone().unwrap_or_else(|| {
                    "write it as the schema says (`spl explain brd-1`)".to_owned()
                }));
                cx.emit(out);
            }
        }
    }
}

/// The span of byte range `at` in the description
fn span(file: &str, src: &Source, at: &Range<usize>) -> Span {
    let (l0, c0) = src.line_col(at.start);
    let (l1, c1) = src.line_col(at.end);
    Span::range(file, (u32_of(l0), u32_of(c0)), (u32_of(l1), u32_of(c1)))
}

fn u32_of(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

fn check(cx: &Context<'_>, file: &str, loaded: &Loaded) {
    let desc = &loaded.description;
    let src = &desc.source;
    // inputs that are there but wrong are findings of brd-1; missing ones stop the lints that need them
    let mut found: Vec<Finding> = Vec::new();
    input(
        &loaded.chip,
        &mut found,
        &[&CHIP_KNOWS, &SIGNAL_PIN],
        cx,
        file,
    );
    for pcb in loaded.pcbs.values() {
        input(pcb, &mut found, &[&PCB_NETS, &PCB_COVERED], cx, file);
    }
    let chip = loaded.chip.ready();
    if let Some(chip) = chip {
        checks::chip::package(&desc.board, chip, &mut found);
    }
    let configs = desc.configurations();
    let mut merged: Vec<(Finding, Vec<String>)> =
        found.into_iter().map(|f| (f, Vec::new())).collect();
    for board in configs {
        let mut found = Vec::new();
        checks::pins::check(board, &mut found);
        if let Some(chip) = chip {
            checks::chip::check(board, &desc.board, chip, &mut found);
        }
        let pcb = board.pcb.as_ref();
        if let Some(mcu) = pcb
            .and_then(|p| loaded.pcbs.get(&p.value))
            .and_then(Input::ready)
        {
            let pcb_at = pcb.map(|p| p.span.clone());
            for f in checks::pcb::check(board, mcu) {
                // a finding about the PCB as a whole points at the `pcb` that names it
                let f = match (&f.at, &pcb_at) {
                    (None, Some(at)) => f.at(at),
                    _ => f,
                };
                found.push(f);
            }
        }
        for f in found {
            match merged.iter_mut().find(|(m, _)| *m == f) {
                Some((_, labels)) => labels.push(board.label.clone()),
                None => merged.push((f, vec![board.label.clone()])),
            }
        }
    }
    for (f, labels) in merged {
        cx.emit(diagnostic(file, src, desc, f, &labels));
    }
}

/// Reports a missing input as not run for `lints`, an invalid one as a finding
fn input<T>(
    i: &Input<T>,
    found: &mut Vec<Finding>,
    lints: &[&'static LintMeta],
    cx: &Context<'_>,
    file: &str,
) {
    match i {
        Input::Ready(_) => {}
        Input::Missing(why) => {
            for l in lints {
                cx.not_run(l.id, "board-input", format!("{file}: skipped: {why}"));
            }
        }
        Input::Invalid { message, at, help } => {
            let mut f = Finding::new(&READABLE, "input", message.clone(), message.clone()).at(at);
            f.help = help.clone();
            found.push(f);
        }
    }
}

fn diagnostic(
    file: &str,
    src: &Source,
    desc: &Description,
    f: Finding,
    labels: &[String],
) -> Diagnostic {
    let all = desc.configurations().len();
    let message = if labels.is_empty() || labels.len() == all {
        f.message
    } else {
        format!("{} (in {})", f.message, labels.join(", "))
    };
    let mut d = Diagnostic::new(f.lint, format!("{file}:{}:{}", f.check, f.subject), message).span(
        match &f.at {
            Some(at) => span(file, src, at),
            None => Span::new(file, 0),
        },
    );
    d = d.help(
        f.help
            .unwrap_or_else(|| format!("`spl explain {}`", f.lint)),
    );
    d
}
