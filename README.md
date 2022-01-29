# tmexclude

Exclude undesired files (node_modules, target, etc) from your TimeMachine backup.

This utility watches your filesystem and excludes the files once they appear, so you won't accidentally include them
in your backups. Full scans can also be performed manually to ensure no file slips through the watcher.

Screenshots available [here](#screenshots).

## Installation

### Homebrew

```shell
$ brew install PhotonQuantum/tap/tmexclude
```

## Usage

### Enable background daemon (recommended)

```shell
$ tmexclude agent start
```

`tmexclude` will start at login and watch for file changes in your home directory.

New files matching a set of rules will be excluded immediately.
By default, rules include dependency directories like `node_modules/` living adjacent to a `package.json` file.

### Perform a full scan

```shell
$ tmexclude scan
```

Home directory will be scanned recursively for files that have missed by the background daemon.

After the scan completes, you have a chance to review the results and decide whether to exclude them or not.

> Refer to `tmexclude help` for more commands and options.

## Configuration

You may customize the rules and directories to scan by editing the config file.

See [`config.example.yaml`](config.example.yaml) for an example configuration file.

The config file is located at `~/.config/tmexclude.yaml`.
A default config is generated when the daemon starts or a full scan is performed if the config file is missing.

You may test your new configuration by running `tmexclude scan -d`, which will perform a dry run.

After you are satisfied with the configuration, you may take it into effect by running `tmexclude reload`.

## Known Issues

### Repeated permission request dialogs

If you include any of your Desktop, Documents or Downloads directories in the scan path and don't skip them,
you will be asked to grant access to these folders.
Unfortunately, macOS won't remember your choice, and will keep popping dialogs.

I have no idea how to let macOS remember the choice, so I added those personal directories to the `skips` list of the
default config. If you don't need the `daemon` mode, feel free to remove them.

It's possible that the annoying dialogs may go away once you grant `tmexclude` full disk access. I've never tested it,
however.

## Screenshots

### Daemon mode
[![daemon](screenshots/daemon.gif)](https://asciinema.org/a/465340)

### Scan mode
[![scan](screenshots/scan.gif)](https://asciinema.org/a/465339)

## License

This project is licensed under [MIT License](LICENSE).