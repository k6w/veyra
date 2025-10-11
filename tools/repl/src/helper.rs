use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Helper};
use std::borrow::Cow;
use std::collections::HashSet;

pub struct ReplHelper {
    completer: CommandCompleter,
    hinter: HistoryHinter,
    highlighter: Box<dyn HighlighterAdapter + Send + Sync>,
    validator: BracketValidator,
}

impl ReplHelper {
    pub fn new(enable_highlight: bool, auto_insert_function_parens: bool) -> Self {
        let highlighter: Box<dyn HighlighterAdapter + Send + Sync> = if enable_highlight {
            Box::new(SyntectHighlighter::new())
        } else {
            Box::new(NoColorHighlighter {})
        };
        Self {
            completer: CommandCompleter::new(auto_insert_function_parens),
            hinter: HistoryHinter::new(),
            highlighter,
            validator: BracketValidator::new(),
        }
    }
    pub fn add_variable(&mut self, name: String) {
        self.completer.add_variable(name);
    }
    pub fn add_function(&mut self, name: String) {
        self.completer.add_function(name);
    }
}

impl Helper for ReplHelper {}
impl Highlighter for ReplHelper {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }
    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
    }
}
impl Hinter for ReplHelper {
    type Hint = String;
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        self.hinter
            .hint(line, pos, ctx)
            .filter(|h| !h.is_empty())
            .map(|h| format!("\x1b[90m{}\x1b[0m", h))
    }
}
impl Completer for ReplHelper {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        self.completer.complete(line, pos, ctx)
    }
}
impl Validator for ReplHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        self.validator.validate(ctx)
    }
}

