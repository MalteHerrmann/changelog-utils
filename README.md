# Changelog Utils

This tool contains utilities to handle your changelogs for the better.
Its linter can be used to enforce a standard for your changelogs and the CLI can be used to add, update or delete entries.

It was designed to be used as a constant companion for all of your projects. When creating a new project, `clu init` shall be the second thing to execute - right after `git init`.

## Installation

To install the application from source, run

```bash
cargo install --git https://github.com/MalteHerrmann/changelog-utils
```

The application is also available to be used with a Docker image.
It can be built locally by executing `make docker-build`
or downloaded from the [GitHub container registry](https://github.com/MalteHerrmann/changelog-utils/pkgs/container/changelog-utils)
by running

```bash
docker pull ghcr.io/malteherrmann/changelog-utils:[TAG]
```

## Usage

The available subcommands can be listed when running

```bash
 $ clu --help
Usage: clu <COMMAND>

Commands:
  add      Adds a new entry to the unreleased section of the changelog
  fix      Applies all possible auto-fixes to the changelog
  lint     Checks if the changelog contents adhere to the defined rules
  init     Initializes the changelog configuration in the current directory
  config   Adjust the changelog configuration like allowed categories, change types or other
  release  Turns the Unreleased section into a new release with the given version
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Getting Started

This application is designed to be one of the first things you use within your projects' folder -
right after creating the folder and running `git init`.
By executing `clu init`, an empty skeleton for the changelog is generated (`CHANGELOG.md`)
as well as a default configuration (`.clconfig.json`).

Note, that a pre-exixisting changelog will not be overwritten, so you can also run this command
in existing projects. In that case, it will only create the default configuration.

## Configuration

You can add or remove configurations as you like with the
corresponding subcommands of `clu config`.

```bash
Usage: clu config <COMMAND>

Commands:
  category        Adjust the allowed categories for changelog entries
  change-type     Adjust the allowed change types within releases (like 'Bug Fixes', 'Features', etc.)
  legacy-version  Set or unset the optional legacy version
  show            Shows the current configuration
  spelling        Adjust the expected spellings that should be enforced in the changelog
  target-repo     Sets the target repository for the changelog entries
  help            Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
