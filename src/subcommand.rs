use {super::*, create::Create};

mod create;

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  /// Create a new worktree and branch.
  Create(Create),
  Remove,
  Switch,
}

impl Subcommand {
  pub(crate) fn run(self) -> Result {
    match self {
      Self::Create(create) => create.run(),
      Self::Remove => todo!(),
      Self::Switch => todo!(),
    }
  }
}
