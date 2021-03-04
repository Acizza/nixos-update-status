#![warn(clippy::pedantic)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::doc_markdown)]

use anyhow::{anyhow, Context, Result};
use argh::FromArgs;
use nanoserde::{DeBin, SerBin};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::{borrow::Cow, env};

/// Display missed NixOS channel updates.
#[derive(FromArgs)]
struct Args {
    /// the NixOS channel to retrieve updates from
    #[argh(positional)]
    channel: String,

    /// the message to display when the system is synced to the latest channel version
    #[argh(option, short = 's')]
    synced_message: Option<String>,

    /// the message to display when the system is out of sync with the latest channel version.
    /// Use "$" to indicate the number of missed updates
    #[argh(option, short = 'u')]
    unsynced_message: Option<String>,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    match UpdateState::determine_system_state(args.channel) {
        Ok(state) => {
            let msg = match state {
                UpdateState::Synced => args
                    .synced_message
                    .map_or_else(|| "synced".into(), Cow::Owned),
                UpdateState::Unsynced(missed, _) => args
                    .unsynced_message
                    .map_or_else(
                        || format!("unsynced ({})", missed),
                        |msg| msg.replace("$", &missed.to_string()),
                    )
                    .into(),
            };

            println!("{}", msg);
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

#[derive(SerBin, DeBin)]
enum UpdateState {
    Synced,
    Unsynced(MissedUpdates, Revision),
}

impl UpdateState {
    const DEFAULT_FILE_NAME: &'static str = "state.bin";

    fn determine_system_state<S>(channel: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let remote_rev =
            remote_system_revision(channel).context("getting latest channel version")?;
        let current_rev = current_system_revision().context("getting current system version")?;

        let is_unsynced = remote_rev != current_rev;

        let mut state = Self::load().unwrap_or_default();

        match &state {
            Self::Synced if is_unsynced => {
                state = Self::Unsynced(1, remote_rev);
                state.save()?;
            }
            Self::Unsynced(missed, last_rev) if is_unsynced && remote_rev != *last_rev => {
                state = Self::Unsynced(missed + 1, remote_rev);
                state.save()?;
            }
            Self::Unsynced(_, _) if !is_unsynced => {
                state = Self::Synced;
                state.save()?;
            }
            Self::Synced | Self::Unsynced(_, _) => (),
        }

        Ok(state)
    }

    fn load() -> Result<Self> {
        let mut path = Self::save_dir();
        path.push(Self::DEFAULT_FILE_NAME);

        let bytes = fs::read_to_string(&path)
            .with_context(|| anyhow!("failed to read state file at {}", path.display()))?;

        let state = DeBin::deserialize_bin(bytes.as_bytes())
            .with_context(|| anyhow!("failed to decode state file at {}", path.display()))?;

        Ok(state)
    }

    fn save(&self) -> Result<()> {
        let dir = Self::save_dir();

        if !dir.exists() {
            fs::create_dir_all(&dir).with_context(|| {
                anyhow!("failed to create state directory at {}", dir.display())
            })?;
        }

        let mut path = dir;
        path.push(Self::DEFAULT_FILE_NAME);

        let contents = SerBin::serialize_bin(self);

        fs::write(&path, contents)
            .with_context(|| anyhow!("failed to write state file to {}", path.display()))?;

        Ok(())
    }

    fn save_dir() -> PathBuf {
        let mut dir =
            dirs_next::data_local_dir().unwrap_or_else(|| PathBuf::from("~/.local/share/"));

        dir.push(env!("CARGO_PKG_NAME"));
        dir
    }
}

impl Default for UpdateState {
    fn default() -> Self {
        Self::Synced
    }
}

fn remote_system_revision<S>(channel: S) -> Result<String>
where
    S: AsRef<str>,
{
    let url = format!(
        "https://nixos.org/channels/{}/git-revision",
        channel.as_ref()
    );

    let resp = attohttpc::get(url).follow_redirects(true).send()?;

    if !resp.is_success() {
        return Err(anyhow!("bad response: {}", resp.status()));
    }

    resp.text().map_err(Into::into)
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
