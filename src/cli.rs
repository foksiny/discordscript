use clap::{Parser as ClapParser, Subcommand, Args};

#[derive(ClapParser, Debug)]
#[command(name = "discordscript", version, about = "DiscordScript - Super easy Discord bot language")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new DiscordScript project
    Init {
        /// Project name
        name: Option<String>,
    },

    /// Create a new component (command, module, schedule, event, type)
    New {
        /// Component type: command, module, schedule, event, type
        kind: String,
        /// Component name
        name: String,
    },

    /// Run a DiscordScript file (native interpreter)
    Run {
        /// Path to the main .ds file
        file: String,
        /// Watch for changes and hot-reload
        #[arg(long, short)]
        watch: bool,
    },

    /// Transpile a DiscordScript file to Python
    Build {
        /// Path to the main .ds file
        file: String,
        /// Target platform
        #[arg(long, short, default_value = "discordpy")]
        target: String,
        /// Build for all targets
        #[arg(long, short)]
        all: bool,
        /// Output directory
        #[arg(long, short)]
        out: Option<String>,
        /// Minify generated Python code
        #[arg(long)]
        minify: bool,
    },

    /// Generate documentation in Markdown
    Docs {
        /// Path to the main .ds file
        file: String,
        /// Serve documentation locally
        #[arg(long)]
        serve: bool,
        /// Only document specific commands
        #[arg(long)]
        only: Option<String>,
        /// Documentation language
        #[arg(long, default_value = "pt")]
        lang: String,
        /// Documentation template
        #[arg(long, default_value = "default")]
        template: String,
    },

    /// Run tests defined in the project
    Test {
        /// Path to the main .ds file
        file: String,
        /// Verbose output
        #[arg(long, short)]
        verbose: bool,
    },

    /// Check syntax and semantics without running
    Check {
        /// Path to the main .ds file
        file: String,
        /// Auto-fix issues
        #[arg(long)]
        fix: bool,
        /// Strict checking
        #[arg(long)]
        strict: bool,
    },

    /// Format DiscordScript code
    Fmt {
        /// Path to the .ds file or directory
        path: String,
        /// Only check, don't modify
        #[arg(long)]
        check: bool,
    },

    /// Validate and show environment variables
    Env {
        /// Show variable names (hides values)
        #[arg(long)]
        show: bool,
    },

    /// Initialize a module file in the current project
    Module {
        /// Module name
        name: String,
    },

    /// Generate a command reference card
    Refcard {
        /// Path to the main .ds file
        file: String,
        /// Output format
        #[arg(long, default_value = "md")]
        format: String,
    },
}

#[derive(Args, Debug)]
pub struct BuildArgs {
    pub target: String,
    pub all: bool,
    pub out: Option<String>,
    pub minify: bool,
}
