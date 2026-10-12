use super::*;

pub(crate) fn run() -> Result {
  let style = Style::stdout();

  let worktrees = Worktree::list(".")?
    .into_iter()
    .filter(|worktree| {
      worktree.branch != Branch::Bare && Path::new(&worktree.path).is_dir()
    })
    .collect::<Vec<_>>();

  if worktrees.is_empty() {
    bail!("no worktrees found");
  }

  let root = Command::new("git")
    .args(["rev-parse", "--show-toplevel"])
    .stderr(Stdio::null())
    .output()?;

  let root = if root.status.success() {
    let root = str::from_utf8(&root.stdout)?;

    Path::new(root.strip_suffix('\n').unwrap_or(root))
      .canonicalize()
      .ok()
  } else {
    None
  };

  let stats = worktrees
    .iter()
    .map(Worktree::diff_stat)
    .collect::<Result<Vec<_>>>()?;

  let branch_width = worktrees
    .iter()
    .map(|worktree| worktree.branch.to_string().len())
    .max()
    .unwrap_or(0);

  for (worktree, (insertions, deletions)) in worktrees.iter().zip(stats.iter())
  {
    let is_current = Path::new(&worktree.path)
      .canonicalize()
      .is_ok_and(|path| root.as_ref() == Some(&path));

    let marker = if is_current {
      style.apply(style::GREEN, "*")
    } else {
      style.apply(style::GREEN, " ")
    };

    let diff = format!(
      "{}/{}",
      style.apply(style::GREEN, format_args!("+{insertions}")),
      style.apply(style::RED, format_args!("-{deletions}")),
    );

    println!(
      "{} {:<width$}  {}  {}  {}",
      marker,
      style.apply(style::BOLD, &worktree.branch),
      style.apply(
        style::CYAN,
        format_args!("{:.7}", worktree.head.as_deref().unwrap_or("unknown")),
      ),
      diff,
      worktree.path,
      width = branch_width,
    );
  }

  Ok(())
}
