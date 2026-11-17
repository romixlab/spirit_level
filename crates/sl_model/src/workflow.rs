//! GitHub Actions workflows: the steps of their jobs, each a `run:` script or a `uses:` action, and the commands a
//! shell script runs.
//!
//! Only the files are read; what the repository's settings on GitHub do with them is not known here.

use crate::{ConfigNode, YamlFile};
use std::path::Path;

/// The folder GitHub reads workflows from, relative to the project root
pub const DIR: &str = ".github/workflows";

/// One workflow file
#[derive(Debug)]
pub struct Workflow {
    /// Path relative to the project root
    pub rel: String,
    /// The steps of every job, in file order
    pub steps: Vec<Step>,
}

/// One step of a job
#[derive(Debug)]
pub struct Step {
    /// The job's id
    pub job: String,
    /// 1-based line the step starts on
    pub line: u32,
    /// The shell script of a `run:` step
    pub run: Option<String>,
    /// The action of a `uses:` step (`actions/deploy-pages@v4`)
    pub uses: Option<String>,
}

impl Workflow {
    /// Parses a workflow's text; the error carries the line
    pub fn parse(rel: impl Into<String>, text: String) -> Result<Self, String> {
        let file = YamlFile::parse(rel, text)?;
        let mut steps = Vec::new();
        for (job, _, body) in file
            .get(&["jobs"])
            .and_then(ConfigNode::as_mapping)
            .unwrap_or_default()
        {
            for s in body
                .get(&["steps"])
                .and_then(ConfigNode::as_sequence)
                .unwrap_or_default()
            {
                let text = |k: &str| s.get(&[k]).and_then(ConfigNode::as_str).map(str::to_owned);
                steps.push(Step {
                    job: job.clone(),
                    line: s.line,
                    run: text("run"),
                    uses: text("uses"),
                });
            }
        }
        Ok(Self {
            rel: file.rel,
            steps,
        })
    }

    /// Reads a workflow relative to the project root; None when the file is not there
    pub fn read(root: &Path, rel: &str) -> Option<Result<Self, String>> {
        let text = std::fs::read_to_string(root.join(rel)).ok()?;
        Some(Self::parse(rel, text))
    }

    /// Every workflow under [`DIR`] (`.yml` and `.yaml`), by file name
    pub fn all(root: &Path) -> Vec<Result<Self, String>> {
        let mut names: Vec<String> = std::fs::read_dir(root.join(DIR))
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| {
                Path::new(n)
                    .extension()
                    .is_some_and(|x| x == "yml" || x == "yaml")
            })
            .collect();
        names.sort();
        names
            .iter()
            .filter_map(|n| Self::read(root, &format!("{DIR}/{n}")))
            .collect()
    }

    /// The commands of every `run:` step, each with its step
    pub fn commands(&self) -> impl Iterator<Item = (&Step, Vec<String>)> {
        self.steps
            .iter()
            .filter_map(|s| Some((s, s.run.as_deref()?)))
            .flat_map(|(s, run)| commands(run).into_iter().map(move |c| (s, c)))
    }

    /// The first step that uses an action, named without its version (`actions/deploy-pages`)
    pub fn uses(&self, action: &str) -> Option<&Step> {
        self.steps.iter().find(|s| {
            s.uses
                .as_deref()
                .is_some_and(|u| u.split('@').next() == Some(action))
        })
    }
}

/// The simple commands of a shell script, each as its words: lines joined at a trailing `\`, then split at `&&`,
/// `||`, `;` and `|`. Quotes are not interpreted; a word keeps them.
pub fn commands(script: &str) -> Vec<Vec<String>> {
    let joined = script.replace("\\\n", " ");
    joined
        .lines()
        .flat_map(|l| l.split("&&").flat_map(|p| p.split("||")))
        .flat_map(|p| p.split([';', '|']))
        .map(|c| c.split_whitespace().map(str::to_owned).collect::<Vec<_>>())
        .filter(|c| !c.is_empty() && !c[0].starts_with('#'))
        .collect()
}

/// Whether a command runs `program` (anywhere in it, so `uv run zensical` counts) with a first argument that is
/// neither a flag nor a toolchain (`+nightly`) and that `sub` accepts
pub fn invokes(command: &[String], program: &str, sub: impl Fn(&str) -> bool) -> bool {
    command.iter().enumerate().any(|(i, w)| {
        let name = w.rsplit('/').next().unwrap_or(w);
        name == program
            && command[i + 1..]
                .iter()
                .find(|a| !a.starts_with(['-', '+']))
                .is_some_and(|a| sub(a))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_and_commands() {
        let w = Workflow::parse(
            ".github/workflows/ci.yml",
            "on: push\njobs:\n  lint:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: |\n          cargo fmt --check && just lint\n          uv run zensical build \\\n            --strict\n  deploy:\n    steps:\n      - id: d\n        uses: actions/deploy-pages@v4\n".into(),
        )
        .unwrap();
        assert_eq!(w.steps.len(), 3);
        assert_eq!(w.steps[1].line, 7);
        assert_eq!(w.steps[2].job, "deploy");
        assert_eq!(w.uses("actions/deploy-pages").map(|s| s.line), Some(13));
        assert!(w.uses("actions/deploy").is_none());
        let cmds: Vec<Vec<String>> = w.commands().map(|(_, c)| c).collect();
        assert_eq!(cmds.len(), 3);
        assert!(invokes(&cmds[1], "just", |r| r == "lint"));
        assert!(invokes(&cmds[2], "zensical", |r| r == "build"));
        assert!(cmds[2].iter().any(|w| w == "--strict"));
        assert!(!invokes(&cmds[0], "cargo", |r| r == "clippy"));
    }

    #[test]
    fn invokes_skips_flags() {
        let c: Vec<String> = ["/usr/bin/cargo", "+nightly", "-q", "clippy"]
            .map(str::to_owned)
            .into();
        assert!(invokes(&c, "cargo", |r| r == "clippy"));
        assert!(!invokes(&c, "just", |_| true));
    }
}
