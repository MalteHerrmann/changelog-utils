# Changelog Utils

This tool contains utilities to handle your changelogs for the better.
Its linter can be used to enforce a standard for your changelogs and the CLI can be used to add, update or delete entries.

## Usage

The available subcommands can be listed when running

```bash
 $ clu --help
Usage: clu <COMMAND>

Commands:
  lint  Lint the changelog contents
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

```

## Configuration

The application looks for a `.clconfig.json` file
in the current directory, where it is executed.
This configuration file is created when running

```bash
clu init
```

