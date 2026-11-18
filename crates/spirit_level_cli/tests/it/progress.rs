//! The progress display: drawn on stderr when it is a terminal (a pseudo-terminal here), cleared at the end, never
//! on stdout; nothing when stderr is a pipe.

use rustix::pty::{OpenptFlags, grantpt, openpt, ptsname, unlockpt};
use rustix::termios::{Winsize, tcsetwinsize};
use std::io::Read;
use std::os::fd::OwnedFd;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Erases the line the cursor is on, after a carriage return: how indicatif clears its last line
const CLEAR_LINE: &[u8] = b"\r\x1b[2K";

fn project() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/sample_workspace")
}

/// `spl check` of the sample workspace with only native lints, in an environment where progress may be drawn
fn check(extra: &[&str]) -> Command {
    let mut cmd = crate::spl();
    cmd.arg("check")
        .arg(project())
        .args(["--only", "workspace-lints"])
        .args(extra)
        .env("SPIRIT_LEVEL_TODAY", "2026-11-13")
        .env("TERM", "xterm-256color")
        .env_remove("NO_COLOR")
        .env_remove("CI")
        .stdin(Stdio::null())
        .stdout(Stdio::piped());
    cmd
}

/// A pseudo-terminal of 100 columns and 30 rows: the controller to read and the terminal side
fn pty() -> (OwnedFd, std::fs::File) {
    let controller = openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY).expect("open a pty");
    grantpt(&controller).expect("grant the pty");
    unlockpt(&controller).expect("unlock the pty");
    let name = ptsname(&controller, Vec::new()).expect("the pty's name");
    let terminal = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(name.to_str().expect("a UTF-8 pty path"))
        .expect("open the pty's terminal side");
    tcsetwinsize(
        &terminal,
        Winsize {
            ws_row: 30,
            ws_col: 100,
            ws_xpixel: 0,
            ws_ypixel: 0,
        },
    )
    .expect("set the pty's size");
    (controller, terminal)
}

/// Runs `cmd` with stderr at a pseudo-terminal: its stdout and what reached the terminal
fn run_at_terminal(mut cmd: Command) -> (Vec<u8>, Vec<u8>) {
    let (controller, terminal) = pty();
    let child = cmd.stderr(terminal).spawn().expect("run spl");
    // the parent's copy of the terminal side went into the command and is closed now, so the read below ends
    // (EIO on Linux) once spl exits
    drop(cmd);
    let reader = std::thread::spawn(move || {
        let mut file = std::fs::File::from(controller);
        let mut seen = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            match file.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => seen.extend_from_slice(&buf[..n]),
            }
        }
        seen
    });
    let out = child.wait_with_output().expect("wait for spl");
    let seen = reader.join().expect("read the pty");
    (out.stdout, seen)
}

#[test]
fn progress_is_drawn_at_a_terminal_and_cleared() {
    let (stdout, stderr) = run_at_terminal(check(&[]));
    let shown = String::from_utf8_lossy(&stderr);
    assert!(shown.contains("spl check"), "{shown:?}");
    assert!(shown.contains("analyses"), "{shown:?}");
    assert!(stderr.ends_with(CLEAR_LINE), "not cleared: {shown:?}");

    let piped = check(&[]).stderr(Stdio::piped()).output().expect("run spl");
    assert_eq!(stdout, piped.stdout, "stdout differs with progress drawn");
    assert!(
        stdout.starts_with(b"deny"),
        "{}",
        String::from_utf8_lossy(&stdout)
    );
}

#[test]
fn json_and_never_draw_nothing_at_a_terminal() {
    for extra in [&["--format", "json"][..], &["--progress", "never"]] {
        let (_, stderr) = run_at_terminal(check(extra));
        assert!(
            stderr.is_empty(),
            "{extra:?}: {:?}",
            String::from_utf8_lossy(&stderr)
        );
    }
}

#[test]
fn nothing_is_drawn_when_stderr_is_not_a_terminal() {
    let out = check(&[]).stderr(Stdio::piped()).output().expect("run spl");
    assert!(
        out.stderr.is_empty(),
        "{:?}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn always_writes_one_line_per_step_without_a_terminal() {
    let out = check(&["--progress", "always"])
        .stderr(Stdio::piped())
        .output()
        .expect("run spl");
    let lines = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        lines,
        "spl check: loading the project\nspl check: finding the tools\nspl check: [1/1] manifest\nspl check: reading the waivers\n"
    );
}
