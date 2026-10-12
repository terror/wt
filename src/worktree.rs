use super::*;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Worktree {
  pub(crate) branch: Branch,
  pub(crate) head: Option<String>,
  pub(crate) index: usize,
  pub(crate) path: String,
}

impl Worktree {
  pub(crate) fn diff_base(&self) -> Result<String> {
    let path = &self.path;

    let output = Command::new("git")
      .args(["rev-parse", "--verify", "--quiet", "HEAD"])
      .current_dir(path)
      .output()?;

    let output = match output.status.code() {
      Some(0) => output,
      Some(1) => {
        let output = Command::new("git")
          .args(["hash-object", "-t", "tree", "--stdin"])
          .current_dir(path)
          .stdin(Stdio::null())
          .output()?;

        if !output.status.success() {
          bail!(
            "failed to hash empty tree for worktree `{path}`: {}",
            str::from_utf8(&output.stderr)?.trim(),
          );
        }

        output
      }
      _ => bail!(
        "failed to resolve HEAD for worktree `{path}`: {}",
        str::from_utf8(&output.stderr)?.trim(),
      ),
    };

    Ok(str::from_utf8(&output.stdout)?.trim().to_string())
  }

  pub(crate) fn diff_stat(&self) -> Result<(usize, usize)> {
    let path = &self.path;

    let output = Command::new("git")
      .args(["diff", "--numstat", &self.diff_base()?, "--"])
      .current_dir(path)
      .output()?;

    if !output.status.success() {
      bail!(
        "failed to diff worktree `{path}`: {}",
        str::from_utf8(&output.stderr)?.trim(),
      );
    }

    Ok(str::from_utf8(&output.stdout)?.lines().fold(
      (0, 0),
      |(insertions, deletions), line| {
        let mut parts = line.split('\t');

        let added = parts
          .next()
          .and_then(|part| part.parse::<usize>().ok())
          .unwrap_or(0);

        let removed = parts
          .next()
          .and_then(|part| part.parse::<usize>().ok())
          .unwrap_or(0);

        (insertions + added, deletions + removed)
      },
    ))
  }

  pub(crate) fn list(directory: impl AsRef<Path>) -> Result<Vec<Self>> {
    let output = Command::new("git")
      .current_dir(directory)
      .args(["worktree", "list", "--porcelain", "-z"])
      .stderr(Stdio::piped())
      .output()?;

    if !output.status.success() {
      bail!(
        "failed to list worktrees: {}",
        str::from_utf8(&output.stderr)?.trim(),
      );
    }

    str::from_utf8(&output.stdout)?
      .split("\0\0")
      .filter(|block| !block.is_empty())
      .map(Self::try_from)
      .collect()
  }
}

impl TryFrom<&str> for Worktree {
  type Error = Error;

  fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
    let path = value
      .split('\0')
      .find_map(|line| line.strip_prefix("worktree "))
      .ok_or_else(|| anyhow!("missing worktree path"))?
      .to_string();

    let head = value
      .split('\0')
      .find_map(|line| line.strip_prefix("HEAD "))
      .map(str::to_string);

    let branch = value
      .split('\0')
      .find_map(|line| line.parse::<Branch>().ok())
      .ok_or_else(|| anyhow!("missing branch"))?;

    Ok(Worktree {
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

  fn preview(&self, _context: PreviewContext) -> ItemPreview {
    match self.diff_base() {
      Ok(base) => ItemPreview::Command(format!(
        "git -C {{}} diff --color=always {base} --"
      )),
      Err(error) => ItemPreview::Text(format!("error: {error}")),
    }
  }

  fn text(&self) -> Cow<'_, str> {
    Cow::Owned(self.branch.to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_block_with_branch() {
    assert_eq!(
      Worktree::try_from(
        "worktree /tmp/repo\0HEAD abc123def456\0branch refs/heads/main\0"
      )
      .unwrap(),
      Worktree {
        branch: Branch::Named("main".to_string()),
        head: Some("abc123def456".to_string()),
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
        branch: Branch::Detached,
        head: Some("abc123".to_string()),
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
