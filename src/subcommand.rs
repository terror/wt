use {super::*, create::Create, init::Init};

mod create;
mod init;
mod remove;
mod switch;

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  /// Create a new worktree.
  Create(Create),
  /// Generate shell integration.
  Init(Init),
  /// Remove worktrees.
  Remove,
  /// Switch to a different worktree.
  Switch,
}

impl Subcommand {
  pub(crate) fn run(self) -> Result {
    match self {
      Self::Create(create) => create.run(),
      Self::Init(init) => {
        init.run();
        Ok(())
      }
      Self::Remove => remove::run(),
      Self::Switch => switch::run(),
    }
  }
}
