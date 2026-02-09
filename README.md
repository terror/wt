## wt

[![build](https://img.shields.io/github/actions/workflow/status/terror/wt/ci.yaml?branch=master&style=flat&labelColor=1d1d1d&color=424242&logo=GitHub%20Actions&logoColor=white&label=build)](https://github.com/terror/wt/actions/workflows/ci.yaml)
[![codecov](https://img.shields.io/codecov/c/gh/terror/wt?style=flat&labelColor=1d1d1d&color=424242&logo=Codecov&logoColor=white)](https://codecov.io/gh/terror/wt)

**wt** is a tool to help you manage your git worktrees.

## Installation

`wt` should run on any unix-based system, including Linux, MacOS, and the BSDs.

The easiest way to install it is by using
[cargo](https://doc.rust-lang.org/cargo/index.html), the Rust package manager:

```bash
cargo install wt-cli
```

Otherwise, see below for the complete package list:

#### Cross-platform

<table>
  <thead>
    <tr>
      <th>Package Manager</th>
      <th>Package</th>
      <th>Command</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><a href=https://www.rust-lang.org>Cargo</a></td>
      <td><a href=https://crates.io/crates/wt-cli>wt-cli</a></td>
      <td><code>cargo install wt-cli</code></td>
    </tr>
    <tr>
      <td><a href=https://brew.sh>Homebrew</a></td>
      <td><a href=https://github.com/terror/homebrew-tap>terror/tap/wt</a></td>
      <td><code>brew install terror/tap/wt</code></td>
    </tr>
  </tbody>
</table>

### Pre-built binaries

Pre-built binaries for Linux, MacOS, and Windows can be found on
[the releases page](https://github.com/terror/wt-cli/releases).

## Usage

`wt` is designed to be very simple to use. We expose a minimal set of
subcommands that let you interact with your worktrees:

```present cargo run -- --help
Usage: wt <COMMAND>

Commands:
  convert  Convert existing branches to worktrees
  create   Create a new worktree
  init     Generate shell integration
  list     List all worktrees
  remove   Remove worktrees
  switch   Switch to a different worktree
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Shell Integration

To enable shell integration, add the following to your `.zshrc`:

```bash
eval "$(wt init zsh)"
```

This defines a `wt` shell function that wraps the `wt` binary. When you run
`convert`, `create`, `remove`, or `switch`, the shell function automatically
`cd`s into the resulting worktree directory and executes any configured hooks.

### Hooks

`wt` supports hooks that run after switching to a worktree. Hooks are
configured in the `wt` config file (typically
`~/.config/wt/config.toml`):

```toml
[[hooks.post_worktree_change]]
command = "direnv reload"

[[hooks.post_worktree_change]]
command = "nvm use"
only_if = ".nvmrc"
```

Each `post_worktree_change` entry has:

- **`command`** — The shell command to run after changing worktrees.
- **`only_if`** *(optional)* — A glob pattern evaluated relative to the
  worktree root. The hook only runs if the pattern matches at least one file.

## Prior Art

I was inspired to build this after using [worktrunk](https://worktrunk.dev/). I
wanted a more minimal set of features with a more powerful fuzzy-search engine
built in.
