use super::*;

fn diff_stat(path: &str) -> (usize, usize) {
  let output = Command::new("git")
    .args(["diff", "--numstat"])
    .current_dir(path)
    .stderr(Stdio::null())
    .output()
    .ok();

  let Some(output) = output.filter(|output| output.status.success()) else {
    return (0, 0);
  };

  let stdout = str::from_utf8(&output.stdout).unwrap_or_default();

  stdout
    .lines()
    .fold((0, 0), |(insertions, deletions), line| {
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
    })
}

pub(crate) fn run() -> Result {
  let style = Style::stdout();

  let current_dir = env::current_dir()?;
  let current_dir = current_dir.canonicalize().unwrap_or(current_dir);

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

  let stats = worktrees
    .iter()
    .map(|w| diff_stat(&w.path))
    .collect::<Vec<_>>();

  let branch_width = worktrees
    .iter()
    .map(|worktree| worktree.branch.len())
    .max()
    .unwrap_or(0);

  for (worktree, (insertions, deletions)) in worktrees.iter().zip(stats.iter())
  {
    let is_current = Path::new(&worktree.path)
      .canonicalize()
      .is_ok_and(|path| current_dir.starts_with(path));

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
      style.apply(style::CYAN, &worktree.head),
      diff,
      worktree.path,
      width = branch_width,
    );
  }

  Ok(())
}
