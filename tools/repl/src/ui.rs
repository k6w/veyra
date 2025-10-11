use nu_ansi_term::{Color, Style};

/// UI theme and styling
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub highlight: Color,
    pub muted: Color,
}

impl Theme {
    pub fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
            highlight: Color::Magenta,
            muted: Color::DarkGray,
        }
    }

    pub fn monokai() -> Self {
        Self {
            primary: Color::Rgb(102, 217, 239),
            secondary: Color::Rgb(249, 38, 114),
            success: Color::Rgb(166, 226, 46),
            warning: Color::Rgb(253, 151, 31),
            error: Color::Rgb(249, 38, 114),
            info: Color::Rgb(174, 129, 255),
            highlight: Color::Rgb(230, 219, 116),
            muted: Color::Rgb(117, 113, 94),
        }
    }

    pub fn dracula() -> Self {
        Self {
            primary: Color::Rgb(139, 233, 253),
            secondary: Color::Rgb(189, 147, 249),
            success: Color::Rgb(80, 250, 123),
            warning: Color::Rgb(241, 250, 140),
            error: Color::Rgb(255, 85, 85),
            info: Color::Rgb(98, 114, 164),
            highlight: Color::Rgb(255, 121, 198),
            muted: Color::Rgb(98, 114, 164),
        }
    }

    pub fn nord() -> Self {
        Self {
            primary: Color::Rgb(143, 188, 187),
            secondary: Color::Rgb(129, 161, 193),
            success: Color::Rgb(163, 190, 140),
            warning: Color::Rgb(235, 203, 139),
            error: Color::Rgb(191, 97, 106),
            info: Color::Rgb(94, 129, 172),
            highlight: Color::Rgb(180, 142, 173),
            muted: Color::Rgb(76, 86, 106),
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            primary: Color::Rgb(38, 139, 210), // blue
            secondary: Color::Rgb(211, 54, 130),
            success: Color::Rgb(133, 153, 0),
            warning: Color::Rgb(181, 137, 0),
            error: Color::Rgb(220, 50, 47),
            info: Color::Rgb(42, 161, 152),
            highlight: Color::Rgb(108, 113, 196),
            muted: Color::Rgb(88, 110, 117),
        }
    }

    pub fn solarized_light() -> Self {
        Self {
            primary: Color::Rgb(38, 139, 210),
            secondary: Color::Rgb(211, 54, 130),
            success: Color::Rgb(133, 153, 0),
            warning: Color::Rgb(181, 137, 0),
            error: Color::Rgb(220, 50, 47),
            info: Color::Rgb(42, 161, 152),
            highlight: Color::Rgb(108, 113, 196),
            muted: Color::Rgb(147, 161, 161),
        }
    }
}

pub struct UI {
    pub theme: Theme,
}

impl UI {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Print the welcome banner
    pub fn print_banner(&self) {
        let banner = format!(
            r#"
╔═══════════════════════════════════════════════════════════════════════╗
║                                                                       ║
║     ██╗   ██╗███████╗██╗   ██╗██████╗  █████╗                        ║
║     ██║   ██║██╔════╝╚██╗ ██╔╝██╔══██╗██╔══██╗                       ║
║     ██║   ██║█████╗   ╚████╔╝ ██████╔╝███████║                       ║
║     ╚██╗ ██╔╝██╔══╝    ╚██╔╝  ██╔══██╗██╔══██║                       ║
║      ╚████╔╝ ███████╗   ██║   ██║  ██║██║  ██║                       ║
║       ╚═══╝  ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝                       ║
║                                                                       ║
║            Interactive REPL - Version 0.2.0                          ║
║            The Modern Programming Language                           ║
║                                                                       ║
╚═══════════════════════════════════════════════════════════════════════╝
"#
        );

        println!("{}", self.theme.primary.paint(banner));
        println!(
            "{}",
            Style::new()
                .dimmed()
                .paint("Type ':help' for help, ':exit' to quit")
        );
        println!();
    }

    /// Print a success message
    pub fn success(&self, message: &str) {
        println!(
            "{} {}",
            self.theme.success.paint("✓"),
            self.theme.success.paint(message)
        );
    }

    /// Print an info message
    pub fn info(&self, message: &str) {
        println!("{} {}", self.theme.info.paint("ℹ"), message);
    }

