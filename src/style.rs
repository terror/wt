use super::*;

pub(crate) const BOLD: &str = "1";
pub(crate) const CYAN: &str = "36";
pub(crate) const GREEN: &str = "32";

pub(crate) struct Styled<T> {
  code: &'static str,
  enabled: bool,
  value: T,
}

impl<T: Display> Display for Styled<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if !self.enabled {
      return self.value.fmt(f);
    }

    write!(f, "\x1b[{}m", self.code)?;

    self.value.fmt(f)?;

    write!(f, "\x1b[0m")
  }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Style {
  enabled: bool,
}

impl Style {
  pub(crate) fn apply<T: Display>(
    self,
    code: &'static str,
    value: T,
  ) -> Styled<T> {
    Styled {
      code,
      enabled: self.enabled,
      value,
    }
  }

  fn new(enabled: bool) -> Self {
    let mut enabled = enabled;

    if env::var_os("NO_COLOR").is_some() {
      enabled = false;
    }

    if env::var_os("CLICOLOR_FORCE").is_some() {
      enabled = true;
    }

    if let Ok(value) = env::var("CLICOLOR")
      && value == "0"
    {
      enabled = false;
    }

    if let Ok(term) = env::var("TERM")
      && term == "dumb"
    {
      enabled = false;
    }

    Self { enabled }
  }

  pub(crate) fn stderr() -> Self {
    Self::new(io::stderr().is_terminal())
  }

  pub(crate) fn stdout() -> Self {
    Self::new(io::stdout().is_terminal())
  }
}
