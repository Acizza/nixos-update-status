use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::fs::{self, File};
use std::path::PathBuf;
use std::process::Command;

fn main() {
    match UpdateState::determine_system_state() {
        Some(state) => println!("{}", state),
        None => println!("error"),
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

    fn determine_system_state() -> Option<UpdateState> {
        let remote_rev = remote_system_revision()?;
        let current_rev = current_system_revision()?;
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

        Some(state)
    }

    fn load() -> Option<UpdateState> {
        let mut path = UpdateState::save_dir()?;
        path.push(UpdateState::DEFAULT_FILE_NAME);

        let file = File::open(path).ok()?;
        let state: UpdateState = rmp_serde::from_read(file).ok()?;

        Some(state)
    }

    fn save(&self) -> Option<()> {
        let dir = UpdateState::save_dir()?;

        if !dir.exists() {
            fs::create_dir_all(&dir).ok()?;
        }

        let mut path = dir;
        path.push(UpdateState::DEFAULT_FILE_NAME);

        let contents = rmp_serde::to_vec(self).ok()?;
        fs::write(path, contents).ok()?;

        Some(())
    }

    fn save_dir() -> Option<PathBuf> {
        let mut dir = dirs::data_local_dir()?;
        dir.push(env!("CARGO_PKG_NAME"));
        Some(dir)
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

// TODO: allow specifying of channel
fn remote_system_revision() -> Option<String> {
    use curl::easy::Easy;

    let mut easy = Easy::new();
    easy.url("https://nixos.org/channels/nixos-unstable-small/git-revision")
        .ok()
        .and_then(|_| easy.follow_location(true).ok())?;

    let mut buffer = Vec::new();

    {
        let mut transfer = easy.transfer();

        transfer
            .write_function(|data| {
                buffer.extend_from_slice(data);
                Ok(data.len())
            })
            .ok()?;
        transfer.perform().ok()?;
    }

    String::from_utf8(buffer).ok()
}

fn current_system_revision() -> Option<String> {
    let mut cmd = Command::new("nixos-version");
    cmd.arg("--revision");

    let output = cmd.output().ok()?;
    let rev = String::from_utf8(output.stdout).ok()?;

    Some(rev.trim_end().to_string())
}