    /// Print a warning message
    pub fn warning(&self, message: &str) {
        println!(
            "{} {}",
            self.theme.warning.paint("⚠"),
            self.theme.warning.paint(message)
        );
    }

    /// Print an error message
    pub fn error(&self, message: &str) {
        eprintln!(
            "{} {}",
            self.theme.error.paint("✗"),
            self.theme.error.paint(message)
        );
    }

    /// Print a section header
    pub fn section(&self, title: &str) {
        println!();
        println!(
            "{}",
            self.theme.primary.bold().paint(format!("▸ {}", title))
        );
        println!("{}", self.theme.muted.paint("─".repeat(70)));
    }

    /// Format the prompt
    pub fn get_prompt(&self, context: &str) -> String {
        // Use only ASCII to avoid variable-width glyph cursor misplacement on some Windows consoles
        let ctx = self.theme.secondary.bold().paint(context);
        // We show a trailing space; user input begins after that space.
        format!("{}> ", ctx)
    }

    /// Format continuation prompt for multiline
    pub fn get_continuation_prompt(&self) -> String {
        // Simple dots with space, no special characters that might cause width issues
        let dots = self.theme.muted.paint("..");
        format!("{}  ", dots)
    }

    /// Print execution result
    pub fn print_result(&self, value: &str, timing: Option<f64>) {
        let arrow = self.theme.highlight.paint("⇒");
        print!("{} {}", arrow, value);

        if let Some(time) = timing {
            print!(" {}", self.theme.muted.paint(format!("({:.3}ms)", time)));
        }
        println!();
    }

    /// Print a separator line
    #[allow(dead_code)]
    pub fn separator(&self) {
        println!("{}", self.theme.muted.paint("═".repeat(70)));
    }

    /// Print a tip
    pub fn tip(&self, tip: &str) {
        println!(
            "{} {}",
            self.theme.info.paint("💡 Tip:"),
            Style::new().italic().paint(tip)
        );
    }

    /// Format syntax error with context
    #[allow(dead_code)]
    pub fn print_syntax_error(&self, error: &str, line: usize, column: usize) {
        self.error(&format!("Syntax Error at line {}, column {}", line, column));
        println!("  {}", self.theme.muted.paint(error));
    }

    /// Format runtime error
    pub fn print_runtime_error(&self, error: &str) {
        self.error(&format!("Runtime Error: {}", error));
    }

    /// Print fancy error with context (miette-style)
    pub fn print_fancy_error(&self, source: &str, error_msg: &str) {
        use miette::Diagnostic;
        use std::fmt;

        // Create a simple diagnostic
        #[derive(Debug)]
        struct ReplError {
            message: String,
        }

        impl fmt::Display for ReplError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.message)
            }
        }

        impl std::error::Error for ReplError {}

        impl Diagnostic for ReplError {
            fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
                Some(Box::new("veyra::runtime_error"))
            }
        }

        let err = ReplError {
            message: error_msg.to_string(),
        };

        // Create a fancy report with source context
        let report = miette::Report::new(err).with_source_code(source.to_string());
        eprintln!("{:?}", report);
    }
}

/// Format a table for display
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new(headers: Vec<String>) -> Self {
        Self {
            headers,
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    pub fn print(&self, theme: &Theme) {
        if self.rows.is_empty() {
            return;
        }

        // Calculate column widths
        let mut widths: Vec<usize> = self.headers.iter().map(|h| h.len()).collect();
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // Print headers
        print!("│ ");
        for (i, header) in self.headers.iter().enumerate() {
            print!(
                "{:width$} │ ",
                theme.primary.bold().paint(header),
                width = widths[i]
            );
        }
        println!();

        // Print separator
        print!("├");
        for width in &widths {
            print!("─{}─┼", "─".repeat(*width));
        }
        println!();

        // Print rows
        for row in &self.rows {
            print!("│ ");
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    print!("{:width$} │ ", cell, width = widths[i]);
                }
            }
            println!();
        }
    }
}

/// Progress indicator for long-running operations
pub fn with_spinner<F, T>(message: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    use indicatif::{ProgressBar, ProgressStyle};

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let result = f();

    pb.finish_and_clear();
    result
}
