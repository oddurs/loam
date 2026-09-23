// loam — a repository's written context, as Markdown under a schema.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.

mod agent;
mod cmd;
mod config;
mod covers;
mod fresh;
mod git;
mod index;
mod reading;
mod scan;
mod style;
mod tree;
mod write;
mod yaml;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

/// Keep a repository's written context — research, design, reference,
/// guides — as Markdown under a schema.
///
/// Exit status: 0 on success; 1 when a check or a search finds what it
/// reports (a failing finding, an out-of-date index, no match); 2 when the
/// command could not run.
#[derive(Parser)]
#[command(name = "loam", version, about, long_about)]
struct Cli {
    /// Run as if loam was started in DIR
    #[arg(short = 'C', long = "directory", global = true, value_name = "DIR")]
    directory: Option<PathBuf>,

    /// Do not run the hooks configured in loam.toml
    #[arg(long, global = true)]
    no_hooks: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Adopt the docs folder here: write loam.toml, change no page
    Init(cmd::init::Args),
    /// Report broken links and anchors, malformed frontmatter, unclaimed pages
    Check(cmd::check::Args),
    /// Write the docs index between its markers
    Render(cmd::render::Args),
    /// Write a new page where its kind lives, from the kind's template
    New(cmd::new::Args),
    /// List pages, by kind and status
    List(cmd::list::Args),
    /// Show one page: what loam read from it, then the page
    Show(cmd::show::Args),
    /// Find what is already written, titles first
    Search(cmd::search::Args),
    /// Move a page or a directory and rewrite every link to it
    Mv(cmd::mv::Args),
    /// Set, change or remove a page's frontmatter keys
    Set(cmd::set::Args),
    /// Record that one page replaces another, on both pages
    Supersede(cmd::supersede::Args),
    /// The pages that matter for files you are about to change, within a budget
    Context(cmd::context::Args),
    /// Print the instructions an agent needs, generated from loam.toml
    Agent(cmd::agent::Args),
    /// Pages whose covered code changed since they were last reviewed
    Stale(cmd::stale::Args),
    /// Record that pages were read against the code as it is now
    Review(cmd::review::Args),
    /// Print the reading of a repository as spec/corpus/README.md shapes it
    #[command(hide = true)]
    Reading { path: Option<PathBuf> },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let ctx = cmd::Ctx {
        cwd: cli.directory.clone().unwrap_or_else(|| PathBuf::from(".")),
        no_hooks: cli.no_hooks,
    };
    let result = match cli.command {
        Command::Init(a) => cmd::init::run(&ctx, a),
        Command::Check(a) => cmd::check::run(&ctx, a),
        Command::Render(a) => cmd::render::run(&ctx, a),
        Command::New(a) => cmd::new::run(&ctx, a),
        Command::List(a) => cmd::list::run(&ctx, a),
        Command::Show(a) => cmd::show::run(&ctx, a),
        Command::Search(a) => cmd::search::run(&ctx, a),
        Command::Mv(a) => cmd::mv::run(&ctx, a),
        Command::Set(a) => cmd::set::run(&ctx, a),
        Command::Supersede(a) => cmd::supersede::run(&ctx, a),
        Command::Stale(a) => cmd::stale::run(&ctx, a),
        Command::Context(a) => cmd::context::run(&ctx, a),
        Command::Agent(a) => cmd::agent::run(&ctx, a),
        Command::Review(a) => cmd::review::run(&ctx, a),
        Command::Reading { path } => {
            // Exit 1 on refusal, as spec/conformance.py expects of any reader.
            let dir = path.unwrap_or(ctx.cwd.clone());
            match config::Config::load(&dir).and_then(tree::Tree::read) {
                Ok(t) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&reading::reading(&t)).expect("JSON")
                    );
                    Ok(0)
                }
                Err(e) => {
                    eprintln!("loam: {e:#}");
                    return ExitCode::from(1);
                }
            }
        }
    };
    match result {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("{}: {e:#}", style::red("loam"));
            ExitCode::from(2)
        }
    }
}
