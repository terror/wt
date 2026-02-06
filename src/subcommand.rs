use {super::*, create::Create, init::Init, switch::Switch};

mod create;
mod init;
mod switch;

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  /// Create a new worktree.
  Create(Create),
  /// Generate shell integration.
  Init(Init),
  Remove,
  /// Switch to a different worktree.
  Switch(Switch),
}

impl Subcommand {
  pub(crate) fn run(self) -> Result {
    match self {
      Self::Create(create) => create.run(),
      Self::Init(init) => {
        init.run();
        Ok(())
      }
      Self::Remove => todo!(),
      Self::Switch(_) => Switch::run(),
    }
  }
}
