use super::*;

#[derive(Clone, Debug, clap::ValueEnum)]
enum Shell {
  Bash,
  Fish,
  Zsh,
}

#[derive(Debug, Parser)]
pub(crate) struct Init {
  #[clap(help = "Shell to generate integration for.")]
  shell: Shell,
}

impl Init {
  pub(crate) fn run(self) {
    match self.shell {
      Shell::Bash | Shell::Zsh => print!("{}", include_str!("init.sh")),
      Shell::Fish => print!("{}", include_str!("init.fish")),
    }
  }
}
