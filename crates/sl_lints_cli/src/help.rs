//! clap's help layout: the text before `Usage:`, then sections (`Commands:`, `Options:`, a custom heading) of
//! indented entries, each a name and its help.

/// One entry of a section: `  -o, --out <FILE>  Where to write` gives the name `-o, --out <FILE>`
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Entry {
    /// The left column
    pub name: String,
    /// The help, its continuation lines joined; empty when the entry has none
    pub help: String,
}

impl Entry {
    /// The help without clap's trailing notes (`[default: x]`, `[possible values: ...]`, `[env: X=]`, `[aliases: y]`)
    pub(crate) fn own_help(&self) -> &str {
        let mut h = self.help.trim();
        while h.ends_with(']')
            && let Some(i) = h.rfind('[')
        {
            h = h[..i].trim_end();
        }
        h
    }

    /// Whether it names an argument or an option, as clap prints them: `-x`, `--long`, or a value name in capitals
    /// (`<VALUE>`, `[PATH]...`); a lowercase `[table]` in an after-help text is not one
    pub(crate) fn is_arg(&self) -> bool {
        self.name.starts_with('-')
            || (self.name.starts_with(['<', '[']) && !self.name.chars().any(|c| c.is_lowercase()))
    }

    /// The short and the long flag of an option entry (`-o, --out <FILE>` gives `-o` and `--out`)
    pub(crate) fn flags(&self) -> (Option<&str>, Option<&str>) {
        let mut short = None;
        let mut long = None;
        for w in self.name.split([',', ' ']).filter(|w| !w.is_empty()) {
            if w.starts_with("--") {
                long = long.or(Some(w));
            } else if w.starts_with('-') && w.len() == 2 {
                short = short.or(Some(w));
            }
        }
        (short, long)
    }
}

/// A section: a heading at the start of a line, ending in `:`, and its entries
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Section {
    /// `Commands:`, `Options:`, ...
    pub heading: String,
    /// Its entries
    pub entries: Vec<Entry>,
}

/// A help text read in clap's layout
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Help {
    /// The text before the `Usage:` line (the about)
    pub about: String,
    /// The sections after it
    pub sections: Vec<Section>,
}

/// The widest indent of an entry's first line; deeper lines continue the entry before (clap indents options with
/// only a long flag by 6)
const ENTRY_INDENT: usize = 6;

impl Help {
    /// Reads a help text; None when it has no `Usage:` line at the start of a line (not clap's layout)
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let lines: Vec<&str> = text.lines().collect();
        let usage = lines.iter().position(|l| l.starts_with("Usage:"))?;
        let about = lines[..usage]
            .iter()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let mut sections: Vec<Section> = Vec::new();
        let mut open = false;
        for line in &lines[usage + 1..] {
            if line.trim().is_empty() {
                continue;
            }
            let indent = line.len() - line.trim_start().len();
            if indent == 0 {
                open = line.trim_end().ends_with(':');
                if open {
                    sections.push(Section {
                        heading: line.trim_end().to_owned(),
                        entries: Vec::new(),
                    });
                }
                continue;
            }
            let Some(sec) = sections.last_mut().filter(|_| open) else {
                continue;
            };
            let body = line.trim();
            match sec.entries.last_mut() {
                Some(e) if indent > ENTRY_INDENT => {
                    if !e.help.is_empty() {
                        e.help.push(' ');
                    }
                    e.help.push_str(body);
                }
                _ => {
                    let (name, help) = body.split_once("  ").unwrap_or((body, ""));
                    sec.entries.push(Entry {
                        name: name.trim().to_owned(),
                        help: help.trim().to_owned(),
                    });
                }
            }
        }
        Some(Self { about, sections })
    }

    /// The section with this heading
    pub(crate) fn section(&self, heading: &str) -> Option<&Section> {
        self.sections.iter().find(|s| s.heading == heading)
    }

    /// Every argument and option entry, of every section
    pub(crate) fn args(&self) -> impl Iterator<Item = &Entry> {
        self.sections
            .iter()
            .flat_map(|s| &s.entries)
            .filter(|e| e.is_arg())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHORT: &str = "\
Check things

Usage: tool [OPTIONS] <COMMAND>

Commands:
  check  Check the project
  bare
  help   Print this message or the help of the given subcommand(s)

Arguments:
  [PATH]  The project directory [default: .]

Options:
      --color <WHEN>  Colour [default: auto] [possible values: auto, always, never]
  -n, --name <N>      [default: x]
      --a-very-long-option-name <VALUE>
          Wrapped onto the next line
  -h, --help          Print help

Examples:
  tool check
";

    #[test]
    fn reads_sections_and_entries() {
        let h = Help::parse(SHORT).unwrap();
        assert_eq!(h.about, "Check things");
        let cmds = h.section("Commands:").unwrap();
        let names: Vec<&str> = cmds.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["check", "bare", "help"]);
        assert_eq!(cmds.entries[1].help, "");
        let args: Vec<(&str, &str)> = h.args().map(|e| (e.name.as_str(), e.own_help())).collect();
        assert_eq!(
            args,
            [
                ("[PATH]", "The project directory"),
                ("--color <WHEN>", "Colour"),
                ("-n, --name <N>", ""),
                (
                    "--a-very-long-option-name <VALUE>",
                    "Wrapped onto the next line"
                ),
                ("-h, --help", "Print help"),
            ]
        );
        let ex = h.section("Examples:").unwrap();
        assert!(!ex.entries[0].is_arg());
        let toml = Entry {
            name: "[dependents]".into(),
            help: String::new(),
        };
        assert!(!toml.is_arg());
    }

    #[test]
    fn flags_of_an_option() {
        let e = Entry {
            name: "-n, --name <N>".into(),
            help: String::new(),
        };
        assert_eq!(e.flags(), (Some("-n"), Some("--name")));
        let e = Entry {
            name: "--color <WHEN>".into(),
            help: String::new(),
        };
        assert_eq!(e.flags(), (None, Some("--color")));
    }

    #[test]
    fn not_clap() {
        assert_eq!(Help::parse("tool: does things\nrun it with a file\n"), None);
    }
}
