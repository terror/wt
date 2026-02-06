use super::*;

struct Entry {
  branch: String,
  head: String,
  path: String,
}

impl TryFrom<&str> for Entry {
  type Error = Error;

  fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
    let path = value
      .lines()
      .find_map(|line| line.strip_prefix("worktree "))
      .ok_or_else(|| anyhow!("missing worktree path"))?
      .to_string();

    let head = value
      .lines()
      .find_map(|line| line.strip_prefix("HEAD "))
      .map_or_else(
        || "unknown".to_string(),
        |h| h[..h.len().min(7)].to_string(),
      );

    let branch = value
      .lines()
      .find_map(|line| {
        line
          .strip_prefix("branch refs/heads/")
          .map(str::to_string)
          .or_else(|| (line == "detached").then(|| "(detached)".to_string()))
      })
      .ok_or_else(|| anyhow!("missing branch"))?;

    Ok(Entry { branch, head, path })
  }
}

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

  let entries = str::from_utf8(&output.stdout)?
    .split("\n\n")
    .filter_map(|block| Entry::try_from(block).ok())
    .filter(|entry| Path::new(&entry.path).is_dir())
    .collect::<Vec<_>>();

  if entries.is_empty() {
    bail!("no worktrees found");
  }

  let branch_width = entries.iter().map(|e| e.branch.len()).max().unwrap_or(0);

  for entry in &entries {
    let is_current = current_dir.starts_with(&entry.path);

    let marker = if is_current {
      style.apply(style::GREEN, "*")
    } else {
      style.apply(style::GREEN, " ")
    };

    println!(
      "{} {:<width$}  {}  {}",
      marker,
      style.apply(style::BOLD, &entry.branch),
      style.apply(style::CYAN, &entry.head),
      entry.path,
      width = branch_width,
    );
  }

  Ok(())
}
