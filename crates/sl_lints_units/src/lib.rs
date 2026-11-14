//! Lints that move plain numbers into `exact_units` types, one analysis: [`UNITS`]. A name says what a number is
//! (`timeout_ms`, `vbat_mv`, `slot_idx`); the lints suggest the type that says it to the compiler, with the
//! bounds the crate's own checks and docs give.

mod bodies;
mod evidence;
mod items;
mod names;

use sl_analysis::syntax::without_tests;
use sl_analysis::{Analysis, Context, lint};
use sl_core::{Confidence, Diagnostic, LintMeta, Span};
use sl_model::{FileKind, Package};
use syn::visit::Visit;

lint!(EXACT_UNITS_QUANTITY, "C0033", "exact-units-quantity", "units", "a number that holds a physical or dimensionless quantity carries its unit in its type",
    params: [
        ("units", StrList, "[\"ns=Fixed<Second, Nano, T>\", \"nanos=Fixed<Second, Nano, T>\", \"us=Fixed<Second, Micro, T>\", \"micros=Fixed<Second, Micro, T>\", \"ms=Fixed<Second, Milli, T>\", \"millis=Fixed<Second, Milli, T>\", \"s=Fixed<Second, One, T>\", \"sec=Fixed<Second, One, T>\", \"secs=Fixed<Second, One, T>\", \"seconds=Fixed<Second, One, T>\", \"min=Fixed<Minute, One, T>\", \"mins=Fixed<Minute, One, T>\", \"minutes=Fixed<Minute, One, T>\", \"h=Fixed<Hour, One, T>\", \"hours=Fixed<Hour, One, T>\", \"days=Fixed<Day, One, T>\", \"hz=Fixed<Hertz, One, T>\", \"khz=Fixed<Hertz, Kilo, T>\", \"mhz=Fixed<Hertz, Mega, T>\", \"ghz=Fixed<Hertz, Giga, T>\", \"rpm=Fixed<Per<Turn, Minute>, One, T>\", \"v=Fixed<Volt, One, T>\", \"mv=Fixed<Volt, Milli, T>\", \"uv=Fixed<Volt, Micro, T>\", \"ma=Fixed<Ampere, Milli, T>\", \"ua=Fixed<Ampere, Micro, T>\", \"ohm=Fixed<Ohm, One, T>\", \"ohms=Fixed<Ohm, One, T>\", \"kohm=Fixed<Ohm, Kilo, T>\", \"uf=Fixed<Farad, Micro, T>\", \"nf=Fixed<Farad, Nano, T>\", \"pf=Fixed<Farad, Pico, T>\", \"uh=Fixed<Henry, Micro, T>\", \"mw=Fixed<Watt, Milli, T>\", \"mj=Fixed<Joule, Milli, T>\", \"c=Point<Celsius, Fixed<Kelvin, One, T>>\", \"degc=Point<Celsius, Fixed<Kelvin, One, T>>\", \"celsius=Point<Celsius, Fixed<Kelvin, One, T>>\", \"mdegc=Point<Celsius, Fixed<Kelvin, Milli, T>>\", \"kelvin=Fixed<Kelvin, One, T>\", \"mm=Fixed<Meter, Milli, T>\", \"um=Fixed<Meter, Micro, T>\", \"cm=Fixed<Meter, Centi, T>\", \"km=Fixed<Meter, Kilo, T>\", \"mg=Fixed<Gram, Milli, T>\", \"kg=Fixed<Gram, Kilo, T>\", \"pa=Fixed<Pascal, One, T>\", \"hpa=Fixed<Pascal, Hecto, T>\", \"kpa=Fixed<Pascal, Kilo, T>\", \"deg=Fixed<Degree, One, T>\", \"degrees=Fixed<Degree, One, T>\", \"mdeg=Fixed<Degree, Milli, T>\", \"rad=Fixed<Radian, One, T>\", \"mrad=Fixed<Radian, Milli, T>\", \"bytes=Fixed<Byte, One, T>\", \"kib=Fixed<Byte, Kibi, T>\", \"mib=Fixed<Byte, Mebi, T>\", \"gib=Fixed<Byte, Gibi, T>\", \"kb=Fixed<Byte, Kilo, T>\", \"mb=Fixed<Byte, Mega, T>\", \"gb=Fixed<Byte, Giga, T>\", \"bits=Fixed<Bit, One, T>\", \"bps=Fixed<Per<Bit, Second>, One, T>\", \"baud=Fixed<Per<Bit, Second>, One, T>\", \"percent=Fixed<Ratio, Centi, T>\", \"pct=Fixed<Ratio, Centi, T>\", \"permille=Fixed<Ratio, Milli, T>\", \"ppm=Fixed<Ratio, Micro, T>\", \"ppb=Fixed<Ratio, Nano, T>\", \"q15=Q15\", \"q31=Q31\", \"db=Fixed<Decibel, P, T>\"]", "unit words, matched as the last word of a name (before a qualifier), with the type that holds them; T stands for the number's own type"),
        ("words", StrList, "[\"voltage=Fixed<Volt, P, T>\", \"vbat=Fixed<Volt, P, T>\", \"vref=Fixed<Volt, P, T>\", \"frequency=Fixed<Hertz, P, T>\", \"freq=Fixed<Hertz, P, T>\", \"temperature=Point<Celsius, Fixed<Kelvin, P, T>>\", \"temp=Point<Celsius, Fixed<Kelvin, P, T>>\", \"pressure=Fixed<Pascal, P, T>\", \"power=Fixed<Watt, P, T>\", \"energy=Fixed<Joule, P, T>\", \"resistance=Fixed<Ohm, P, T>\", \"capacitance=Fixed<Farad, P, T>\", \"inductance=Fixed<Henry, P, T>\", \"angle=Fixed<Turn, P, T>\", \"speed=Fixed<Speed, P, T>\", \"velocity=Fixed<Speed, P, T>\", \"timeout=Fixed<Second, P, T>\", \"delay=Fixed<Second, P, T>\", \"interval=Fixed<Second, P, T>\", \"period=Fixed<Second, P, T>\", \"latency=Fixed<Second, P, T>\", \"elapsed=Fixed<Second, P, T>\", \"ttl=Fixed<Second, P, T>\", \"backoff=Fixed<Second, P, T>\", \"deadline=Fixed<Second, P, T>\", \"size=Fixed<Byte, P, T>\", \"duty=Fixed<Ratio, P, T>\", \"gain=Fixed<Ratio, P, T>\", \"ratio=Fixed<Ratio, P, T>\", \"percent=Fixed<Ratio, Centi, T>\", \"percentage=Fixed<Ratio, Centi, T>\"]", "quantity words, matched anywhere in a name without a unit word, with the type that holds them; P is the scale to choose"),
        ("untyped-units", StrList, "[\"ticks\", \"cycles\", \"words\", \"pages\", \"sectors\", \"blocks\", \"count\", \"items\", \"entries\", \"elements\", \"chars\", \"lines\", \"samples\", \"frames\", \"packets\", \"px\", \"pt\", \"lsb\", \"b\"]", "unit words with no type in the units table: a name that ends in one says its unit and passes the quantity words"),
        ("qualifiers", StrList, "[\"max\", \"min\", \"limit\", \"nominal\", \"typ\", \"default\", \"target\", \"threshold\", \"step\", \"lo\", \"hi\"]", "words that may follow the unit at the end of a name (`latency_ms_max`)"),
    ],
    needs: [], fix: None);
