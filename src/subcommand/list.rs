use super::*;

pub(crate) fn run() -> Result {
  let style = Style::stdout();

  let current_dir = env::current_dir()?;

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

  let branch_width =
    worktrees.iter().map(|w| w.branch.len()).max().unwrap_or(0);

  for worktree in &worktrees {
    let is_current = current_dir.starts_with(&worktree.path);

    let marker = if is_current {
      style.apply(style::GREEN, "*")
    } else {
      style.apply(style::GREEN, " ")
    };

    println!(
      "{} {:<width$}  {}  {}",
      marker,
      style.apply(style::BOLD, &worktree.branch),
      style.apply(style::CYAN, &worktree.head),
      worktree.path,
      width = branch_width,
    );
  }

  Ok(())
}
