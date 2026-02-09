use super::*;

#[derive(Clone, Debug, clap::ValueEnum)]
enum HookName {
  PostWorktreeChange,
}

#[derive(Debug, Parser)]
pub(crate) struct Hook {
  name: HookName,
}

impl Hook {
  pub(crate) fn run(self) -> Result {
    let config = Config::load()?;

    match self.name {
      HookName::PostWorktreeChange => {
        for entry in &config.hooks.post_worktree_change {
          if entry.matches()? {
            println!("{}", entry.command);
          }
        }
      }
    }

    Ok(())
  }
}