lint!(EXACT_UNITS_PLAIN_NUMBER, "C0034", "exact-units-plain-number", "units", "an index, a count, a sequence number or a duration carries what it counts in its type",
    params: [
        ("words", StrList, "[\"idx=Index<Of, T>\", \"index=Index<Of, T>\", \"pos=Index<Of, T>\", \"count=Fixed<Of, One, T>\", \"num=Fixed<Of, One, T>\", \"n=Fixed<Of, One, T>\", \"seq=Seq<Of, T>\", \"seqno=Seq<Of, T>\", \"samples=Fixed<Sample, One, T>\", \"frames=Fixed<Frame, One, T>\", \"packets=Fixed<Packet, One, T>\", \"items=Fixed<Item, One, T>\", \"cycles=Fixed<Cycle, One, T>\", \"ticks=Fixed<Second, Rational<1, TICK_HZ>, T>\"]", "name words of positions and counts, matched anywhere in a name, with the type that holds them; Of is the thing counted, a unit of the crate made with counted!"),
        ("durations", StrList, "[\"Duration\"]", "types of a time span that a Fixed of seconds holds in the bytes of its integer"),
    ],
    needs: [], fix: None);
lint!(EXACT_UNITS_BOUNDS, "C0035", "exact-units-bounds", "units", "a number whose range the code knows says it in its type",
    params: [
        ("ranges", StrList, "[\"percent=0..=100\", \"pct=0..=100\", \"percentage=0..=100\", \"permille=0..=1000\"]", "name words whose stored value has a known range"),
        ("unbounded", StrList, "[\"Millis\", \"Micros\", \"Nanos\", \"Secs\", \"Minutes\", \"Hours\", \"Days\", \"Hz\", \"Bytes\", \"Kibibytes\", \"Mebibytes\", \"Gibibytes\", \"Percent\", \"Permille\", \"Ppm\", \"Ppb\", \"Count\"]", "aliases of exact_units that take no bounds; Fixed, Index and IndexFrom1 without their last parameter count as well"),
    ],
    needs: [], fix: None);
