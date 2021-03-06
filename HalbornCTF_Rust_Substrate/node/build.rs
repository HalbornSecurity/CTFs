include!("src/cli.rs");

use sc_cli::structopt::clap::Shell;
use std::{env, fs, path::Path};
use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

fn main() {
    build_shell_completion();
    generate_cargo_keys();

    rerun_if_git_head_changed();
}

/// Build shell completion scripts for all known shells
/// Full list in https://github.com/kbknapp/clap-rs/blob/e9d0562a1dc5dfe731ed7c767e6cee0af08f0cf9/src/app/parser.rs#L123
fn build_shell_completion() {
    for shell in &[
        Shell::Bash,
        Shell::Fish,
        Shell::Zsh,
        Shell::Elvish,
        Shell::PowerShell,
    ] {
        build_completion(shell);
    }
}

/// Build the shell auto-completion for a given Shell
fn build_completion(shell: &Shell) {
    let outdir = match env::var_os("OUT_DIR") {
        None => return, // undefined, skip
        Some(dir) => dir,
    };
    let path = Path::new(&outdir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("completion-scripts");

    fs::create_dir(&path).ok();

    Cli::clap().gen_completions("malborn-chain", *shell, &path);
}
