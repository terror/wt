use super::*;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Branch {
  Bare,
  Detached,
  Named(String),
}

impl Display for Branch {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.pad(match self {
      Self::Bare => "(bare)",
      Self::Detached => "(detached)",
      Self::Named(branch) => branch,
    })
  }
}