lint!(EXACT_UNITS_IN_BODIES, "C0036", "exact-units-in-bodies", "units", "inside functions too, a quantity stays typed: no unit-named locals or literal arguments, casts, float maths, bare comparisons, raw arithmetic, scale constants or conversions by hand",
    params: [
        ("factors", StrList, "[\"10\", \"100\", \"1000\", \"1000000\", \"1000000000\", \"60\", \"3600\", \"86400\", \"1024\", \"1048576\"]", "literal factors that convert between scales when a unit-named value is multiplied or divided by them; underscores are ignored"),
        ("escapes", StrList, "[\"get\", \"raw\"]", "methods without arguments that take the plain number out of a typed value; checked only in crates that depend on exact_units"),
        ("ignore-literals", StrList, "[\"0\"]", "literals a unit-named value may be compared with"),
    ],
    needs: [], fix: None);

/// The `exact_units` analysis
pub struct Units;

/// The registered instance
pub static UNITS: Units = Units;

static LINTS: &[&LintMeta] = &[
    &EXACT_UNITS_QUANTITY,
    &EXACT_UNITS_PLAIN_NUMBER,
    &EXACT_UNITS_BOUNDS,
    &EXACT_UNITS_IN_BODIES,
];

pub(crate) const QUANTITY: &str = "exact-units-quantity";
pub(crate) const PLAIN: &str = "exact-units-plain-number";
pub(crate) const BOUNDS: &str = "exact-units-bounds";
pub(crate) const BODIES: &str = "exact-units-in-bodies";

/// The crate the suggestions name
const CRATE: &str = "exact_units";

/// One finding before it becomes a diagnostic
pub(crate) struct Found {
    pub lint: &'static str,
    pub range: sl_analysis::syntax::Range,
    pub item: String,
    pub message: String,
    pub help: String,
    pub confidence: Confidence,
}

impl Analysis for Units {
    fn name(&self) -> &'static str {
        "units"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        if !LINTS.iter().any(|l| cx.enabled(l.id)) {
            return;
        }
        for pkg in &cx.project.packages {
            run_package(cx, pkg);
        }
    }
}

fn run_package(cx: &Context<'_>, pkg: &Package) {
    let p = cx.project;
    let quantity = cx.params_for(QUANTITY, pkg);
    let plain = cx.params_for(PLAIN, pkg);
    let bounds = cx.params_for(BOUNDS, pkg);
    let bodies = cx.params_for(BODIES, pkg);
    let tables = names::Tables::new(&quantity, &plain, &bounds);
    let files: Vec<_> = p
        .sources(pkg)
        .iter()
        .filter(|f| f.kind == FileKind::Src && !p.is_test_file(pkg, f))
        .filter_map(|f| f.ast().ok().map(|ast| (f, without_tests(ast))))
        .collect();
    let ev = evidence::Evidence::new(files.iter().map(|(f, ast)| (f.rel.as_str(), ast)));
    let has_dep = pkg.deps.iter().any(|d| d.name == CRATE);
    let fns = bodies::signatures(files.iter().map(|(_, ast)| ast), &tables);
    for (f, ast) in &files {
        let mut v = items::Items::new(cx, &tables, &ev, has_dep);
        v.visit_file(ast);
        let mut found = v.found;
        if cx.enabled(BODIES) {
            let mut b = bodies::Bodies::new(&tables, &fns, &bodies, has_dep);
            b.visit_file(ast);
            found.extend(b.found);
        }
        for x in found {
            if !cx.enabled(x.lint) {
                continue;
            }
            cx.emit(
                Diagnostic::new(
                    x.lint,
                    format!("{}:{}:{}", pkg.name, f.rel, x.item),
                    x.message,
                )
                .span(Span::range(f.rel.clone(), x.range.0, x.range.1))
                .krate(&pkg.name)
                .confidence(x.confidence)
                .help(x.help),
            );
        }
    }
}
