use {super::*, skim::prelude::*};

#[derive(Debug, PartialEq, Eq)]
struct WorktreeItem {
  branch: String,
  path: String,
}

impl SkimItem for WorktreeItem {
  fn output(&self) -> Cow<'_, str> {
    Cow::Borrowed(&self.path)
  }

  fn text(&self) -> Cow<'_, str> {
    Cow::Borrowed(&self.branch)
  }
}

/// Switch to a different worktree.
#[derive(Debug, Parser)]
pub(crate) struct Switch;

impl Switch {
  fn parse_worktrees(input: &str) -> Vec<WorktreeItem> {
    input
      .split("\n\n")
      .filter_map(|block| {
        let path = block
          .lines()
          .find_map(|line| line.strip_prefix("worktree "))?
          .to_string();

        let branch = block.lines().find_map(|line| {
          line
            .strip_prefix("branch refs/heads/")
            .map(str::to_string)
            .or_else(|| (line == "detached").then(|| "(detached)".to_string()))
        })?;

        Some(WorktreeItem { branch, path })
      })
      .filter(|worktree| Path::new(&worktree.path).is_dir())
      .collect()
  }

  pub(crate) fn run() -> Result {
    let output = Command::new("git")
      .args(["worktree", "list", "--porcelain"])
      .stderr(Stdio::null())
      .output()?;

    if !output.status.success() {
      bail!("failed to list worktrees");
    }

    let worktrees = Self::parse_worktrees(str::from_utf8(&output.stdout)?);

    if worktrees.is_empty() {
      bail!("no worktrees found");
    }

    let items = worktrees
      .into_iter()
      .map(|worktree| Arc::new(worktree) as Arc<dyn SkimItem>)
      .collect::<Vec<Arc<dyn SkimItem>>>();

    let options = SkimOptionsBuilder::default()
      .prompt("worktree> ".to_string())
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
}

#[cfg(test)]
mod tests {
  use {super::*, tempfile::tempdir};

  #[test]
  fn filters_nonexistent_paths() {
    let input = "worktree /no/such/path\nHEAD abc123\nbranch refs/heads/gone\n";

    assert_eq!(Switch::parse_worktrees(input), Vec::new());
  }

  #[test]
  fn parse_detached_head() {
    let directory = tempdir().unwrap();

    let path = directory.path().display();

    assert_eq!(
      Switch::parse_worktrees(&format!(
        "worktree {path}\nHEAD abc123\ndetached\n"
      )),
      vec![WorktreeItem {
        branch: "(detached)".to_string(),
        path: path.to_string(),
      }],
    );
  }

  #[test]
  fn parse_empty_input() {
    assert_eq!(Switch::parse_worktrees(""), Vec::new());
  }

  #[test]
  fn parse_multiple_worktrees() {
    let (directory_a, directory_b) = (tempdir().unwrap(), tempdir().unwrap());

    let (path_a, path_b) =
      (directory_a.path().display(), directory_b.path().display());

    let input = format!(
      "worktree {path_a}\nHEAD abc123\nbranch refs/heads/main\n\n\
       worktree {path_b}\nHEAD def456\nbranch refs/heads/feature\n"
    );

    assert_eq!(
      Switch::parse_worktrees(&input),
      vec![
        WorktreeItem {
          branch: "main".to_string(),
          path: path_a.to_string(),
        },
        WorktreeItem {
          branch: "feature".to_string(),
          path: path_b.to_string(),
        },
      ],
    );
  }

  #[test]
  fn parse_single_branch() {
    let directory = tempdir().unwrap();

    let path = directory.path().display();

    assert_eq!(
      Switch::parse_worktrees(&format!(
        "worktree {path}\nHEAD abc123\nbranch refs/heads/main\n"
      )),
      vec![WorktreeItem {
        branch: "main".to_string(),
        path: path.to_string(),
      }],
    );
  }

  #[test]
  fn skips_block_without_branch() {
    let directory = tempdir().unwrap();

    let path = directory.path().display();

    assert_eq!(
      Switch::parse_worktrees(&format!("worktree {path}\nHEAD abc123\n")),
      Vec::new()
    );
  }
}