pub struct CommandCompleter {
    keywords: HashSet<String>,
    commands: HashSet<String>,
    variables: HashSet<String>,
    functions: HashSet<String>,
    auto_insert_function_parens: bool,
}
impl CommandCompleter {
    pub fn new(auto_insert_function_parens: bool) -> Self {
        let keywords: HashSet<String> = [
            "let", "const", "mut", "fn", "return", "if", "else", "elif", "match", "case",
            "default", "for", "while", "loop", "break", "continue", "struct", "enum", "trait",
            "impl", "type", "use", "pub", "mod", "async", "await", "defer", "try", "catch",
            "throw", "in", "is", "as", "new", "self", "super", "true", "false", "None", "Some",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let commands: HashSet<String> = [
            ":help",
            ":exit",
            ":quit",
            ":clear",
            ":reset",
            ":history",
            ":vars",
            ":funcs",
            ":info",
            ":config",
            ":save",
            ":load",
            ":time",
            ":type",
            ":ast",
            ":tokens",
            ":verbose",
            ":theme",
            ":multiline",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        Self {
            keywords,
            commands,
            variables: HashSet::new(),
            functions: HashSet::new(),
            auto_insert_function_parens,
        }
    }
    pub fn add_variable(&mut self, name: String) {
        self.variables.insert(name);
    }
    pub fn add_function(&mut self, name: String) {
        self.functions.insert(name);
    }
    fn find_matches(&self, line: &str, pos: usize) -> Vec<Pair> {
        let start = line[..pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != ':')
            .map(|i| i + 1)
            .unwrap_or(0);
        let prefix = &line[start..pos];
        if prefix.is_empty() {
            return vec![];
        }
        let mut out = Vec::new();
        if prefix.starts_with(':') {
            for c in &self.commands {
                if c.starts_with(prefix) {
                    out.push(Self::styled_pair(c, "cmd", c, false));
                }
            }
        } else {
            for k in &self.keywords {
                if k.starts_with(prefix) {
                    out.push(Self::styled_pair(k, "kw", k, false));
                }
            }
            for v in &self.variables {
                if v.starts_with(prefix) {
                    out.push(Self::styled_pair(v, "var", v, false));
                }
            }
            for f in &self.functions {
                if f.starts_with(prefix) {
                    out.push(Self::styled_pair(
                        f,
                        "fn",
                        f,
                        self.auto_insert_function_parens,
                    ));
                }
            }
        }
        out.sort_by(|a, b| a.display.cmp(&b.display));
        out
    }
    fn styled_pair(raw: &str, kind: &str, replacement: &str, auto_parens: bool) -> Pair {
        let (color, label) = match kind {
            "kw" => ("36", "keyword"),
            "var" => ("33", "var"),
            "fn" => ("35", "fn"),
            "cmd" => ("32", "cmd"),
            _ => ("90", kind),
        };
        let repl = if kind == "fn" && auto_parens {
            format!("{}()", replacement)
        } else {
            replacement.to_string()
        };
        Pair {
            display: format!("\x1b[{}m{}\x1b[0m ({})", color, raw, label),
            replacement: repl,
        }
    }
}
impl Completer for CommandCompleter {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let m = self.find_matches(line, pos);
        let start = line[..pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != ':')
            .map(|i| i + 1)
            .unwrap_or(0);
        Ok((start, m))
    }
}

pub trait HighlighterAdapter {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str>;
    fn highlight_char(&self, _: &str, _: usize, _: bool) -> bool {
        true
    }
}
pub struct NoColorHighlighter;
impl HighlighterAdapter for NoColorHighlighter {
    fn highlight<'l>(&self, line: &'l str, _: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }
}
pub struct SyntaxHighlighter {
    keywords: HashSet<String>,
}
impl SyntaxHighlighter {
    pub fn new() -> Self {
        let mut k = HashSet::new();
        for w in [
            "let", "const", "mut", "fn", "return", "if", "else", "elif", "match", "case",
            "default", "for", "while", "loop", "break", "continue", "struct", "enum", "trait",
            "impl", "type", "use", "pub", "mod", "async", "await", "defer", "try", "catch",
            "throw", "in", "is", "as", "new", "self", "super", "true", "false", "None", "Some",
        ] {
            k.insert(w.to_string());
        }
        Self { keywords: k }
    }
}
impl HighlighterAdapter for SyntaxHighlighter {
    fn highlight<'l>(&self, line: &'l str, _: usize) -> Cow<'l, str> {
        use nu_ansi_term::Color;
        let mut out = String::new();
        let mut cur = String::new();
        let mut in_str = false;
        let mut q = '\0';
        let mut str_buf = String::new();
        for ch in line.chars() {
            if in_str {
                if ch == q {
                    in_str = false;
                    str_buf.push(ch);
                    out.push_str(&Color::Green.paint(&str_buf).to_string());
                    str_buf.clear();
                } else {
                    str_buf.push(ch);
                }
                continue;
            }
            if ch == '"' || ch == '\'' {
                if !cur.is_empty() {
                    self.color_word(&mut out, &cur);
                    cur.clear();
                }
                in_str = true;
                q = ch;
                str_buf.push(ch);
            } else if ch.is_alphanumeric() || ch == '_' {
                cur.push(ch);
            } else {
                if !cur.is_empty() {
                    self.color_word(&mut out, &cur);
                    cur.clear();
                }
                out.push(ch);
            }
        }
        if !cur.is_empty() {
            self.color_word(&mut out, &cur);
        }
        if in_str {
            out.push_str(&Color::Green.paint(&str_buf).to_string());
        }
        Cow::Owned(out)
    }
}
impl SyntaxHighlighter {
    fn color_word(&self, out: &mut String, word: &str) {
        use nu_ansi_term::Color;
        if self.keywords.contains(word) {
            out.push_str(&Color::Cyan.bold().paint(word).to_string());
        } else if word == "true" || word == "false" {
            out.push_str(&Color::Yellow.paint(word).to_string());
        } else if word.chars().all(|c| c.is_numeric()) {
            out.push_str(&Color::Magenta.paint(word).to_string());
        } else {
            out.push_str(word);
        }
    }
}
pub struct SyntectHighlighter {
    ps: syntect::parsing::SyntaxSet,
    ts: syntect::highlighting::ThemeSet,
    theme: syntect::highlighting::Theme,
}
impl SyntectHighlighter {
    pub fn new() -> Self {
        let ps = syntect::parsing::SyntaxSet::load_defaults_newlines();
        let ts = syntect::highlighting::ThemeSet::load_defaults();
        let theme = ts
            .themes
            .get("base16-ocean.dark")
            .cloned()
            .or_else(|| ts.themes.values().next().cloned())
            .expect("theme");
        Self { ps, ts, theme }
    }
}
impl HighlighterAdapter for SyntectHighlighter {
    fn highlight<'l>(&self, line: &'l str, _: usize) -> Cow<'l, str> {
        use syntect::easy::HighlightLines;
        use syntect::util::as_24_bit_terminal_escaped;
        if line.trim().is_empty() {
            return Cow::Borrowed(line);
        }
        let syntax = self
            .ps
            .find_syntax_by_extension("rs")
            .unwrap_or(self.ps.find_syntax_plain_text());
        let mut h = HighlightLines::new(syntax, &self.theme);
        let ranges = h.highlight_line(line, &self.ps).unwrap_or_default();
        Cow::Owned(as_24_bit_terminal_escaped(&ranges[..], false))
    }
}

pub struct BracketValidator;
impl BracketValidator {
    pub fn new() -> Self {
        Self
    }

    fn count(&self, line: &str) -> (i32, i32, i32) {
        let (mut p, mut b, mut c, mut in_s, mut esc) = (0, 0, 0, false, false);
        let mut quote = '\0';
        for ch in line.chars() {
            if esc {
                esc = false;
                continue;
            }
            if ch == '\\' {
                esc = true;
                continue;
            }
            if ch == '"' || ch == '\'' {
                if in_s && ch == quote {
                    in_s = false;
                } else if !in_s {
                    in_s = true;
                    quote = ch;
                }
                continue;
            }
            if in_s {
                continue;
            }
            match ch {
                '(' => p += 1,
                ')' => p -= 1,
                '[' => b += 1,
                ']' => b -= 1,
                '{' => c += 1,
                '}' => c -= 1,
                _ => {}
            }
        }
        (p, b, c)
    }
}
impl Validator for BracketValidator {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();
        if input.trim().starts_with(':') {
            return Ok(ValidationResult::Valid(None));
        }
        let (p, b, c) = self.count(input);
        if p > 0 || b > 0 || c > 0 {
            return Ok(ValidationResult::Incomplete);
        }
        if p < 0 || b < 0 || c < 0 {
            return Ok(ValidationResult::Invalid(Some(
                "Unmatched closing bracket".into(),
            )));
        }
        Ok(ValidationResult::Valid(None))
    }
}
