//! Stop-gap smells and agent-config hygiene: marker comments, commented-out code, lint tables that allow without
//! a reason, features that enable nothing, near-duplicate functions and modules left from a rewrite, and files
//! committed by mistake (leftovers, agent settings with secrets or broad permissions, session transcripts).

mod dupes;
mod hygiene;
mod stopgap;

use sl_analysis::{Analysis, Context, lint};
use sl_core::{LintMeta, Need};

lint!(STOP_GAP_COMMENTS, "A0010", "stop-gap-comments", "stop-gap", "comments carry no stop-gap markers: TODO, HACK, for now, workaround",
    params: [("markers", StrList, "[\"TODO\", \"XXX\", \"HACK\"]", "markers matched as whole words, case-sensitive; FIXME is fixme-before-land's"),
             ("phrases", StrList, "[\"for now\", \"hack\", \"workaround\", \"shim\", \"quick fix\", \"temporary fix\", \"temporary solution\", \"temporary workaround\", \"temporary hack\", \"temporary code\", \"temporary implementation\", \"temporary patch\", \"is temporary\", \"temporarily disabled\", \"fix later\", \"refactor later\", \"clean up later\"]", "phrases matched as whole words, case-insensitive"),
             ("tracked-ok", Bool, "true", "a comment with a tracker id (`CFG-3`), an issue number (`#12`) or a URL is tracked work and passes"),
             ("docs", Bool, "true", "also check doc comments")],
    needs: [], fix: None);
lint!(FIXME_BEFORE_LAND, "A0011", "fixme-before-land", "stop-gap", "no FIXME is left when a branch lands",
    params: [("markers", StrList, "[\"FIXME\"]", "markers of a note to revisit before landing, matched as whole words, case-sensitive"),
             ("docs", Bool, "true", "also check doc comments")],
    needs: [], fix: None);
lint!(COMMENTED_OUT_CODE, "A0005", "commented-out-code", "stop-gap", "no code is left commented out",
    params: [("min-lines", Int, "1", "lines a run of comments needs to be judged")],
    needs: [], fix: None);
lint!(LINT_TABLE_ALLOW_REASON, "A0007", "lint-table-allow-reason", "stop-gap", "a lint allowed in a Cargo.toml [lints] table says why in a comment",
    params: [], needs: [], fix: None);
lint!(NEAR_DUPLICATE_FNS, "A0008", "near-duplicate-fns", "stop-gap", "no two functions are near copies of each other",
    params: [("min-similarity", Float, "0.9", "Jaccard index of normalized 4-token shingles from which two functions are copies"),
             ("min-tokens", Int, "60", "tokens a function needs to be compared")],
    needs: [], fix: None);
lint!(NEAR_DUPLICATE_MODULES, "A0009", "near-duplicate-modules", "stop-gap", "no two modules are near copies of each other",
    params: [("min-share", Float, "0.7", "share of the two files' functions that are near copies from which the files are"),
             ("min-fns", Int, "2", "matched functions each file needs")],
    needs: [], fix: None);
lint!(DEAD_FEATURES, "A0006", "dead-features", "stop-gap", "every cargo feature enables something or is tested by a cfg",
    params: [("skip", StrList, "[\"default\"]", "features never reported")],
    needs: [], fix: None);
lint!(LEFTOVER_FILES, "A0004", "leftover-files", "hygiene", "no merge, patch, editor or scratch leftovers are committed",
    params: [("patterns", StrList, "[\"**/*.orig\", \"**/*.rej\", \"**/*.bak\", \"**/*~\", \"**/*.swp\", \"**/*.tmp\", \"**/scratch*\", \"**/HANDOFF.md\"]", "globs of tracked files that are leftovers")],
    needs: [Need::Tool("git")], fix: None);
lint!(LARGE_BLOBS, "A0000", "large-blobs", "hygiene", "no git-tracked file is over the size limit outside the allow list",
    params: [("max-bytes", Int, "1000000", "the size in bytes a tracked file may reach before it is flagged"),
             ("allow", StrList, "[]", "globs of tracked files allowed past the limit")],
    needs: [Need::Tool("git")], fix: None);
lint!(AGENT_CONFIG_SECRETS, "A0002", "agent-config-secrets", "hygiene", "committed agent configuration holds no secrets",
    params: [("files", StrList, "[\"**/.claude/**\", \"**/.mcp.json\", \"**/.cursor/**\", \"**/.codex/**\", \"**/.continue/**\", \"**/.windsurf/**\", \"**/.gemini/**\", \"**/.vscode/mcp.json\"]", "globs of agent configuration files"),
             ("keys", StrList, "[\"token\", \"secret\", \"password\", \"passwd\", \"api_key\", \"apikey\", \"api-key\", \"private_key\", \"access_key\", \"authorization\"]", "parts of a key name whose literal value is a secret"),
             ("prefixes", StrList, "[\"sk-\", \"ghp_\", \"gho_\", \"ghs_\", \"github_pat_\", \"xoxb-\", \"xoxp-\", \"glpat-\", \"AKIA\", \"AIza\"]", "prefixes of known token formats")],
    needs: [Need::Tool("git")], fix: None);
lint!(AGENT_CONFIG_PERMISSIONS, "A0001", "agent-config-permissions", "hygiene", "committed agent settings grant no blanket permissions",
    params: [("files", StrList, "[\"**/.claude/settings.json\", \"**/.claude/settings.local.json\"]", "globs of agent settings files (JSON)"),
             ("broad", StrList, "[\"Bash\", \"Bash(*)\", \"Bash(:*)\", \"Bash(*:*)\", \"*\", \"Bash(sudo:*)\", \"Bash(rm:*)\", \"Bash(sh:*)\", \"Bash(bash:*)\", \"Bash(eval:*)\", \"Bash(curl:*)\", \"Bash(git push:*)\"]", "permission rules that grant too much")],
    needs: [Need::Tool("git")], fix: None);
lint!(AGENT_SESSION_FILES, "A0003", "agent-session-files", "hygiene", "no agent transcripts or per-person agent files are committed",
    params: [("patterns", StrList, "[\"**/.claude/settings.local.json\", \"**/CLAUDE.local.md\", \"**/.claude/projects/**\", \"**/.claude/todos/**\", \"**/.claude/**/*.jsonl\", \"**/*transcript*\", \"**/.aider.chat.history.md\", \"**/.aider.input.history\", \"**/.specstory/**\"]", "globs of tracked files that are session files")],
    needs: [Need::Tool("git")], fix: None);

/// The stop-gap and hygiene analysis
pub struct Ai;

/// The registered instance
pub static AI: Ai = Ai;

static LINTS: &[&LintMeta] = &[
    &STOP_GAP_COMMENTS,
    &FIXME_BEFORE_LAND,
    &COMMENTED_OUT_CODE,
    &LINT_TABLE_ALLOW_REASON,
    &NEAR_DUPLICATE_FNS,
    &NEAR_DUPLICATE_MODULES,
    &DEAD_FEATURES,
    &LEFTOVER_FILES,
    &LARGE_BLOBS,
    &AGENT_CONFIG_SECRETS,
    &AGENT_CONFIG_PERMISSIONS,
    &AGENT_SESSION_FILES,
];

impl Analysis for Ai {
    fn name(&self) -> &'static str {
        "ai"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        stopgap::run(cx);
        dupes::run(cx);
        hygiene::run(cx);
    }
}
