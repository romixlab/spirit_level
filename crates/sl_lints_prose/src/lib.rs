//! Prose lints: the terse-output limits (CHANGELOG entries, commit messages, agent files, sentences), the marks of
//! text written on autopilot (filler, narration, stream-of-thought, emoji, comments that restate the code), and
//! agent files that point where they should and agree with the lints. One analysis, [`PROSE`]: it reads Markdown
//! files and Rust comments, and the branch's commit messages with git.

mod agents;
mod changelog;
mod commits;
mod fixes;
mod hygiene;
mod style;
mod text;
mod tracking;
mod units;

use sl_analysis::{Analysis, Context, lint};
use sl_core::{LintMeta, Need};

lint!(CHANGELOG_ENTRY_LENGTH, "P0009", "changelog-entry-length", "prose", "a CHANGELOG entry is one short line",
    params: [("max-lines", Int, "1", "lines an entry may span"),
             ("max-words", Int, "20", "words an entry may have"),
             ("file", Str, "\"CHANGELOG.md\"", "the changelog, relative to the root"),
             ("all-sections", Bool, "false", "check released sections too, not only `## [Unreleased]`")],
    needs: [], fix: None);
lint!(CHANGELOG_DESCRIBES_DIFF, "P0008", "changelog-describes-diff", "prose", "a CHANGELOG entry says what changed for the user, not which code was edited",
    params: [("extensions", StrList, "[\".rs\"]", "file endings that mark a source file named in an entry"),
             ("verbs", StrList, "[\"update\", \"updated\", \"modify\", \"modified\", \"edit\", \"edited\", \"change\", \"changed\", \"tweak\", \"tweaked\", \"refactor\", \"refactored\", \"clean\", \"cleaned\", \"touch\", \"touched\"]", "edit verbs that open a diff description"),
             ("code-words", StrList, "[\"file\", \"files\", \"function\", \"functions\", \"fn\", \"method\", \"methods\", \"struct\", \"module\", \"modules\", \"line\", \"lines\", \"code\", \"impl\"]", "objects of an edit verb that name code rather than behaviour"),
             ("file", Str, "\"CHANGELOG.md\"", "the changelog, relative to the root"),
             ("all-sections", Bool, "false", "check released sections too, not only `## [Unreleased]`")],
    needs: [], fix: None);
lint!(AGENT_FILE_LINES, "P0001", "agent-file-lines", "agents", "an agent file stays under its line limit",
    params: [("files", StrList, "[\"AGENTS.md\"]", "the agent files, relative to the root"),
             ("max-lines", Int, "150", "lines a file may have")],
    needs: [], fix: None);
lint!(CLAUDE_MD_DUPLICATE, "P0004", "claude-md-duplicate", "agents", "CLAUDE.md points to AGENTS.md instead of copying it",
    params: [("file", Str, "\"CLAUDE.md\"", "the tool-specific agent file"),
             ("agents", Str, "\"AGENTS.md\"", "the file that holds the rules"),
             ("max-shared", Float, "0.3", "share of the file's lines that may also be in the rules file")],
    needs: [], fix: None);
lint!(AGENT_FILE_LINKS, "P0002", "agent-file-links", "agents", "links and @imports in agent files resolve",
    params: [("files", StrList, "[\"AGENTS.md\", \"CLAUDE.md\"]", "the agent files, relative to the root")],
    needs: [], fix: None);
lint!(AGENT_RULES_MATCH_LINTS, "P0003", "agent-rules-match-lints", "agents", "limits an agent file states are the limits the lints check",
    params: [("files", StrList, "[\"AGENTS.md\"]", "the agent files, relative to the root")],
    needs: [], fix: None);
lint!(FILLER_PHRASES, "P0011", "filler-phrases", "prose", "docs and comments use no filler phrases",
    params: [("phrases", StrList, "[\"note that\", \"it is worth\", \"it's worth\", \"it should be noted\", \"it is important to note\", \"needless to say\", \"as mentioned\", \"as noted above\", \"in order to\", \"basically\", \"essentially\", \"obviously\", \"of course\", \"at the end of the day\", \"keep in mind that\", \"please note\"]", "the banned phrases, matched as whole words, case-insensitive"),
             ("files", StrList, "[\"**/*.md\"]", "globs of the Markdown files checked, relative to the root"),
             ("skip", StrList, "[]", "globs of Markdown files left out"),
             ("comments", Bool, "true", "also check Rust comments")],
    needs: [], fix: None);
