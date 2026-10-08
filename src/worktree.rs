use super::*;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Worktree {
  pub(crate) bare: bool,
  pub(crate) branch: String,
  pub(crate) head: String,
  pub(crate) index: usize,
  pub(crate) path: String,
}

impl TryFrom<&str> for Worktree {
  type Error = Error;

  fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
    let bare = value.split('\0').any(|line| line == "bare");

    let path = value
      .split('\0')
      .find_map(|line| line.strip_prefix("worktree "))
      .ok_or_else(|| anyhow!("missing worktree path"))?
      .to_string();

    let head = value
      .split('\0')
      .find_map(|line| line.strip_prefix("HEAD "))
      .map_or_else(
        || "unknown".to_string(),
        |h| h[..h.len().min(7)].to_string(),
      );

    let branch = value
      .split('\0')
      .find_map(|line| {
        line
          .strip_prefix("branch refs/heads/")
          .map(str::to_string)
          .or_else(|| (line == "detached").then(|| "(detached)".to_string()))
      })
      .or_else(|| bare.then(|| "(bare)".to_string()))
      .ok_or_else(|| anyhow!("missing branch"))?;

    Ok(Worktree {
      bare,
      branch,
      head,
      index: 0,
      path,
    })
  }
}

#[cfg(unix)]
impl SkimItem for Worktree {
  fn display(&self, context: DisplayContext) -> Line<'_> {
    context.to_line(Cow::Owned(format!("{}  {}", self.branch, self.path)))
  }

  fn get_index(&self) -> usize {
    self.index
  }

  fn output(&self) -> Cow<'_, str> {
    Cow::Borrowed(&self.path)
  }

  fn text(&self) -> Cow<'_, str> {
    Cow::Borrowed(&self.branch)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_block_with_branch() {
    assert_eq!(
      Worktree::try_from(
        "worktree /tmp/repo\0HEAD abc123\0branch refs/heads/main\0"
      )
      .unwrap(),
      Worktree {
        bare: false,
        branch: "main".to_string(),
        head: "abc123".to_string(),
        index: 0,
        path: "/tmp/repo".to_string(),
      },
    );
  }

  #[test]
  fn from_block_with_detached() {
    assert_eq!(
      Worktree::try_from("worktree /tmp/repo\0HEAD abc123\0detached\0")
        .unwrap(),
      Worktree {
        bare: false,
        branch: "(detached)".to_string(),
        head: "abc123".to_string(),
        index: 0,
        path: "/tmp/repo".to_string(),
      },
    );
  }

  #[test]
  fn from_block_without_branch() {
    assert!(Worktree::try_from("worktree /tmp/repo\0HEAD abc123\0").is_err());
  }

  #[test]
  fn from_empty_block() {
    assert!(Worktree::try_from("").is_err());
  }
}
