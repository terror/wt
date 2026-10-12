use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Create {
  /// Base revision for the new branch.
  #[clap(long)]
  from: Option<String>,
  /// Branch name for the new worktree.
  name: String,
}

impl Create {
  pub(crate) fn run(self) -> Result {
    let style = Style::stdout();

    let worktrees = Worktree::list(".")?;

    let head_path = worktrees
      .first()
      .map(|worktree| Path::new(&worktree.path))
      .ok_or_else(|| anyhow!("no worktrees found"))?;

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

    match worktree.symlink_metadata() {
      Ok(_) => bail!("worktree path `{}` already exists", worktree.display()),
      Err(error) if error.kind() == io::ErrorKind::NotFound => {}
      Err(error) => return Err(error.into()),
    }

    if worktrees
      .iter()
      .any(|registered| Path::new(&registered.path) == worktree)
    {
      bail!(
        "worktree path `{}` is already registered",
        worktree.display()
      );
    }

    let branch = Command::new("git")
      .args([
        "show-ref",
        "--verify",
        "--quiet",
        &format!("refs/heads/{}", self.name),
      ])
      .output()?;

    let exists = match branch.status.code() {
      Some(0) => true,
      Some(1) => false,
      _ => bail!(
        "failed to check branch `{}`: {}",
        self.name,
        str::from_utf8(&branch.stderr)?.trim()
      ),
    };

    if exists && self.from.is_some() {
      bail!("cannot use `--from` with existing branch `{}`", self.name);
    }

    let mut command = Command::new("git");

    command.args(["worktree", "add"]);

    if !exists {
      command.args(["-b", &self.name]);
    }

    let output = command
      .arg("--")
      .arg(&worktree)
      .args(self.from.as_ref().or(exists.then_some(&self.name)))
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
