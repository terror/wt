use {super::*, create::Create};

mod create;
mod list;
mod remove;
mod switch;

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  /// Create a new worktree.
  #[clap(alias = "c")]
  Create(Create),
  /// List all worktrees.
  #[clap(alias = "l")]
  List,
  /// Remove worktrees.
  #[clap(alias = "r")]
  Remove,
  /// Switch to a different worktree.
  #[clap(alias = "s")]
  Switch,
}

impl Subcommand {
  pub(crate) fn run(self) -> Result {
    match self {
      Self::Create(create) => create.run(),
      Self::List => list::run(),
      Self::Remove => remove::run(),
      Self::Switch => switch::run(),
    }
  }
}
