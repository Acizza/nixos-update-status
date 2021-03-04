# nixos-update-status

This is a simple program to print the current system update status on NixOS. It is primarily meant to be used with toolbars like polybar and awesome.

If your system is fully updated, it will by default print out "synced", and "unsynced ($)" if it's outdated (where the $ represents how many channel bumps have been missed).

If you care about keeping track of every missed channel bump, the program will need to be ran at least once every 4 hours.

# Building

## NixOS Users

There is a package definition [here](https://github.com/Acizza/nixos-config/blob/desktop/overlays/pkgs/nixos-update-status.nix) that can be used to build the program. Alternatively, you can run `nix-shell` and then `cargo build --release` in the project directory to build it manually. The compiled binary will be located in the `target/release/` folder.

## Manually

This project requires the following dependencies:

* A recent stable version of Rust
* pkg-config

In most cases, pkg-config should already be installed. If your distribution does not provide a recent version of Rust, you can obtain the latest version [here](https://rustup.rs/).

Once the dependencies are installed, you can build the project simply by running `cargo build --release` in the project directory. Once compilation is complete, you will find the compiled binary in the `target/release/` folder. None of the other files in that directory need to be kept.

# Usage

The only required argument for the program is which NixOS channel to use. You can see which channel(s) your system is currently following by running `nix-channel --list`, and see all channels by viewing https://status.nixos.org. Only one channel can be followed by the program at a time, and supplying a different one will overwrite the number of missed updates.

For example, the following command would launch the program and track the `nixos-unstable-small` channel:
`nixos-update-status nixos-unstable-small`.

## Custom Messages

You can change what message to display when the system is synced or unsynced with the following flags:

| Flag | Description |
| ---- | ----------- |
| `-s` | The message to display when the system is synced to the desired channel. |
| `-u` | The message to display when the system is out of sync with the desired channel. All instances of `$` will be replaced with the number of missed updates. |

For example:

`nixos-update-status nixos-unstable-small -s "The system is synced!" -u "Out of sync by $ update(s)!"`
