use super::*;

fn diff_stat(path: &str) -> Result<(usize, usize)> {
  let output = Command::new("git")
    .args(["rev-parse", "--verify", "--quiet", "HEAD"])
    .current_dir(path)
    .output()?;

  let output = match output.status.code() {
    Some(0) => output,
    Some(1) => {
      let output = Command::new("git")
        .args(["hash-object", "-t", "tree", "--stdin"])
        .current_dir(path)
        .stdin(Stdio::null())
        .output()?;

      if !output.status.success() {
        bail!(
          "failed to hash empty tree for worktree `{path}`: {}",
          str::from_utf8(&output.stderr)?.trim(),
        );
      }

      output
    }
    _ => bail!(
      "failed to resolve HEAD for worktree `{path}`: {}",
      str::from_utf8(&output.stderr)?.trim(),
    ),
  };

  let base = str::from_utf8(&output.stdout)?.trim();

  let output = Command::new("git")
    .args(["diff", "--numstat", base, "--"])
    .current_dir(path)
    .output()?;

  if !output.status.success() {
    bail!(
      "failed to diff worktree `{path}`: {}",
      str::from_utf8(&output.stderr)?.trim(),
    );
  }

  let stdout = str::from_utf8(&output.stdout)?;

  Ok(
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
      }),
  )
}

pub(crate) fn run() -> Result {
  let style = Style::stdout();

  let output = Command::new("git")
    .args(["worktree", "list", "--porcelain", "-z"])
    .stderr(Stdio::null())
    .output()?;

  if !output.status.success() {
    bail!("failed to list worktrees");
  }

  let worktrees = str::from_utf8(&output.stdout)?
    .split("\0\0")
    .filter_map(|block| Worktree::try_from(block).ok())
    .filter(|worktree| !worktree.bare && Path::new(&worktree.path).is_dir())
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
    .map(|w| diff_stat(&w.path))
    .collect::<Result<Vec<_>>>()?;

  let branch_width = worktrees
    .iter()
    .map(|worktree| worktree.branch.len())
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
      style.apply(style::CYAN, &worktree.head),
      diff,
      worktree.path,
      width = branch_width,
    );
  }

  Ok(())
}
