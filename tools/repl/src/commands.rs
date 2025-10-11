use crate::state::ReplState;
use crate::ui::{Table, Theme, UI};
use anyhow::Result;

/// Handle REPL commands (starting with :)
pub fn handle_command(cmd: &str, state: &mut ReplState, ui: &mut UI) -> Result<bool> {
    let parts: Vec<&str> = cmd.trim().split_whitespace().collect();

    if parts.is_empty() {
        return Ok(true);
    }

    match parts[0] {
        ":help" | ":h" => {
            show_help(ui);
        }
        ":exit" | ":quit" | ":q" => {
            ui.success("Goodbye! 👋");
            return Ok(false);
        }
        ":clear" | ":cls" => {
            clear_screen();
        }
        ":reset" => {
            state.reset();
            ui.success("REPL state has been reset");
        }
        ":history" => {
            show_history(state, ui);
        }
        ":vars" | ":variables" => {
            show_variables(state, ui);
        }
        ":funcs" | ":functions" => {
            show_functions(state, ui);
        }
        ":info" => {
            show_info(state, ui);
        }
        ":config" => {
            if parts.len() > 1 {
                handle_config_command(&parts[1..], state, ui)?;
            } else {
                show_config(state, ui);
            }
        }
        ":theme" => {
            if parts.len() > 1 {
                change_theme(parts[1], state, ui)?;
            } else {
                ui.info("Available themes: default, monokai, dracula, nord, solarized-dark, solarized-light");
            }
        }
        ":save" => {
            if parts.len() > 1 {
                save_session(parts[1], state, ui)?;
            } else {
                ui.error("Usage: :save <filename>");
            }
        }
        ":load" => {
            if parts.len() > 1 {
                load_file(parts[1], state, ui)?;
            } else {
                ui.error("Usage: :load <filename>");
            }
        }
        ":time" => {
            toggle_timing(state, ui);
        }
        ":verbose" => {
            toggle_verbose(state, ui);
        }
        ":multiline" => {
            toggle_multiline(state, ui);
        }
        ":type" => {
            if parts.len() > 1 {
                show_type(&parts[1..].join(" "), state, ui)?;
            } else {
                ui.error("Usage: :type <expression>");
            }
        }
        ":tips" => {
            show_tips(ui);
        }
        ":themes" => {
            ui.section("Available Themes");
            let themes = vec![
                ("default", "Classic terminal colors"),
                ("monokai", "Monokai inspired dark theme"),
                ("dracula", "Dracula theme with vibrant colors"),
                ("nord", "Nord arctic-inspired palette"),
                ("solarized-dark", "Solarized dark theme"),
                ("solarized-light", "Solarized light theme"),
            ];
            for (name, desc) in themes {
                println!("  {} - {}", ui.theme.primary.paint(name), desc);
            }
            println!();
            ui.info("Use ':theme <name>' to switch themes");
        }
        _ => {
            ui.error(&format!("Unknown command: {}", parts[0]));
            ui.info("Type :help for available commands");
        }
    }

    Ok(true)
}

fn show_help(ui: &UI) {
    ui.section("REPL Commands");

    let mut table = Table::new(vec!["Command".to_string(), "Description".to_string()]);

    let commands = vec![
        (":help, :h", "Show this help message"),
        (":exit, :quit, :q", "Exit the REPL"),
        (":clear, :cls", "Clear the screen"),
        (":reset", "Reset the REPL state"),
        (":history", "Show command history"),
        (":vars, :variables", "Show defined variables"),
        (":funcs, :functions", "Show defined functions"),
        (":info", "Show REPL information"),
        (":config [set key value]", "View or modify configuration"),
        (":theme [name]", "Change color theme"),
        (":save <file>", "Save session history to file"),
        (":load <file>", "Load and execute a file"),
        (":time", "Toggle execution timing"),
        (":verbose", "Toggle verbose output"),
        (":multiline", "Toggle multiline mode"),
        (":type <expr>", "Show type of an expression"),
        (":tips", "Show helpful tips"),
        (":themes", "List available color themes"),
    ];

    for (cmd, desc) in commands {
        table.add_row(vec![cmd.to_string(), desc.to_string()]);
    }

    table.print(&ui.theme);

    println!();
    ui.section("Keyboard Shortcuts");
    println!("  Ctrl+C      - Interrupt current input");
    println!("  Ctrl+D      - Exit REPL");
    println!("  Ctrl+L      - Clear screen");
    println!("  Tab         - Auto-complete");
    println!("  ↑/↓         - Navigate history");
    println!();

    ui.tip("Use :tips to see helpful tips for using Veyra");
}

