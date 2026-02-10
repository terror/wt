use super::*;

#[cfg(unix)]
fn remove_directories(
  selected: &[(String, String)],
  head_path: &str,
) -> Result<Vec<PathBuf>> {
  let mut pending = Vec::new();

  for (i, (branch, path)) in selected.iter().enumerate() {
    let worktree_path = Path::new(path);

    let trash_path = worktree_path
      .parent()
      .unwrap_or(worktree_path)
      .join(format!(".wt-removing-{}-{i}", process::id()));

    if fs::rename(worktree_path, &trash_path).is_ok() {
      pending.push(trash_path);
    } else {
      let result = Command::new("git")
        .args(["worktree", "remove", "--force", "--force", path])
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

  if !pending.is_empty() {
    let prune = Command::new("git")
      .current_dir(head_path)
      .args(["worktree", "prune"])
      .stderr(Stdio::piped())
      .output()?;

    if !prune.status.success() {
      bail!(
        "failed to prune worktrees: {}",
        str::from_utf8(&prune.stderr)?.trim()
      );
    }
  }

  Ok(pending)
}

#[cfg(not(unix))]
pub(crate) fn run() -> Result {
  bail!("interactive selection is not supported on this platform");
}

#[cfg(unix)]
pub(crate) fn run() -> Result {
  let current_dir = env::current_dir()?;

  let style = Style::stderr();

  let output = Command::new("git")
    .args(["worktree", "list", "--porcelain"])
    .stderr(Stdio::null())
    .output()?;

  if !output.status.success() {
    bail!("failed to list worktrees");
  }

  let worktrees = str::from_utf8(&output.stdout)?
    .split("\n\n")
    .filter_map(|block| Worktree::try_from(block).ok())
    .filter(|worktree| Path::new(&worktree.path).is_dir())
    .collect::<Vec<_>>();

  if worktrees.len() < 2 {
    bail!("no worktrees to remove");
  }

  let head_path = worktrees[0].path.clone();

  let items = worktrees
    .into_iter()
    .skip(1)
    .map(|worktree| Arc::new(worktree) as Arc<dyn SkimItem>)
    .collect::<Vec<Arc<dyn SkimItem>>>();

  let options = SkimOptionsBuilder::default()
    .multi(true)
    .preview(Some("git -C {} diff --color=always".to_string()))
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

  let pending_deletes = remove_directories(&selected, &head_path)?;

  for (branch, path) in &selected {
    eprintln!(
      "{} worktree {} at {}",
      style.apply(style::GREEN, "removed"),
      style.apply(style::BOLD, branch),
      style.apply(style::CYAN, path),
    );

    if branch != "(detached)" {
      let result = Command::new("git")
        .current_dir(&head_path)
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

  if selected
    .iter()
    .any(|(_, path)| current_dir.starts_with(path))
  {
    println!("{head_path}");
  }

  std::thread::scope(|scope| {
    for path in &pending_deletes {
      scope.spawn(move || {
        let _ = fs::remove_dir_all(path);
      });
    }
  });

  Ok(())
}
