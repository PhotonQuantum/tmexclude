# tmexclude

> WIP

Exclude undesired files (node_modules, target, etc) from your TimeMachine backup.

This utility watches your filesystem and excludes the files once they appear, so you won't accidentally include them
in your backups. Full scans are also performed periodically to ensure no file is slipped through the watcher.

You may configure *tmexclude* with an expressive config file to tailor to your needs.

## License

This project is licensed under [MIT License](LICENSE).