fn show_history(state: &ReplState, ui: &UI) {
    ui.section("Command History");

    let history = state.history();
    if history.is_empty() {
        ui.info("No history yet");
        return;
    }

    for (i, cmd) in history.iter().rev().take(50).enumerate() {
        let num = history.len() - i;
        println!("{:4} │ {}", ui.theme.muted.paint(num.to_string()), cmd);
    }

    if history.len() > 50 {
        println!();
        ui.info(&format!("Showing last 50 of {} entries", history.len()));
    }
}

fn show_variables(state: &ReplState, ui: &UI) {
    ui.section("Defined Variables");

    let vars = state.variables();
    if vars.is_empty() {
        ui.info("No variables defined yet");
        return;
    }

    let mut table = Table::new(vec!["Variable".to_string(), "Type".to_string()]);

    for (name, type_name) in vars {
        table.add_row(vec![name.clone(), type_name.clone()]);
    }

    table.print(&ui.theme);
}

fn show_functions(state: &ReplState, ui: &UI) {
    ui.section("Defined Functions");

    let funcs = state.functions();
    if funcs.is_empty() {
        ui.info("No functions defined yet");
        return;
    }

    for func in funcs {
        println!("  • {}", ui.theme.secondary.paint(func));
    }
}

fn show_info(state: &ReplState, ui: &UI) {
    ui.section("REPL Information");

    let config = state.config();

    println!("  Version:           v0.2.0");
    println!("  History entries:   {}", state.history().len());
    println!("  Variables:         {}", state.variables().len());
    println!("  Functions:         {}", state.functions().len());
    println!(
        "  Syntax highlight:  {}",
        if config.syntax_highlighting {
            "ON"
        } else {
            "OFF"
        }
    );
    println!(
        "  Auto-completion:   {}",
        if config.auto_completion { "ON" } else { "OFF" }
    );
    println!(
        "  Multiline mode:    {}",
        if config.multiline_mode { "ON" } else { "OFF" }
    );
    println!(
        "  Show timing:       {}",
        if config.show_timing { "ON" } else { "OFF" }
    );
    println!(
        "  VI mode:           {}",
        if config.vi_mode { "ON" } else { "OFF" }
    );
    println!(
        "  Auto indent:       {}",
        if config.auto_indent { "ON" } else { "OFF" }
    );
    println!(
        "  Fancy errors:      {}",
        if config.fancy_errors { "ON" } else { "OFF" }
    );
    println!("  Theme:             {:?}", config.color_scheme);

    if let Some(time) = state.last_timing() {
        println!("  Last exec time:    {:.3}ms", time);
    }

    println!();

    // System info
    ui.section("System Information");
    println!("  OS:                {}", std::env::consts::OS);
    println!("  Architecture:      {}", std::env::consts::ARCH);
}

