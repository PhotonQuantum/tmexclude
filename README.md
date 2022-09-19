# TimeMachine Exclude (tmexclude)

Exclude undesired files (node_modules, target, etc) from your TimeMachine backup.

This utility watches your filesystem and excludes the files once they appear, so you won't accidentally include them
in your backups. Full scans can also be performed manually to ensure no file slips through the watcher.

Screenshots available [here](#screenshots).

*If you find this utility useful, please consider [buy me a coffee](https://buymeacoffee.com/lightquantum).*

## Installation

### Homebrew

```shell
$ brew install PhotonQuantum/tap/tmexclude
```

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

> WIP

## Acknowledgements

<a href="https://www.flaticon.com/free-icons/harddisk" title="harddisk icons">Harddisk icons created by Smashicons - Flaticon</a>

## License

This project is licensed under [MIT License](LICENSE).