lint!(NARRATION, "P0012", "narration", "prose", "docs and comments state facts, not how they were written",
    params: [("phrases", StrList, "[\"let me\", \"let's\", \"as requested\", \"as you asked\", \"as discussed\", \"now we\", \"now i\", \"first i\", \"first we\", \"then i\", \"next i\", \"next we\", \"i'll\", \"i've\", \"i will\", \"we'll\", \"here's\", \"here is the\"]", "the narration phrases, matched as whole words, case-insensitive"),
             ("files", StrList, "[\"**/*.md\"]", "globs of the Markdown files checked, relative to the root"),
             ("skip", StrList, "[]", "globs of Markdown files left out"),
             ("comments", Bool, "true", "also check Rust comments")],
    needs: [], fix: None);
lint!(THINKING_COMMENTS, "P0016", "thinking-comments", "prose", "comments state conclusions, not a stream of thought",
    params: [("phrases", StrList, "[\"hmm\", \"wait,\", \"actually,\", \"i think\", \"i guess\", \"not sure\", \"let's see\", \"ok so\", \"okay so\", \"oops\", \"on second thought\", \"i wonder\"]", "the phrases, matched as whole words, case-insensitive"),
             ("questions", Bool, "true", "also report comments that end with a question mark")],
    needs: [], fix: None);
lint!(EMOJI, "P0010", "emoji", "prose", "docs and comments carry no emoji",
    params: [("allow", StrList, "[\"✅\", \"🚧\", \"📋\", \"🐛\"]", "emoji allowed (status marks of FEATURES.md)"),
             ("files", StrList, "[\"**/*.md\"]", "globs of the Markdown files checked, relative to the root"),
             ("skip", StrList, "[]", "globs of Markdown files left out"),
             ("comments", Bool, "true", "also check Rust comments")],
    needs: [], fix: None);
lint!(RESTATING_COMMENTS, "P0014", "restating-comments", "prose", "a comment says what the code cannot",
    params: [("min-overlap", Float, "0.75", "share of the comment's content words found in the next line from which it restates it"),
             ("min-words", Int, "2", "content words a comment needs to be judged"),
             ("max-words", Int, "8", "comments with more content words are not judged")],
    needs: [], fix: None);
lint!(SENTENCE_WORDS, "P0015", "sentence-words", "prose", "sentences in docs stay short",
    params: [("max-words", Int, "25", "words a sentence may have"),
             ("files", StrList, "[\"**/*.md\"]", "globs of the Markdown files checked, relative to the root"),
             ("skip", StrList, "[]", "globs of Markdown files left out")],
    needs: [], fix: None);
lint!(PARAGRAPH_SENTENCES, "P0013", "paragraph-sentences", "prose", "paragraphs in docs stay short",
    params: [("max-sentences", Int, "2", "sentences a paragraph or list item may have"),
             ("files", StrList, "[\"**/*.md\"]", "globs of the Markdown files checked, relative to the root"),
             ("skip", StrList, "[]", "globs of Markdown files left out")],
    needs: [], fix: None);
lint!(COMMIT_SUBJECT_LENGTH, "P0007", "commit-subject-length", "commits", "a commit subject stays under its length limit",
    params: [("max-chars", Int, "60", "characters a subject may have"),
             ("base", Str, "\"main\"", "the branch work merges into; commits in base..HEAD are checked"),
             ("after", Str, "\"\"", "a commit or tag; only commits after it are checked, for history that cannot be rewritten")],
    needs: [Need::Tool("git")], fix: None);
lint!(COMMIT_BODY_LINES, "P0005", "commit-body-lines", "commits", "a commit body stays short",
    params: [("max-lines", Int, "5", "lines a body may have, trailers not counted"),
             ("base", Str, "\"main\"", "the branch work merges into; commits in base..HEAD are checked"),
             ("after", Str, "\"\"", "a commit or tag; only commits after it are checked, for history that cannot be rewritten")],
    needs: [Need::Tool("git")], fix: None);
lint!(COMMIT_PROSE, "P0006", "commit-prose", "commits", "commit messages use no filler, narration or emoji and do not describe the diff",
    params: [("base", Str, "\"main\"", "the branch work merges into; commits in base..HEAD are checked"),
             ("after", Str, "\"\"", "a commit or tag; only commits after it are checked, for history that cannot be rewritten")],
    needs: [Need::Tool("git")], fix: None);
lint!(FIXES_TAG, "P0017", "fixes-tag", "commits", "a Fixes: trailer names its culprit by a hash that resolves and the culprit's subject",
    params: [("trailer", Str, "\"Fixes\"", "the trailer key that names the commit a fix repairs"),
             ("min-hex", Int, "12", "hex digits the hash must have"),
             ("base", Str, "\"main\"", "the branch work merges into; commits in base..HEAD are checked"),
             ("after", Str, "\"\"", "a commit or tag; only commits after it are checked, for history that cannot be rewritten")],
    needs: [Need::Tool("git")], fix: None);
