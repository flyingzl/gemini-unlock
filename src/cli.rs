use clap::Parser;

/// Command line arguments definition.
///
/// # Examples
///
/// ```text
/// chrome_gemini --kill-chrome
/// ```
#[derive(Debug, Parser)]
#[command(
    name = "chrome_gemini",
    version,
    about = "Enable Chrome Gemini features by modifying Local State configuration",
    after_help = "Examples:\n  chrome_gemini              # Apply patches (requires Chrome to be closed)\n  chrome_gemini -k            # Close Chrome and apply patches\n  chrome_gemini -r            # Restore from backup\n\nEnvironment Variables:\n  RUST_LOG=info              # Enable info level logging\n  RUST_LOG=debug             # Enable debug level logging"
)]
pub struct Cli {
    /// Close running Chrome before applying patches [short aliases: -k]
    #[arg(long, short = 'k', default_value_t = false)]
    pub kill_chrome: bool,

    /// Restore Local State from backup instead of applying patches [short aliases: -r]
    #[arg(long, short = 'r', default_value_t = false)]
    pub restore: bool,
}
