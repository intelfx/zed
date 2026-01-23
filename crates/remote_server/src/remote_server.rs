mod headless_project;

#[cfg(not(windows))]
pub mod unix;

#[cfg(test)]
mod remote_editing_tests;

use clap::Subcommand;
use std::path::PathBuf;

pub use headless_project::{HeadlessAppState, HeadlessProject};

#[derive(Subcommand)]
pub enum Commands {
    Run {
        #[arg(long)]
        log_file: PathBuf,
        #[arg(long)]
        pid_file: PathBuf,
        #[arg(long)]
        stdin_socket: PathBuf,
        #[arg(long)]
        stdout_socket: PathBuf,
        #[arg(long)]
        stderr_socket: PathBuf,
    },
    Proxy {
        #[arg(long)]
        reconnect: bool,
        #[arg(long)]
        identifier: String,
    },
    Version {
        #[arg(long, conflicts_with = "binary_name")]
        full: bool,
        #[arg(long, conflicts_with = "full")]
        binary_name: bool,
    },
}

#[cfg(not(windows))]
pub fn run(command: Commands) -> anyhow::Result<()> {
    use anyhow::Context;
    use release_channel::{APP_VERSION, RELEASE_CHANNEL, ReleaseChannel};
    use unix::{ExecuteProxyError, execute_proxy, execute_run};

    match command {
        Commands::Run {
            log_file,
            pid_file,
            stdin_socket,
            stdout_socket,
            stderr_socket,
        } => execute_run(
            log_file,
            pid_file,
            stdin_socket,
            stdout_socket,
            stderr_socket,
        ),
        Commands::Proxy {
            identifier,
            reconnect,
        } => execute_proxy(identifier, reconnect)
            .inspect_err(|err| {
                if let ExecuteProxyError::ServerNotRunning(err) = err {
                    std::process::exit(err.to_exit_code());
                }
            })
            .context("running proxy on the remote server"),
        Commands::Version { full, binary_name } => {
            let release_channel = *RELEASE_CHANNEL;

            // New output format. This matches all other places where AppVersion
            // is used to derive the version string.
            if full || binary_name {
                let app_version = &*APP_VERSION;

                if binary_name {
                    let version_str = match release_channel {
                        ReleaseChannel::Dev => "build".to_string(),
                        _ => app_version.to_string(),
                    };
                    let binary_name = format!(
                        "zed-remote-server-{}-{}{}",
                        release_channel.dev_name(),
                        version_str,
                        if cfg!(target_os = "windows") { ".exe" } else { "" }
                    );
                    println!("{}", binary_name);

                } else {
                    println!("{}", app_version.to_string());
                }
                return Ok(());
            }

            // Old, irregular output format
            match release_channel {
                ReleaseChannel::Stable | ReleaseChannel::Preview => {
                    println!("{}", env!("ZED_PKG_VERSION"))
                }
                ReleaseChannel::Nightly | ReleaseChannel::Dev => {
                    let commit_sha =
                        option_env!("ZED_COMMIT_SHA").unwrap_or(release_channel.dev_name());
                    let build_id = option_env!("ZED_BUILD_ID");
                    if let Some(build_id) = build_id {
                        println!("{}+{}", build_id, commit_sha)
                    } else {
                        println!("{commit_sha}");
                    }
                }
            };
            Ok(())
        }
    }
}
