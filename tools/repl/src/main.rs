mod ansi;
mod commands;
mod config;
mod helper;
mod state;
mod ui;

use anyhow::Result;
use clap::Parser;
use config::ReplConfig;
use helper::ReplHelper;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{EditMode, Editor};
use state::{format_value, ReplState};
use std::path::PathBuf;
use ui::{Theme, UI};

#[derive(Parser)]
#[command(
    name = "veyra-repl",
    about = "Interactive REPL for the Veyra programming language",
    version = "0.2.0",
    author = "Veyra Team"
)]
struct Cli {
    /// Load and execute a startup file
    #[arg(short, long, value_name = "FILE")]
    startup: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Disable syntax highlighting
    #[arg(long)]
    no_highlight: bool,

    /// Disable all color output
    #[arg(long)]
    no_color: bool,

    /// Disable auto-completion
    #[arg(long)]
    no_completion: bool,

    /// Enable VI mode
    #[arg(long)]
    vi_mode: bool,

    /// Color theme
    #[arg(long, value_name = "THEME")]
    theme: Option<String>,

    /// Don't show tips on startup
    #[arg(long)]
    no_tips: bool,

    /// Execute code and exit
    #[arg(short, long, value_name = "CODE")]
    execute: Option<String>,
}

fn main() -> Result<()> {
    // Parse CLI
    let cli = Cli::parse();
    // Initialize ANSI/Color environment early
    ansi::init(cli.no_color);

    // Load or create configuration
    let mut config = ReplConfig::load().unwrap_or_default();

    // Override config with CLI arguments
    if cli.no_highlight {
        config.syntax_highlighting = false;
    }
    if cli.no_completion {
        config.auto_completion = false;
    }
    if cli.vi_mode {
        config.vi_mode = true;
    }
    if cli.no_tips {
        config.show_tips = false;
    }
    if let Some(startup) = cli.startup {
        config.startup_script = Some(startup);
    }

    // Initialize UI with theme from config or CLI
    let theme = if let Some(ref theme_name) = cli.theme {
        match theme_name.as_str() {
            "monokai" => Theme::monokai(),
            "dracula" => Theme::dracula(),
            "nord" => Theme::nord(),
            "solarized-dark" => Theme::solarized_dark(),
            "solarized-light" => Theme::solarized_light(),
            _ => Theme::default(),
        }
    } else {
        match config.color_scheme {
            crate::config::ColorScheme::Monokai => Theme::monokai(),
            crate::config::ColorScheme::Dracula => Theme::dracula(),
            crate::config::ColorScheme::Nord => Theme::nord(),
            crate::config::ColorScheme::SolarizedDark => Theme::solarized_dark(),
            crate::config::ColorScheme::SolarizedLight => Theme::solarized_light(),
            _ => Theme::default(),
        }
    };
    let mut ui = UI::new(theme);

    // If execute mode, run code and exit
    if let Some(code) = cli.execute {
        let mut state = ReplState::new(config);
        match state.execute(&code) {
            Ok(Some(value)) => {
                println!("{}", format_value(&value));
                return Ok(());
            }
            Ok(None) => return Ok(()),
            Err(e) => {
                ui.error(&format!("Execution error: {}", e));
                std::process::exit(1);
            }
        }
    }

    // Print banner
    ui.print_banner();

    // Show tips if enabled
    if config.show_tips {
        show_startup_tip(&ui);
    }

    // Initialize REPL state
    let mut state = ReplState::new(config.clone());

    // Load startup script if specified
    if let Some(ref startup_path) = config.startup_script {
        if startup_path.exists() {
            ui.info(&format!(
                "Loading startup script: {}",
                startup_path.display()
            ));
            if let Err(e) = state.load_file(startup_path) {
                ui.error(&format!("Failed to load startup script: {}", e));
            } else {
                ui.success("Startup script loaded");
            }
            println!();
        }
    }

    // Create rustyline editor with helper
    let mut rl = Editor::<ReplHelper, rustyline::history::FileHistory>::new()?;

    // Set edit mode
    if config.vi_mode {
        rl.set_edit_mode(EditMode::Vi);
    } else {
        rl.set_edit_mode(EditMode::Emacs);
    }

    // Set helper for completion and highlighting (advanced if syntect present)
    if config.auto_completion || (config.syntax_highlighting && !cli.no_highlight) {
        let enable_highlight = config.syntax_highlighting && !cli.no_highlight;
        rl.set_helper(Some(ReplHelper::new(
            enable_highlight,
            config.auto_insert_function_parens,
        )));
    }

    // Load history
    let history_path = ReplConfig::history_path()?;
    if let Some(parent) = history_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let _ = rl.load_history(&history_path);

    // Main REPL loop
    loop {
        // Determine prompt
        let prompt = if state.is_multiline() {
            ui.get_continuation_prompt()
        } else {
            ui.get_prompt("veyra")
        };

        // Read line
        match rl.readline(&prompt) {
            Ok(line) => {
                let input = line.trim();

                // Skip empty lines
                if input.is_empty() && !state.is_multiline() {
                    continue;
                }

                // Handle REPL commands
                if input.starts_with(':') && !state.is_multiline() {
                    rl.add_history_entry(&line)?;

                    match commands::handle_command(input, &mut state, &mut ui) {
                        Ok(true) => continue,
                        Ok(false) => break,
                        Err(e) => {
                            ui.error(&format!("Command error: {}", e));
                            continue;
                        }
                    }
                }

                // Add to history
                rl.add_history_entry(&line)?;

                // Handle multiline input (with auto-indent)
                if state.config().multiline_mode && needs_more_lines(input) {
                    state.add_to_multiline(input);
                    continue;
                }

                // Get complete input
                let complete_input = if state.is_multiline() {
                    state.add_to_multiline(input);
                    state.take_multiline()
                } else {
                    input.to_string()
                };

                // Execute code
                match state.execute(&complete_input) {
                    Ok(Some(value)) => {
                        // Extract symbols (variables/functions) from input for completion
                        if let Some(helper) = rl.helper_mut() {
                            extract_symbols_for_completion(&complete_input, helper);
                        }

                        let timing = if state.config().show_timing {
                            state.last_timing()
                        } else {
                            None
                        };
                        let value_str = format_value(&value);
                        let tname = state::type_name(&value);
                        ui.print_result(&format!("[{}] {}", tname, value_str), timing);
                    }
                    Ok(None) => {
                        // No output for statements like variable declarations
                        if let Some(helper) = rl.helper_mut() {
                            extract_symbols_for_completion(&complete_input, helper);
                        }
                    }
                    Err(e) => {
                        if state.config().fancy_errors {
                            ui.print_fancy_error(&complete_input, &e.to_string());
                        } else {
                            ui.print_runtime_error(&e.to_string());
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C - clear current input or exit
                if state.is_multiline() {
                    state.take_multiline();
                    ui.warning("Input cancelled");
                } else {
                    ui.info("Press Ctrl+D or type :exit to quit");
                }
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D - exit
                println!();
                ui.success("Goodbye! 👋");
                break;
            }
            Err(err) => {
                ui.error(&format!("Error: {}", err));
                break;
            }
        }
    }

    // Save history
    if config.auto_save_history {
        if let Err(e) = rl.save_history(&history_path) {
            eprintln!("Warning: Failed to save history: {}", e);
        }
    }

    Ok(())
}

/// Check if input needs more lines (unclosed brackets)
fn needs_more_lines(input: &str) -> bool {
    let mut parens = 0;
    let mut brackets = 0;
    let mut braces = 0;
    let mut in_string = false;
    let mut string_char = '\0';
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        if ch == '\\' {
            escape_next = true;
            continue;
        }

        if ch == '"' || ch == '\'' {
            if in_string && ch == string_char {
                in_string = false;
            } else if !in_string {
                in_string = true;
                string_char = ch;
            }
            continue;
        }

        if in_string {
            continue;
        }

        match ch {
            '(' => parens += 1,
            ')' => parens -= 1,
            '[' => brackets += 1,
            ']' => brackets -= 1,
            '{' => braces += 1,
            '}' => braces -= 1,
            _ => {}
        }
    }

    parens > 0 || brackets > 0 || braces > 0
}

/// Show a random tip on startup
fn show_startup_tip(ui: &UI) {
    let tips = vec![
        "Use Tab to auto-complete keywords and identifiers",
        "Type :help to see all available commands",
        "Press Ctrl+L to clear the screen",
        "Use :config to customize your REPL experience",
        "Unclosed brackets automatically enable multiline mode",
    ];

    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    let tip = tips[seed % tips.len()];

    ui.tip(tip);
    println!();
}

// Simple heuristic extraction of variable and function names for completion.
fn extract_symbols_for_completion(src: &str, helper: &mut helper::ReplHelper) {
    // naive regex-less scan: look for 'let <ident>' or 'fn <ident>(' patterns
    for line in src.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("let ") {
            let ident: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !ident.is_empty() {
                helper.add_variable(ident);
            }
        }
        if let Some(rest) = trimmed.strip_prefix("fn ") {
            let ident: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !ident.is_empty() {
                helper.add_function(ident);
            }
        }
    }
}
