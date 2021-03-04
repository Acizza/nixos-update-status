use anyhow::{anyhow, Context, Result};
use argh::FromArgs;
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::fs::{self, File};
use std::path::PathBuf;
use std::process::Command;

/// Display missed NixOS channel updates.
#[derive(FromArgs)]
struct Args {
    /// the NixOS channel to retrieve updates from
    #[argh(positional)]
    channel: String,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    match UpdateState::determine_system_state(args.channel) {
        Ok(state) => {
            println!("{}", state);
            Ok(())
        }
        Err(err) => {
            println!("error");
            Err(err)
        }
    }
}

type MissedUpdates = u32;
type Revision = String;

#[derive(Serialize, Deserialize)]
enum UpdateState {
    Synced,
    Unsynced(MissedUpdates, Revision),
}

impl UpdateState {
    const DEFAULT_FILE_NAME: &'static str = "state.mpack";

    fn determine_system_state<S>(channel: S) -> Result<UpdateState>
    where
        S: AsRef<str>,
    {
        let remote_rev =
            remote_system_revision(channel).context("getting latest channel version")?;
        let current_rev = current_system_revision().context("getting current system version")?;

        let is_unsynced = remote_rev != current_rev;

        let mut state = UpdateState::load().unwrap_or_default();

        match &state {
            UpdateState::Synced if is_unsynced => {
                state = UpdateState::Unsynced(1, remote_rev);
                state.save()?;
            }
            UpdateState::Unsynced(missed, last_rev) if is_unsynced && remote_rev != *last_rev => {
                state = UpdateState::Unsynced(missed + 1, remote_rev);
                state.save()?;
            }
            UpdateState::Unsynced(_, _) if !is_unsynced => {
                state = UpdateState::Synced;
                state.save()?;
            }
            UpdateState::Synced | UpdateState::Unsynced(_, _) => (),
        }

        Ok(state)
    }

    fn load() -> Result<UpdateState> {
        let mut path = UpdateState::save_dir();
        path.push(UpdateState::DEFAULT_FILE_NAME);

        let file = File::open(&path)
            .with_context(|| anyhow!("failed to open state file at {}", path.display()))?;

        let state: UpdateState = rmp_serde::from_read(file)
            .with_context(|| anyhow!("failed to decode state file at {}", path.display()))?;

        Ok(state)
    }

    fn save(&self) -> Result<()> {
        let dir = UpdateState::save_dir();

        if !dir.exists() {
            fs::create_dir_all(&dir).with_context(|| {
                anyhow!("failed to create state directory at {}", dir.display())
            })?;
        }

        let mut path = dir;
        path.push(UpdateState::DEFAULT_FILE_NAME);

        let contents = rmp_serde::to_vec(self)?;

        fs::write(&path, contents)
            .with_context(|| anyhow!("failed to write state file to {}", path.display()))?;

        Ok(())
    }

    fn save_dir() -> PathBuf {
        let mut dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("~/.local/share/"));
        dir.push(env!("CARGO_PKG_NAME"));
        dir
    }
}

impl fmt::Display for UpdateState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateState::Synced => write!(f, "synced"),
            UpdateState::Unsynced(missed, _) => write!(f, "unsynced ({})", missed),
        }
    }
}

impl Default for UpdateState {
    fn default() -> UpdateState {
        UpdateState::Synced
    }
}

fn remote_system_revision<S>(channel: S) -> Result<String>
where
    S: AsRef<str>,
{
    use curl::easy::Easy;

    let mut easy = Easy::new();

    easy.url(&format!(
        "https://nixos.org/channels/{}/git-revision",
        channel.as_ref()
    ))
    .and_then(|_| easy.follow_location(true))?;

    let mut buffer = Vec::new();

    {
        let mut transfer = easy.transfer();

        transfer
            .write_function(|data| {
                buffer.extend_from_slice(data);
                Ok(data.len())
            })
            .and_then(|_| transfer.perform())?;
    }

    Ok(String::from_utf8(buffer)?)
}

fn current_system_revision() -> Result<String> {
    let mut cmd = Command::new("nixos-version");
    cmd.arg("--revision");

    let output = cmd
        .output()
        .context("failed to retrieve current system revision with nixos-version command")?;

    let rev = String::from_utf8(output.stdout)?;

    Ok(rev.trim_end().to_string())
}