fn show_config(state: &ReplState, ui: &UI) {
    ui.section("Configuration");

    let config = state.config();

    let mut table = Table::new(vec!["Setting".to_string(), "Value".to_string()]);

    table.add_row(vec![
        "syntax_highlighting".to_string(),
        config.syntax_highlighting.to_string(),
    ]);
    table.add_row(vec![
        "auto_completion".to_string(),
        config.auto_completion.to_string(),
    ]);
    table.add_row(vec![
        "multiline_mode".to_string(),
        config.multiline_mode.to_string(),
    ]);
    table.add_row(vec![
        "show_timing".to_string(),
        config.show_timing.to_string(),
    ]);
    table.add_row(vec![
        "max_history".to_string(),
        config.max_history.to_string(),
    ]);
    table.add_row(vec!["vi_mode".to_string(), config.vi_mode.to_string()]);
    table.add_row(vec!["show_tips".to_string(), config.show_tips.to_string()]);
    table.add_row(vec![
        "auto_indent".to_string(),
        config.auto_indent.to_string(),
    ]);
    table.add_row(vec![
        "auto_insert_function_parens".to_string(),
        config.auto_insert_function_parens.to_string(),
    ]);
    table.add_row(vec![
        "fancy_errors".to_string(),
        config.fancy_errors.to_string(),
    ]);

    table.print(&ui.theme);

    println!();
    ui.info("Use ':config set <key> <value>' to change settings");
}

fn handle_config_command(args: &[&str], state: &mut ReplState, ui: &UI) -> Result<()> {
    if args.is_empty() {
        show_config(state, ui);
        return Ok(());
    }

    match args[0] {
        "set" => {
            if args.len() < 3 {
                ui.error("Usage: :config set <key> <value>");
                return Ok(());
            }

            let key = args[1];
            let value = args[2];

            let config = state.config_mut();

            match key {
                "syntax_highlighting" => {
                    config.syntax_highlighting = value.parse().unwrap_or(true);
                    ui.success(&format!("Set syntax_highlighting to {}", value));
                }
                "auto_completion" => {
                    config.auto_completion = value.parse().unwrap_or(true);
                    ui.success(&format!("Set auto_completion to {}", value));
                }
                "multiline_mode" => {
                    config.multiline_mode = value.parse().unwrap_or(true);
                    ui.success(&format!("Set multiline_mode to {}", value));
                }
                "show_timing" => {
                    config.show_timing = value.parse().unwrap_or(true);
                    ui.success(&format!("Set show_timing to {}", value));
                }
                "vi_mode" => {
                    config.vi_mode = value.parse().unwrap_or(false);
                    ui.success(&format!("Set vi_mode to {}", value));
                    ui.warning("Restart REPL for this change to take effect");
                }
                "show_tips" => {
                    config.show_tips = value.parse().unwrap_or(true);
                    ui.success(&format!("Set show_tips to {}", value));
                }
                "auto_indent" => {
                    config.auto_indent = value.parse().unwrap_or(true);
                    ui.success(&format!("Set auto_indent to {}", value));
                }
                "auto_insert_function_parens" => {
                    config.auto_insert_function_parens = value.parse().unwrap_or(true);
                    ui.success(&format!("Set auto_insert_function_parens to {}", value));
                }
                "fancy_errors" => {
                    config.fancy_errors = value.parse().unwrap_or(true);
                    ui.success(&format!("Set fancy_errors to {}", value));
                }
                _ => {
                    ui.error(&format!("Unknown config key: {}", key));
                }
            }

            config.save()?;
        }
        "save" => {
            state.config().save()?;
            ui.success("Configuration saved");
        }
        _ => {
            ui.error(&format!("Unknown config command: {}", args[0]));
        }
    }

    Ok(())
}

