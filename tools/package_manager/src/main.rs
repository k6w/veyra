use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tokio;

#[derive(Parser)]
#[command(name = "veyra-pkg")]
#[command(about = "Package manager for the Veyra programming language")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Veyra project
    Init {
        /// Project name
        name: Option<String>,
        /// Project directory
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Install packages
    Install {
        /// Package name(s) to install
        packages: Vec<String>,
        /// Install as development dependency
        #[arg(long)]
        dev: bool,
        /// Install globally
        #[arg(short, long)]
        global: bool,
    },
    /// Uninstall packages
    Uninstall {
        /// Package name(s) to uninstall
        packages: Vec<String>,
    },
    /// Update packages
    Update {
        /// Specific package to update
        package: Option<String>,
    },
    /// List installed packages
    List,
    /// Search for packages
    Search {
        /// Search query
        query: String,
    },
    /// Show package information
    Info {
        /// Package name
        package: String,
    },
    /// Publish a package
    Publish {
        /// Package directory
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Build the project
    Build {
        /// Release build
        #[arg(short, long)]
        release: bool,
    },
    /// Run the project
    Run {
        /// Arguments to pass to the program
        args: Vec<String>,
    },
    /// Run tests
    Test {
        /// Test filter
        filter: Option<String>,
    },
    /// Clean build artifacts
    Clean,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct VeyraProject {
    name: String,
    version: String,
    description: Option<String>,
    author: Option<String>,
    license: Option<String>,
    main: Option<String>,

    #[serde(default)]
    dependencies: HashMap<String, String>,

    #[serde(default, rename = "dev-dependencies")]
    dev_dependencies: HashMap<String, String>,

    #[serde(default)]
    scripts: HashMap<String, String>,
}

impl Default for VeyraProject {
    fn default() -> Self {
        Self {
            name: "my-veyra-project".to_string(),
            version: "0.1.0".to_string(),
            description: None,
            author: None,
            license: Some("MIT".to_string()),
            main: Some("main.vey".to_string()),
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            scripts: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct _PackageInfo {
    name: String,
    version: String,
    description: Option<String>,
    author: Option<String>,
    license: Option<String>,
    repository: Option<String>,
    keywords: Vec<String>,
    downloads: u64,
    created: String,
    updated: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Registry {
    url: String,
    auth_token: Option<String>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            url: "https://registry.veyra-lang.org".to_string(),
            auth_token: None,
        }
    }
}

struct PackageManager {
    project_dir: PathBuf,
    config_dir: PathBuf,
    _cache_dir: PathBuf,
    _registry: Registry,
    verbose: bool,
}

impl PackageManager {
    fn new(verbose: bool) -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not find config directory"))?
            .join("veyra");

        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow!("Could not find cache directory"))?
            .join("veyra");

        // Create directories if they don't exist
        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(&cache_dir)?;

        let registry = Self::load_registry(&config_dir)?;

        Ok(Self {
            project_dir: std::env::current_dir()?,
            config_dir,
            _cache_dir: cache_dir,
            _registry: registry,
            verbose,
        })
    }

    fn load_registry(config_dir: &Path) -> Result<Registry> {
        let registry_file = config_dir.join("registry.toml");

        if registry_file.exists() {
            let content = fs::read_to_string(&registry_file)?;
            Ok(toml::from_str(&content)?)
        } else {
            let registry = Registry::default();
            let content = toml::to_string_pretty(&registry)?;
            fs::write(&registry_file, content)?;
            Ok(registry)
        }
    }

    fn load_project(&self) -> Result<VeyraProject> {
        let project_file = self.project_dir.join("veyra.toml");

        if !project_file.exists() {
            return Err(anyhow!(
                "No veyra.toml found. Run 'veyra-pkg init' to create a new project."
            ));
        }

        let content = fs::read_to_string(&project_file)?;
        Ok(toml::from_str(&content)?)
    }

    fn save_project(&self, project: &VeyraProject) -> Result<()> {
        let project_file = self.project_dir.join("veyra.toml");
        let content = toml::to_string_pretty(project)?;
        fs::write(&project_file, content)?;
        Ok(())
    }

    async fn init_project(&self, name: Option<String>, path: Option<PathBuf>) -> Result<()> {
        let project_dir = if let Some(path) = path {
            path
        } else {
            self.project_dir.clone()
        };

        let project_name = name.unwrap_or_else(|| {
            project_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("my-veyra-project")
                .to_string()
        });

        // Create project structure
        fs::create_dir_all(&project_dir)?;
        fs::create_dir_all(project_dir.join("src"))?;
        fs::create_dir_all(project_dir.join("tests"))?;
        fs::create_dir_all(project_dir.join("docs"))?;

        // Create veyra.toml
        let mut project = VeyraProject::default();
        project.name = project_name.clone();

        let project_file = project_dir.join("veyra.toml");
        let content = toml::to_string_pretty(&project)?;
        fs::write(&project_file, content)?;

        // Create main.vey
        let main_file = project_dir.join("src").join("main.vey");
        let main_content = format!(
            r#"# Welcome to {}!
# This is your main Veyra source file.

print("Hello, Veyra!")
print("Project: {}")

# Add your code here
fn main() {{
    print("Running main function...")
}}

main()
"#,
            project_name, project_name
        );
        fs::write(&main_file, main_content)?;

        // Create test file
        let test_file = project_dir.join("tests").join("test_main.vey");
        let test_content = r#"# Test file for your Veyra project
import test

test.describe("Main functionality") {
    test.it("should print hello message") {
        # Add your tests here
        test.assert_true(true)
    }
}
"#;
        fs::write(&test_file, test_content)?;

        // Create README
        let readme_file = project_dir.join("README.md");
        let readme_content = format!(
            r#"# {}

A Veyra programming language project.

## Getting Started

```bash
# Run the project
veyra-pkg run

# Build the project
veyra-pkg build

# Run tests
veyra-pkg test
```

## Dependencies

Add dependencies to your `veyra.toml` file:

```toml
[dependencies]
math = "1.0.0"
utils = "0.2.1"
```

Then install them:

```bash
veyra-pkg install
```
"#,
            project_name
        );
        fs::write(&readme_file, readme_content)?;

        // Create .gitignore
        let gitignore_file = project_dir.join(".gitignore");
        let gitignore_content = r#"# Veyra build artifacts
/target/
/build/

# Package manager files
/veyra-modules/
veyra-lock.json

# OS files
.DS_Store
Thumbs.db

# Editor files
*.swp
*.swo
*~
"#;
        fs::write(&gitignore_file, gitignore_content)?;

        println!(
            "{} Created new Veyra project '{}'",
            "✓".green().bold(),
            project_name
        );
        println!("  {} {}", "Directory:".bold(), project_dir.display());
        println!("  {} veyra.toml", "Created:".bold());
        println!("  {} src/main.vey", "Created:".bold());
        println!("  {} tests/test_main.vey", "Created:".bold());
        println!("  {} README.md", "Created:".bold());

        Ok(())
    }

    async fn install_packages(&self, packages: Vec<String>, dev: bool, global: bool) -> Result<()> {
        if packages.is_empty() {
            // Install from veyra.toml
            return self.install_from_project_file().await;
        }

        for package in packages {
            println!("{} Installing {}...", "→".blue().bold(), package);

            if global {
                self.install_package_globally(&package).await?;
            } else {
                self.install_package_locally(&package, dev).await?;
            }

            println!("{} Installed {}", "✓".green().bold(), package);
        }

        Ok(())
    }

    async fn install_from_project_file(&self) -> Result<()> {
        let project = self.load_project()?;

        println!("{} Installing dependencies...", "→".blue().bold());

        for (name, version) in &project.dependencies {
            let package_spec = format!("{}@{}", name, version);
            self.install_package_locally(&package_spec, false).await?;
        }

        for (name, version) in &project.dev_dependencies {
            let package_spec = format!("{}@{}", name, version);
            self.install_package_locally(&package_spec, true).await?;
        }

        println!("{} All dependencies installed", "✓".green().bold());
        Ok(())
    }

    async fn install_package_locally(&self, package: &str, dev: bool) -> Result<()> {
        // Parse package specification (name@version)
        let (name, version) = if let Some(pos) = package.find('@') {
            (&package[..pos], &package[pos + 1..])
        } else {
            (package, "latest")
        };

        // Create veyra-modules directory
        let modules_dir = self.project_dir.join("veyra-modules");
        fs::create_dir_all(&modules_dir)?;

        // Download and extract package
        self.download_package(name, version, &modules_dir).await?;

        // Update project file
        let mut project = self.load_project()?;
        if dev {
            project
                .dev_dependencies
                .insert(name.to_string(), version.to_string());
        } else {
            project
                .dependencies
                .insert(name.to_string(), version.to_string());
        }
        self.save_project(&project)?;

        Ok(())
    }

    async fn install_package_globally(&self, package: &str) -> Result<()> {
        let global_dir = self.config_dir.join("global-packages");
        fs::create_dir_all(&global_dir)?;

        let (name, version) = if let Some(pos) = package.find('@') {
            (&package[..pos], &package[pos + 1..])
        } else {
            (package, "latest")
        };

        self.download_package(name, version, &global_dir).await?;
        Ok(())
    }

    async fn download_package(&self, name: &str, version: &str, target_dir: &Path) -> Result<()> {
        // TODO: Implement actual package download from registry
        // For now, create a placeholder
        let package_dir = target_dir.join(name);
        fs::create_dir_all(&package_dir)?;

        let placeholder_file = package_dir.join("main.vey");
        let placeholder_content = format!(
            r#"# Package: {} v{}
# This is a placeholder package

print("Package {} loaded")
"#,
            name, version, name
        );
        fs::write(&placeholder_file, placeholder_content)?;

        if self.verbose {
            println!("  {} Downloaded {} v{}", "→".blue(), name, version);
        }

        Ok(())
    }

    async fn build_project(&self, release: bool) -> Result<()> {
        let project = self.load_project()?;

        println!(
            "{} Building project '{}'...",
            "→".blue().bold(),
            project.name
        );

        let build_dir = self.project_dir.join("target");
        let build_mode = if release { "release" } else { "debug" };
        let output_dir = build_dir.join(build_mode);

        fs::create_dir_all(&output_dir)?;

        // Find main file
        let main_file = if let Some(main) = &project.main {
            self.project_dir.join("src").join(main)
        } else {
            self.project_dir.join("src").join("main.vey")
        };

        if !main_file.exists() {
            return Err(anyhow!("Main file not found: {}", main_file.display()));
        }

        // TODO: Implement actual compilation
        // For now, just copy the source
        let output_file = output_dir.join(&project.name).with_extension("vey");
        fs::copy(&main_file, &output_file)?;

        println!("{} Built project '{}'", "✓".green().bold(), project.name);
        println!("  {} {}", "Output:".bold(), output_file.display());

        Ok(())
    }

    async fn run_project(&self, args: Vec<String>) -> Result<()> {
        let project = self.load_project()?;

        println!(
            "{} Running project '{}'...",
            "→".blue().bold(),
            project.name
        );

        // Find main file
        let main_file = if let Some(main) = &project.main {
            self.project_dir.join("src").join(main)
        } else {
            self.project_dir.join("src").join("main.vey")
        };

        if !main_file.exists() {
            return Err(anyhow!("Main file not found: {}", main_file.display()));
        }

        // Run with veyra compiler
        let mut cmd = std::process::Command::new("cargo");
        cmd.args(&["run", "--manifest-path"])
            .arg(self.project_dir.join("../compiler/Cargo.toml"))
            .arg("--")
            .arg(&main_file);

        for arg in args {
            cmd.arg(arg);
        }

        let status = cmd.status()?;

        if !status.success() {
            return Err(anyhow!("Project execution failed"));
        }

        Ok(())
    }

    async fn run_tests(&self, filter: Option<String>) -> Result<()> {
        let project = self.load_project()?;

        println!(
            "{} Running tests for '{}'...",
            "→".blue().bold(),
            project.name
        );

        let tests_dir = self.project_dir.join("tests");

        if !tests_dir.exists() {
            println!("{} No tests directory found", "!".yellow().bold());
            return Ok(());
        }

        let mut test_files = Vec::new();

        for entry in fs::read_dir(&tests_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("vey") {
                if let Some(ref filter) = filter {
                    if !path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .map(|s| s.contains(filter))
                        .unwrap_or(false)
                    {
                        continue;
                    }
                }
                test_files.push(path);
            }
        }

        if test_files.is_empty() {
            println!("{} No test files found", "!".yellow().bold());
            return Ok(());
        }

        let mut passed = 0;
        let mut failed = 0;

        for test_file in test_files {
            println!(
                "  {} Running {}...",
                "→".blue(),
                test_file.file_name().unwrap().to_str().unwrap()
            );

            // TODO: Implement actual test runner
            // For now, just run the file
            let mut cmd = std::process::Command::new("cargo");
            cmd.args(&["run", "--manifest-path"])
                .arg(self.project_dir.join("../compiler/Cargo.toml"))
                .arg("--")
                .arg(&test_file);

            let status = cmd.status()?;

            if status.success() {
                passed += 1;
                println!("    {} PASSED", "✓".green().bold());
            } else {
                failed += 1;
                println!("    {} FAILED", "✗".red().bold());
            }
        }

        println!();
        println!(
            "{} {} passed, {} failed",
            "Test results:".bold(),
            passed.to_string().green().bold(),
            failed.to_string().red().bold()
        );

        if failed > 0 {
            std::process::exit(1);
        }

        Ok(())
    }

    async fn list_packages(&self) -> Result<()> {
        println!("{}", "Installed packages:".bold());

        // List local packages
        let modules_dir = self.project_dir.join("veyra-modules");
        if modules_dir.exists() {
            println!("\n{}", "Local packages:".yellow().bold());
            for entry in fs::read_dir(&modules_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    println!("  {}", entry.file_name().to_str().unwrap());
                }
            }
        }

        // List global packages
        let global_dir = self.config_dir.join("global-packages");
        if global_dir.exists() {
            println!("\n{}", "Global packages:".blue().bold());
            for entry in fs::read_dir(&global_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    println!("  {}", entry.file_name().to_str().unwrap());
                }
            }
        }

        Ok(())
    }

    async fn clean(&self) -> Result<()> {
        let target_dir = self.project_dir.join("target");

        if target_dir.exists() {
            fs::remove_dir_all(&target_dir)?;
            println!("{} Cleaned build artifacts", "✓".green().bold());
        } else {
            println!("{} Nothing to clean", "!".yellow().bold());
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let package_manager = PackageManager::new(cli.verbose)?;

    match cli.command {
        Commands::Init { name, path } => {
            package_manager.init_project(name, path).await?;
        }
        Commands::Install {
            packages,
            dev,
            global,
        } => {
            package_manager
                .install_packages(packages, dev, global)
                .await?;
        }
        Commands::Build { release } => {
            package_manager.build_project(release).await?;
        }
        Commands::Run { args } => {
            package_manager.run_project(args).await?;
        }
        Commands::Test { filter } => {
            package_manager.run_tests(filter).await?;
        }
        Commands::List => {
            package_manager.list_packages().await?;
        }
        Commands::Clean => {
            package_manager.clean().await?;
        }
        Commands::Uninstall { packages: _ } => {
            println!("{} Uninstall not yet implemented", "!".yellow().bold());
        }
        Commands::Update { package: _ } => {
            println!("{} Update not yet implemented", "!".yellow().bold());
        }
        Commands::Search { query: _ } => {
            println!("{} Search not yet implemented", "!".yellow().bold());
        }
        Commands::Info { package: _ } => {
            println!("{} Info not yet implemented", "!".yellow().bold());
        }
        Commands::Publish { path: _ } => {
            println!("{} Publish not yet implemented", "!".yellow().bold());
        }
    }

    Ok(())
}
