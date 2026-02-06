use {super::*, skim::prelude::*};

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

  let worktrees: Vec<_> = str::from_utf8(&output.stdout)?
    .split("\n\n")
    .filter_map(|block| worktree::Worktree::try_from(block).ok())
    .filter(|worktree| Path::new(&worktree.path).is_dir())
    .collect();

  if worktrees.len() < 2 {
    bail!("no worktrees to remove");
  }

  let head_path = worktrees[0].path.clone();

  let items = worktrees
    .into_iter()
    .skip(1)
    .map(|wt| Arc::new(wt) as Arc<dyn SkimItem>)
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

  let selected: Vec<_> = output
    .selected_items
    .iter()
    .map(|item| (item.text().to_string(), item.output().to_string()))
    .collect();

  if selected.is_empty() {
    return Ok(());
  }

  for (branch, path) in &selected {
    let result = Command::new("git")
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

  Ok(())
}