fn change_theme(theme_name: &str, state: &mut ReplState, ui: &mut UI) -> Result<()> {
    let new_theme = match theme_name.to_lowercase().as_str() {
        "default" => Some((Theme::default(), crate::config::ColorScheme::Default)),
        "monokai" => Some((Theme::monokai(), crate::config::ColorScheme::Monokai)),
        "dracula" => Some((Theme::dracula(), crate::config::ColorScheme::Dracula)),
        "nord" => Some((Theme::nord(), crate::config::ColorScheme::Nord)),
        "solarized-dark" => Some((
            Theme::solarized_dark(),
            crate::config::ColorScheme::SolarizedDark,
        )),
        "solarized-light" => Some((
            Theme::solarized_light(),
            crate::config::ColorScheme::SolarizedLight,
        )),
        _ => None,
    };

    if let Some((theme, scheme)) = new_theme {
        ui.theme = theme;
        state.config_mut().color_scheme = scheme;
        state.config().save()?;
        ui.success(&format!("Theme changed to '{}'", theme_name));
    } else {
        ui.error(&format!("Unknown theme: '{}'", theme_name));
        ui.info(
            "Available themes: default, monokai, dracula, nord, solarized-dark, solarized-light",
        );
    }
    Ok(())
}

fn save_session(filename: &str, state: &ReplState, ui: &UI) -> Result<()> {
    use crate::ui::with_spinner;

    with_spinner("Saving session...", || {
        let path = std::path::Path::new(filename);
        state.save_history(path)
    })?;

    ui.success(&format!("Session saved to {}", filename));
    Ok(())
}

fn load_file(filename: &str, state: &mut ReplState, ui: &UI) -> Result<()> {
    use crate::ui::with_spinner;

    let path = std::path::Path::new(filename);

    if !path.exists() {
        ui.error(&format!("File not found: {}", filename));
        return Ok(());
    }

    with_spinner(&format!("Loading {}...", filename), || {
        state.load_file(path)
    })?;

    ui.success(&format!("Loaded {}", filename));
    Ok(())
}

fn toggle_timing(state: &mut ReplState, ui: &UI) {
    let config = state.config_mut();
    config.show_timing = !config.show_timing;

    if config.show_timing {
        ui.success("Execution timing enabled");
    } else {
        ui.success("Execution timing disabled");
    }
}

fn toggle_verbose(_state: &mut ReplState, ui: &UI) {
    ui.warning("Verbose mode toggle will be implemented in a future version");
}

fn toggle_multiline(state: &mut ReplState, ui: &UI) {
    let config = state.config_mut();
    config.multiline_mode = !config.multiline_mode;

    if config.multiline_mode {
        ui.success("Multiline mode enabled");
    } else {
        ui.success("Multiline mode disabled");
    }
}

fn show_type(expr: &str, state: &mut ReplState, ui: &UI) -> Result<()> {
    match state.execute(expr) {
        Ok(Some(value)) => {
            let type_name = crate::state::type_name(&value);
            ui.info(&format!("Type: {}", type_name));
        }
        Ok(None) => {
            ui.warning("Expression returned no value");
        }
        Err(e) => {
            ui.error(&format!("Error evaluating expression: {}", e));
        }
    }
    Ok(())
}

fn show_tips(ui: &UI) {
    ui.section("Helpful Tips");

    let tips = vec![
        "Use Tab to auto-complete keywords, variables, and functions",
        "Press Ctrl+C to cancel current input without exiting",
        "Unclosed brackets will automatically enable multiline mode",
        "Use :history to view your previous commands",
        "Save your work with :save <filename> before exiting",
        "Load Veyra files with :load <filename>",
        "Check variable types with :type <expression>",
        "Use :config to customize your REPL experience",
        "Arrow keys navigate through command history",
        "The REPL preserves state between commands",
        "Switch themes dynamically with :theme <name>",
        "Use :themes to see all available color themes",
        "Enable fancy error reports with :config set fancy_errors true",
        "Auto-indentation helps with multiline code blocks",
        "Function completions can auto-insert parentheses",
    ];

    for tip in tips {
        ui.tip(tip);
    }
}

fn clear_screen() {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("cmd")
            .args(&["/C", "cls"])
            .status();
    }

    #[cfg(not(windows))]
    {
        print!("\x1B[2J\x1B[1;1H");
    }
}
