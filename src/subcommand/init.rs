use super::*;

#[derive(Clone, Debug, clap::ValueEnum)]
enum Shell {
  Zsh,
}

#[derive(Debug, Parser)]
pub(crate) struct Init {
  /// Shell to generate integration for.
  shell: Shell,
}

impl Init {
  pub(crate) fn run(self) {
    match self.shell {
      Shell::Zsh => print!("{}", include_str!("init.zsh")),
    }
  }
}
