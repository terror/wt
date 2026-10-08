use super::*;

#[cfg(unix)]
struct Removal {
  paths: Vec<RemovalPath>,
}

#[cfg(unix)]
impl Removal {
  fn cleanup(mut self) -> Result {
    thread::scope(|scope| {
      self
        .paths
        .iter()
        .map(|path| {
          scope.spawn(move || {
            fs::remove_dir_all(&path.trash_path).map_err(|error| {
              anyhow!(
                "failed to remove worktree directory `{}`: {error}",
                path.trash_path.display(),
              )
            })
          })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .try_for_each(|handle| handle.join().unwrap())
    })?;

    self.paths.clear();

    Ok(())
  }
}

#[cfg(unix)]
impl Drop for Removal {
  fn drop(&mut self) {
    thread::scope(|scope| {
      for path in self.paths.iter().rev() {
        if path.unregistered().unwrap_or(false) {
          scope.spawn(move || {
            let _ = fs::remove_dir_all(&path.trash_path);
          });
        } else if let Err(error) = fs::rename(&path.trash_path, &path.path) {
          eprintln!(
            "failed to restore worktree `{}` from `{}`: {error}",
            path.path.display(),
            path.trash_path.display(),
          );
        }
      }
    });
  }
}

#[cfg(unix)]
struct RemovalPath {
  git_dir: PathBuf,
  path: PathBuf,
  trash_path: PathBuf,
}

#[cfg(unix)]
impl RemovalPath {
  fn new(branch: &str, path: &str, index: usize) -> Result<Self> {
    let output = Command::new("git")
      .current_dir(path)
      .args(["rev-parse", "--absolute-git-dir"])
      .stderr(Stdio::piped())
      .output()?;

    if !output.status.success() {
      bail!(
        "failed to locate worktree `{branch}`: {}",
        str::from_utf8(&output.stderr)?.trim(),
      );
    }

    let git_dir = str::from_utf8(&output.stdout)?;
    let git_dir = PathBuf::from(git_dir.strip_suffix('\n').unwrap_or(git_dir));

    if git_dir.join("locked").try_exists()? {
      bail!("worktree `{branch}` is locked");
    }

    let path = PathBuf::from(path);

    let trash_path = path
      .parent()
      .unwrap_or(&path)
      .join(format!(".wt-removing-{}-{index}", process::id()));

    Ok(Self {
      git_dir,
      path,
      trash_path,
    })
  }

  fn unregistered(&self) -> Result<bool> {
    match fs::symlink_metadata(&self.git_dir) {
      Ok(_) => Ok(false),
      Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(true),
      Err(error) => bail!(
        "failed to inspect worktree registration `{}`: {error}",
        self.git_dir.display(),
      ),
    }
  }
}

#[cfg(unix)]
fn remove_directories(
  selected: &[(String, String)],
  head_path: &str,
) -> Result<Removal> {
  let output = Command::new("git")
    .current_dir(head_path)
    .args(["worktree", "list", "--porcelain", "-z"])
    .stderr(Stdio::piped())
    .output()?;

  if !output.status.success() {
    bail!(
      "failed to list worktrees: {}",
      str::from_utf8(&output.stderr)?.trim(),
    );
  }

  let worktrees = str::from_utf8(&output.stdout)?
    .split("\0\0")
    .filter(|block| !block.is_empty())
    .map(Worktree::try_from)
    .collect::<Result<Vec<_>>>()?;

  for (branch, path) in selected {
    let path = Path::new(path);

    let worktree = worktrees
      .iter()
      .find(|worktree| Path::new(&worktree.path) == path)
      .ok_or_else(|| {
        anyhow!(
          "cannot remove worktree `{branch}`: `{}` is no longer registered",
          path.display(),
        )
      })?;

    if worktree.branch != *branch {
      bail!(
        "cannot remove worktree `{branch}`: `{}` is now on `{}`",
        path.display(),
        worktree.branch,
      );
    }

    for worktree in &worktrees {
      let worktree = Path::new(&worktree.path);

      if worktree != path && worktree.starts_with(path) {
        bail!(
          "cannot remove worktree `{branch}`: contains registered worktree `{}`",
          worktree.display(),
        );
      }
    }
  }

  let paths = selected
    .iter()
    .enumerate()
    .map(|(index, (branch, path))| RemovalPath::new(branch, path, index))
    .collect::<Result<Vec<_>>>()?;

  let mut removal = Removal { paths: Vec::new() };

  for ((branch, path), removal_path) in selected.iter().zip(paths) {
    if fs::rename(&removal_path.path, &removal_path.trash_path).is_ok() {
      removal.paths.push(removal_path);
    } else {
      let result = Command::new("git")
        .current_dir(head_path)
        .args(["worktree", "remove", "--force", path])
        .stderr(Stdio::piped())
        .output()?;

      if !result.status.success() {
        bail!(
          "failed to remove worktree `{}`: {}",
          branch,
          str::from_utf8(&result.stderr)?.trim()
        );
      }
    }
  }

  for path in &removal.paths {
    let output = Command::new("git")
      .current_dir(head_path)
      .args(["worktree", "remove", "--force", "--"])
      .arg(&path.path)
      .stderr(Stdio::piped())
      .output()?;

    if !output.status.success() {
      bail!(
        "failed to remove worktree `{}`: {}",
        path.path.display(),
        str::from_utf8(&output.stderr)?.trim(),
      );
    }

    if !path.unregistered()? {
      bail!(
        "failed to remove worktree `{}`: registration `{}` still exists",
        path.path.display(),
        path.git_dir.display(),
      );
    }
  }

  Ok(removal)
}

#[cfg(unix)]
fn remove_worktrees(
  selected: &[(String, String)],
  head_path: &str,
  current_dir: &Path,
) -> Result {
  let style = Style::stderr();

  let output = Command::new("git")
    .current_dir(head_path)
    .args(["for-each-ref", "--format=%(refname)", "refs/heads/"])
    .output()?;

  if !output.status.success() {
    bail!(
      "failed to list branches: {}",
      str::from_utf8(&output.stderr)?.trim(),
    );
  }

  let branches = str::from_utf8(&output.stdout)?
    .lines()
    .filter_map(|line| line.strip_prefix("refs/heads/"))
    .collect::<Vec<_>>();

  let result =
    remove_directories(selected, head_path).and_then(Removal::cleanup);

  if selected
    .iter()
    .any(|(_, path)| current_dir.starts_with(path))
    && !current_dir.is_dir()
  {
    println!("{head_path}");
  }

  result?;

  for (branch, path) in selected {
    eprintln!(
      "{} worktree {} at {}",
      style.apply(style::GREEN, "removed"),
      style.apply(style::BOLD, branch),
      style.apply(style::CYAN, path),
    );

    if branch != "(detached)" && branches.contains(&branch.as_str()) {
      let result = Command::new("git")
        .current_dir(head_path)
        .args(["branch", "-D", branch])
        .stderr(Stdio::piped())
        .output()?;

      if !result.status.success() {
        bail!(
          "failed to delete branch `{}`: {}",
          branch,
          str::from_utf8(&result.stderr)?.trim()
        );
      }

      eprintln!(
        "{} branch {}",
        style.apply(style::GREEN, "deleted"),
        style.apply(style::BOLD, branch),
      );
    }
  }

  Ok(())
}

#[cfg(not(unix))]
pub(crate) fn run() -> Result {
  bail!("interactive selection is not supported on this platform");
}

#[cfg(unix)]
pub(crate) fn run() -> Result {
  let current_dir = env::current_dir()?;

  let output = Command::new("git")
    .args(["worktree", "list", "--porcelain", "-z"])
    .stderr(Stdio::null())
    .output()?;

  if !output.status.success() {
    bail!("failed to list worktrees");
  }

  let mut worktrees = str::from_utf8(&output.stdout)?
    .split("\0\0")
    .filter_map(|block| Worktree::try_from(block).ok());

  let head_path = worktrees
    .next()
    .ok_or_else(|| anyhow!("no worktrees to remove"))?
    .path;

  let items = worktrees
    .filter(|worktree| !worktree.bare && Path::new(&worktree.path).is_dir())
    .enumerate()
    .map(|(index, worktree)| {
      Arc::new(Worktree { index, ..worktree }) as Arc<dyn SkimItem>
    })
    .collect::<Vec<Arc<dyn SkimItem>>>();

  if items.is_empty() {
    bail!("no worktrees to remove");
  }

  let options = SkimOptionsBuilder::default()
    .multi(true)
    .preview(Some(String::new()))
    .build()?;

  let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

  tx.send(items)?;

  drop(tx);

  let output =
    Skim::run_with(options, Some(rx)).map_err(|error| anyhow!("{error}"))?;

  if output.is_abort {
    return Ok(());
  }

  let selected = output
    .selected_items
    .iter()
    .map(|item| (item.text().to_string(), item.output().to_string()))
    .collect::<Vec<_>>();

  if selected.is_empty() {
    return Ok(());
  }

  remove_worktrees(&selected, &head_path, &current_dir)
}
