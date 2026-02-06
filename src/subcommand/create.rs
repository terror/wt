use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Create {
  /// Branch name for the new worktree.
  name: String,
}

impl Create {
  pub(crate) fn run(self) -> Result {
    let root = Command::new("git")
      .args(["rev-parse", "--show-toplevel"])
      .output()?;

    let root = Path::new(str::from_utf8(&root.stdout)?.trim());

    let project = root.file_name().ok_or_else(|| {
      anyhow!("failed to get project name from `{}`", root.display())
    })?;

    let worktree = root
      .parent()
      .ok_or_else(|| {
        anyhow!("repo root `{}` has no parent directory", root.display())
      })?
      .join(format!(
        "{}.{}",
        project.to_string_lossy(),
        self.name.replace('/', "-")
      ));

    let status = Command::new("git")
      .args([
        "worktree",
        "add",
        "-b",
        &self.name,
        &worktree.to_string_lossy(),
      ])
      .status()?;

    if !status.success() {
      bail!("failed to create worktree `{}`", self.name);
    }

    Ok(())
  }
}
