use super::*;

pub(crate) fn run() -> Result {
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

  if worktrees.is_empty() {
    bail!("no worktrees found");
  }

  let items = worktrees
    .into_iter()
    .map(|worktree| Arc::new(worktree) as Arc<dyn SkimItem>)
    .collect::<Vec<Arc<dyn SkimItem>>>();

  let options = SkimOptionsBuilder::default()
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

  if let Some(item) = output.selected_items.first() {
    println!("{}", item.output());
  }

  Ok(())
}
