use super::*;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Worktree {
  pub(crate) branch: String,
  pub(crate) path: String,
}

impl TryFrom<&str> for Worktree {
  type Error = Error;

  fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
    let path = value
      .lines()
      .find_map(|line| line.strip_prefix("worktree "))
      .ok_or_else(|| anyhow!("missing worktree path"))?
      .to_string();

    let branch = value
      .lines()
      .find_map(|line| {
        line
          .strip_prefix("branch refs/heads/")
          .map(str::to_string)
          .or_else(|| (line == "detached").then(|| "(detached)".to_string()))
      })
      .ok_or_else(|| anyhow!("missing branch"))?;

    Ok(Worktree { branch, path })
  }
}

impl SkimItem for Worktree {
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
        "worktree /tmp/repo\nHEAD abc123\nbranch refs/heads/main\n"
      )
      .unwrap(),
      Worktree {
        branch: "main".to_string(),
        path: "/tmp/repo".to_string(),
      },
    );
  }

  #[test]
  fn from_block_with_detached() {
    assert_eq!(
      Worktree::try_from("worktree /tmp/repo\nHEAD abc123\ndetached\n")
        .unwrap(),
      Worktree {
        branch: "(detached)".to_string(),
        path: "/tmp/repo".to_string(),
      },
    );
  }

  #[test]
  fn from_block_without_branch() {
    assert!(Worktree::try_from("worktree /tmp/repo\nHEAD abc123\n").is_err());
  }

  #[test]
  fn from_empty_block() {
    assert!(Worktree::try_from("").is_err());
  }
}
