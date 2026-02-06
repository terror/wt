use {
  anyhow::{Error, anyhow, bail},
  arguments::Arguments,
  clap::Parser,
  skim::{
    Skim, SkimItem, SkimItemReceiver, SkimItemSender,
    options::SkimOptionsBuilder, prelude::unbounded,
  },
  std::{
    borrow::Cow,
    env,
    fmt::{self, Display, Formatter},
    io::{self, IsTerminal},
    path::Path,
    process::{self, Command, Stdio},
    str,
    sync::Arc,
  },
  style::Style,
  subcommand::Subcommand,
  worktree::Worktree,
};

mod arguments;
mod style;
mod subcommand;
mod worktree;

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn main() {
  if let Err(error) = Arguments::parse().run() {
    eprintln!("error: {error}");

    let causes = error.chain().skip(1).count();

    for (i, err) in error.chain().skip(1).enumerate() {
      eprintln!("       {}─ {err}", if i < causes - 1 { '├' } else { '└' });
    }

    process::exit(1);
  }
}
