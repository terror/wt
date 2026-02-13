use {super::*, std::path::PathBuf};

#[derive(Debug, Parser)]
pub(crate) struct Create {
  /// Branch name for the new worktree.
  name: String,
}

impl Create {
  pub(crate) fn run(self) -> Result {
    let style = Style::stdout();

    let root = Command::new("git")
      .args(["rev-parse", "--show-toplevel"])
      .stderr(Stdio::null())
      .output()?;

    let root = Path::new(str::from_utf8(&root.stdout)?.trim());

    let head_path = Command::new("git")
      .args(["worktree", "list", "--porcelain"])
      .stderr(Stdio::null())
      .output()
      .ok()
      .filter(|output| output.status.success())
      .and_then(|output| {
        str::from_utf8(&output.stdout)
          .ok()
          .and_then(|stdout| stdout.split("\n\n").next())
          .and_then(|block| Worktree::try_from(block).ok())
          .map(|worktree| PathBuf::from(worktree.path))
      })
      .unwrap_or_else(|| root.to_path_buf());

    let project = head_path.file_name().ok_or_else(|| {
      anyhow!("failed to get project name from `{}`", head_path.display())
    })?;

    let dir_name = format!(
      "{}.{}",
      project.to_string_lossy(),
      self.name.replace('/', "-")
    );

    let worktree = head_path
      .parent()
      .ok_or_else(|| {
        anyhow!(
          "repo root `{}` has no parent directory",
          head_path.display()
        )
      })?
      .join(&dir_name);

    let output = Command::new("git")
      .args([
        "worktree",
        "add",
        "-b",
        &self.name,
        &worktree.to_string_lossy(),
      ])
      .stdout(Stdio::null())
      .stderr(Stdio::piped())
      .output()?;

    if !output.status.success() {
      bail!(
        "failed to create worktree `{}`: {}",
        self.name,
        str::from_utf8(&output.stderr)?.trim()
      );
    }

    eprintln!(
      "{} worktree {} at {}",
      style.apply(style::GREEN, "created"),
      style.apply(style::BOLD, &self.name),
      style.apply(style::CYAN, &dir_name),
    );

    println!("{}", worktree.display());

    Ok(())
  }
}
