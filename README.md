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
  create  Create a new worktree
  init    Generate shell integration
  list    List all worktrees
  remove  Remove worktrees
  switch  Switch to a different worktree
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Prior Art

I was inspired to build this after using [worktrunk](https://worktrunk.dev/). I
wanted a more minimal set of features with a more powerful fuzzy-search engine
built in.