lint!(FIX_WITHOUT_TEST, "P0018", "fix-without-test", "commits", "a commit that fixes code adds a test",
    params: [("fix-words", StrList, "[\"fix\", \"fixes\", \"fixed\"]", "words that, opening a subject, make a commit a fix (case-insensitive); a `Fixes:` trailer always does"),
             ("code", StrList, "[\"**/*.rs\"]", "globs of the files whose change makes a fix need a test"),
             ("tests", StrList, "[\"tests/**\", \"**/tests/**\", \"**/benches/**\", \"fuzz/**\"]", "globs of test files: changing one counts as a test"),
             ("markers", StrList, "[\"#[test]\", \"#[tokio::test\", \"#[test_case\", \"#[rstest\", \"proptest!\"]", "text of an added line that counts as a test"),
             ("base", Str, "\"main\"", "the branch work merges into; commits in base..HEAD are checked"),
             ("after", Str, "\"\"", "a commit or tag; only commits after it are checked, for history that cannot be rewritten")],
    needs: [Need::Tool("git")], fix: None);
lint!(COMMIT_TESTED_LINE, "P0019", "commit-tested-line", "commits", "a commit says how it was tested in a Tested: trailer",
    params: [("trailer", Str, "\"Tested\"", "the trailer key that says how the commit was tested"),
             ("base", Str, "\"main\"", "the branch work merges into; commits in base..HEAD are checked"),
             ("after", Str, "\"\"", "a commit or tag; only commits after it are checked, for history that cannot be rewritten")],
    needs: [Need::Tool("git")], fix: None);
lint!(COMMIT_MIXED_MOVE, "P0020", "commit-mixed-move", "commits", "a commit that moves a file does not also edit it beyond its paths",
    params: [("files", StrList, "[\"**/*.rs\"]", "globs of the moved files checked"),
             ("path-lines", StrList, "[\"use \", \"pub use \", \"pub(crate) use \", \"mod \", \"pub mod \", \"pub(crate) mod \", \"#[path\", \"//!\"]", "starts of changed lines that only follow the move"),
             ("base", Str, "\"main\"", "the branch work merges into; commits in base..HEAD are checked"),
             ("after", Str, "\"\"", "a commit or tag; only commits after it are checked, for history that cannot be rewritten")],
    needs: [Need::Tool("git")], fix: None);
lint!(URL_TRACKING_PARAMS, "P0021", "url-tracking-params", "prose", "a link in docs or a comment carries no tracking query parameter",
    params: [("params", StrList, "[\"utm_\", \"?si=\", \"&si=\", \"fbclid=\", \"gclid=\", \"msclkid=\", \"igshid=\", \"mc_cid=\", \"mc_eid=\", \"yclid=\", \"ref_src=\", \"spm=\", \"_hsenc=\", \"_hsmi=\"]", "the tracking parameters, matched as substrings, case-insensitive"),
             ("files", StrList, "[\"**/*.md\"]", "globs of the Markdown files checked, relative to the root"),
             ("skip", StrList, "[]", "globs of Markdown files left out"),
             ("comments", Bool, "true", "also check Rust comments")],
    needs: [], fix: None);

/// The prose analysis; the commit lints read the history with git and need it
pub struct Prose;

/// The registered instance
pub static PROSE: Prose = Prose;

static PROSE_LINTS: &[&LintMeta] = &[
    &CHANGELOG_ENTRY_LENGTH,
    &CHANGELOG_DESCRIBES_DIFF,
    &AGENT_FILE_LINES,
    &CLAUDE_MD_DUPLICATE,
    &AGENT_FILE_LINKS,
    &AGENT_RULES_MATCH_LINTS,
    &FILLER_PHRASES,
    &NARRATION,
    &THINKING_COMMENTS,
    &EMOJI,
    &RESTATING_COMMENTS,
    &SENTENCE_WORDS,
    &PARAGRAPH_SENTENCES,
    &COMMIT_SUBJECT_LENGTH,
    &COMMIT_BODY_LINES,
    &COMMIT_PROSE,
    &FIXES_TAG,
    &FIX_WITHOUT_TEST,
    &COMMIT_TESTED_LINE,
    &COMMIT_MIXED_MOVE,
    &URL_TRACKING_PARAMS,
];

impl Analysis for Prose {
    fn name(&self) -> &'static str {
        "prose"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        PROSE_LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        changelog::run(cx);
        agents::run(cx);
        style::run(cx);
        commits::run(cx);
        fixes::run(cx);
        hygiene::run(cx);
        tracking::run(cx);
    }
}
