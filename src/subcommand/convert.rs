use super::*;

#[cfg(unix)]
struct Branch(String);

#[cfg(unix)]
impl SkimItem for Branch {
  fn output(&self) -> Cow<'_, str> {
    Cow::Borrowed(&self.0)
  }

  fn text(&self) -> Cow<'_, str> {
    Cow::Borrowed(&self.0)
  }
}

#[cfg(not(unix))]
pub(crate) fn run() -> Result {
  bail!("interactive selection is not supported on this platform");
}

#[cfg(unix)]
pub(crate) fn run() -> Result {
  let style = Style::stderr();

  let root = Command::new("git")
    .args(["rev-parse", "--show-toplevel"])
    .stderr(Stdio::null())
    .output()?;

  if !root.status.success() {
    bail!("not a git repository");
  }

  let root = Path::new(str::from_utf8(&root.stdout)?.trim()).to_path_buf();

  let project = root
    .file_name()
    .ok_or_else(|| {
      anyhow!("failed to get project name from `{}`", root.display())
    })?
    .to_string_lossy()
    .to_string();

  let branch_output = Command::new("git")
    .args(["branch", "--format=%(refname:short)"])
    .stderr(Stdio::null())
    .output()?;

  if !branch_output.status.success() {
    bail!("failed to list branches");
  }

  let all_branches = str::from_utf8(&branch_output.stdout)?
    .lines()
    .map(str::to_string)
    .collect::<Vec<_>>();

  let worktree_output = Command::new("git")
    .args(["worktree", "list", "--porcelain"])
    .stderr(Stdio::null())
    .output()?;

  if !worktree_output.status.success() {
    bail!("failed to list worktrees");
  }

  let worktree_branches = str::from_utf8(&worktree_output.stdout)?
    .split("\n\n")
    .filter_map(|block| Worktree::try_from(block).ok())
    .map(|w| w.branch)
    .collect::<std::collections::HashSet<_>>();

  let branches = all_branches
    .into_iter()
    .filter(|branch| !worktree_branches.contains(branch))
    .collect::<Vec<_>>();

  if branches.is_empty() {
    bail!("no branches without worktrees");
  }

  let items = branches
    .into_iter()
    .map(|branch| Arc::new(Branch(branch)) as Arc<dyn SkimItem>)
    .collect::<Vec<Arc<dyn SkimItem>>>();

  let options = SkimOptionsBuilder::default().multi(true).build()?;

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
    .map(|item| item.text().to_string())
    .collect::<Vec<_>>();

  if selected.is_empty() {
    return Ok(());
  }

  let parent = root.parent().ok_or_else(|| {
    anyhow!("repo root `{}` has no parent directory", root.display())
  })?;

  let mut created = Vec::new();

  for branch in &selected {
    let dir_name = format!("{}.{}", project, branch.replace('/', "-"));

    let worktree = parent.join(&dir_name);

    let result = Command::new("git")
      .args(["worktree", "add", &worktree.to_string_lossy(), branch])
      .stdout(Stdio::null())
      .stderr(Stdio::piped())
      .output()?;

    if !result.status.success() {
      bail!(
        "failed to create worktree for `{}`: {}",
        branch,
        str::from_utf8(&result.stderr)?.trim()
      );
    }

    eprintln!(
      "{} worktree {} at {}",
      style.apply(style::GREEN, "created"),
      style.apply(style::BOLD, branch),
      style.apply(style::CYAN, &dir_name),
    );

    created.push(worktree);
  }

  if selected.len() == 1 {
    println!("{}", created[0].display());
  }

  Ok(())
}
