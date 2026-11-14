//! Running a CLI binary in a scratch directory: a clean environment, no input, output into pipes, a time limit.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// What one run of the binary gave
#[derive(Clone, Debug, Default)]
pub(crate) struct Run {
    /// The exit code; None when a signal ended it or it was killed
    pub code: Option<i32>,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Killed after the time limit
    pub timed_out: bool,
}

/// An empty directory that is the binary's working directory, `HOME` and XDG directories; removed when dropped
pub(crate) struct Sandbox {
    dir: PathBuf,
    timeout: Duration,
}

impl Sandbox {
    /// A fresh scratch directory named after `key`, unique to this process
    pub(crate) fn new(key: &str, timeout: Duration) -> std::io::Result<Self> {
        let dir =
            std::env::temp_dir().join(format!("spirit-level-cli-{}-{key}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
        for sub in ["work", "home", "config", "data", "state", "cache"] {
            std::fs::create_dir_all(dir.join(sub))?;
        }
        Ok(Self { dir, timeout })
    }

    /// Runs `exe args` with only `PATH`, the scratch `HOME` and XDG directories, `TERM=dumb` and `env` set
    pub(crate) fn run(
        &self,
        exe: &Path,
        args: &[&str],
        env: &[(&str, &str)],
    ) -> std::io::Result<Run> {
        let mut cmd = Command::new(exe);
        cmd.args(args)
            .current_dir(self.dir.join("work"))
            .env_clear()
            .env("HOME", self.dir.join("home"))
            .env("XDG_CONFIG_HOME", self.dir.join("config"))
            .env("XDG_DATA_HOME", self.dir.join("data"))
            .env("XDG_STATE_HOME", self.dir.join("state"))
            .env("XDG_CACHE_HOME", self.dir.join("cache"))
            .env("TERM", "dumb")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(path) = std::env::var_os("PATH") {
            cmd.env("PATH", path);
        }
        for (k, v) in env {
            cmd.env(k, v);
        }
        let mut child = cmd.spawn()?;
        let out = child.stdout.take().map(reader);
        let err = child.stderr.take().map(reader);
        let start = Instant::now();
        let mut timed_out = false;
        let status = loop {
            if let Some(s) = child.try_wait()? {
                break Some(s);
            }
            if start.elapsed() > self.timeout {
                child.kill()?;
                child.wait()?;
                timed_out = true;
                break None;
            }
            std::thread::sleep(Duration::from_millis(5));
        };
        let collect = |h: Option<std::thread::JoinHandle<Vec<u8>>>| {
            h.and_then(|h| h.join().ok())
                .map(|b| String::from_utf8_lossy(&b).into_owned())
                .unwrap_or_default()
        };
        Ok(Run {
            code: status.and_then(|s| s.code()),
            stdout: collect(out),
            stderr: collect(err),
            timed_out,
        })
    }
}

/// The most output read from one stream of one run; a tool that prints more is cut there and killed at the time
/// limit
const OUTPUT_BYTES_MAX: u64 = 4 << 20;

/// Reads a pipe to its end, or to [`OUTPUT_BYTES_MAX`], on a thread of its own, so a full pipe never blocks the
/// child
fn reader(r: impl Read + Send + 'static) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        // a read error ends the output there; what was read is still reported
        r.take(OUTPUT_BYTES_MAX).read_to_end(&mut buf).ok();
        buf
    })
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        // a scratch directory that cannot be removed stays in the temporary directory; the run is not affected
        std::fs::remove_dir_all(&self.dir).ok();
    }
}
