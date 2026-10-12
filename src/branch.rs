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

impl FromStr for Branch {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    match s {
      "bare" => Ok(Self::Bare),
      "detached" => Ok(Self::Detached),
      _ => s
        .strip_prefix("branch refs/heads/")
        .map(|branch| Self::Named(branch.to_string()))
        .ok_or_else(|| anyhow!("invalid branch `{s}`")),
    }
  }
}
