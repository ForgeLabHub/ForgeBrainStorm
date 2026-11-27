//! Repository automation: `cargo xtask <command>`.
//!
//! Keeping the CI steps in Rust means a contributor runs exactly what CI runs,
//! on any shell, without a Makefile/Bash translation layer.
//!
//! Commands:
//! * `fmt`        — format the whole workspace
//! * `fmt-check`  — fail if anything is unformatted
//! * `clippy`     — `clippy --workspace --all-targets -- -D warnings`
//! * `test`       — `test --workspace --all-targets`
//! * `ci`         — `fmt-check`, `clippy`, `test` in that order
//! * `help`       — this text

use std::process::{Command, ExitCode};

/// A named cargo invocation that `xtask` can run.
struct Step {
    name: &'static str,
    args: &'static [&'static str],
}

const FMT: Step = Step {
    name: "fmt",
    args: &["fmt", "--all"],
};

const FMT_CHECK: Step = Step {
    name: "fmt-check",
    args: &["fmt", "--all", "--", "--check"],
};

const CLIPPY: Step = Step {
    name: "clippy",
    args: &[
        "clippy",
        "--workspace",
        "--all-targets",
        "--locked",
        "--",
        "-D",
        "warnings",
    ],
};

const TEST: Step = Step {
    name: "test",
    args: &["test", "--workspace", "--all-targets", "--locked"],
};

const CI: &[&Step] = &[&FMT_CHECK, &CLIPPY, &TEST];

const HELP: &str = "\
usage: cargo xtask <command>

commands:
  fmt         format the workspace
  fmt-check   verify formatting (what CI runs)
  clippy      clippy over all targets with -D warnings
  test        run every test in the workspace
  ci          fmt-check + clippy + test
  help        show this message
";

fn main() -> ExitCode {
    let command = std::env::args().nth(1).unwrap_or_else(|| "help".into());

    let steps: &[&Step] = match command.as_str() {
        "fmt" => &[&FMT],
        "fmt-check" => &[&FMT_CHECK],
        "clippy" => &[&CLIPPY],
        "test" => &[&TEST],
        "ci" => CI,
        "help" | "--help" | "-h" => {
            print!("{HELP}");
            return ExitCode::SUCCESS;
        }
        other => {
            eprintln!("xtask: unknown command `{other}`\n\n{HELP}");
            return ExitCode::FAILURE;
        }
    };

    for step in steps {
        if !run(step) {
            eprintln!("xtask: `{}` failed", step.name);
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

/// Run one cargo step, streaming its output to the terminal.
fn run(step: &Step) -> bool {
    println!("xtask: cargo {}", step.args.join(" "));
    match Command::new("cargo").args(step.args).status() {
        Ok(status) => status.success(),
        Err(err) => {
            eprintln!("xtask: could not spawn cargo: {err}");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ci_runs_the_three_gates_in_order() {
        let names: Vec<_> = CI.iter().map(|step| step.name).collect();
        assert_eq!(names, ["fmt-check", "clippy", "test"]);
    }

    #[test]
    fn fmt_check_only_reports_and_fmt_rewrites() {
        assert!(FMT_CHECK.args.contains(&"--check"));
        assert!(!FMT.args.contains(&"--check"));
    }

    #[test]
    fn clippy_gates_on_warnings() {
        assert!(CLIPPY
            .args
            .windows(2)
            .any(|pair| pair == ["-D", "warnings"]));
    }
}
