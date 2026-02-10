use {
  anyhow::Error,
  indoc::indoc,
  pretty_assertions::assert_eq,
  regex::Regex,
  std::{
    iter::once,
    path::{Path, PathBuf},
    process::{Command, Output},
    str,
  },
  tempfile::TempDir,
};

type Result<T = (), E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
struct Test<'a> {
  arguments: Vec<String>,
  exists: Vec<&'a str>,
  expected_status: i32,
  expected_stderr: String,
  expected_stdout: String,
  tempdir: TempDir,
  workdir: PathBuf,
}

impl<'a> Test<'a> {
  fn argument(self, argument: &str) -> Self {
    Self {
      arguments: self
        .arguments
        .into_iter()
        .chain(once(argument.to_owned()))
        .collect(),
      ..self
    }
  }

  fn command(&self, arguments: &[String]) -> Result<Output> {
    Ok(
      Command::new(env!("CARGO_BIN_EXE_wt"))
        .args(arguments)
        .current_dir(&self.workdir)
        .env("NO_COLOR", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_CEILING_DIRECTORIES", self.tempdir.path())
        .output()?,
    )
  }

  fn exists(self, paths: &[&'a str]) -> Self {
    Self {
      exists: self
        .exists
        .into_iter()
        .chain(paths.iter().copied())
        .collect(),
      ..self
    }
  }

  fn expected_status(self, expected_status: i32) -> Self {
    Self {
      expected_status,
      ..self
    }
  }

  fn expected_stderr(self, expected_stderr: &str) -> Self {
    Self {
      expected_stderr: expected_stderr.to_owned(),
      ..self
    }
  }

  fn expected_stdout(self, expected_stdout: &str) -> Self {
    Self {
      expected_stdout: expected_stdout.to_owned(),
      ..self
    }
  }

  fn git(directory: &Path, arguments: &[&str]) {
    let output = Command::new("git")
      .args(arguments)
      .current_dir(directory)
      .env("GIT_CONFIG_GLOBAL", "/dev/null")
      .env("GIT_CONFIG_SYSTEM", "/dev/null")
      .output()
      .unwrap();

    assert!(
      output.status.success(),
      "git {} failed: {}",
      arguments.join(" "),
      String::from_utf8_lossy(&output.stderr)
    );
  }

  fn new(name: &str) -> Result<Self> {
    let tempdir = TempDir::new()?;

    let workdir = tempdir.path().join(name);

    Self::git(
      tempdir.path(),
      &["init", "-b", "main", workdir.to_str().unwrap()],
    );

    Self::git(&workdir, &["config", "user.email", "test@test.com"]);
    Self::git(&workdir, &["config", "user.name", "Test"]);

    Self::git(
      &workdir,
      &["commit", "--allow-empty", "-m", "Initial commit"],
    );

    Ok(Self {
      arguments: Vec::new(),
      exists: Vec::new(),
      expected_status: 0,
      expected_stderr: String::new(),
      expected_stdout: String::new(),
      tempdir,
      workdir,
    })
  }

  fn run(self) -> Result {
    let root = self.tempdir.path().canonicalize()?.display().to_string();

    let output = self.command(&self.arguments)?;

    let hash_regex = Regex::new(r"\b[0-9a-fA-F]{7,40}\b").unwrap();

    let stderr = hash_regex
      .replace_all(
        &str::from_utf8(&output.stderr)?.replace(&root, "[ROOT]"),
        "[HASH]",
      )
      .into_owned();

    let stdout = hash_regex
      .replace_all(
        &str::from_utf8(&output.stdout)?.replace(&root, "[ROOT]"),
        "[HASH]",
      )
      .into_owned();

    assert_eq!(
      output.status.code(),
      Some(self.expected_status),
      "unexpected exit status\nstderr: {stderr}"
    );

    if self.expected_stderr.is_empty() && !stderr.is_empty() {
      panic!("expected empty stderr, got: {stderr}");
    } else {
      assert_eq!(stderr, self.expected_stderr);
    }

    assert_eq!(stdout, self.expected_stdout);

    for path in &self.exists {
      assert!(
        self.tempdir.path().join(path).exists(),
        "expected path to exist: {path}"
      );
    }

    Ok(())
  }

  fn setup(self, arguments: &[&str]) -> Self {
    let arguments = arguments
      .iter()
      .map(|argument| (*argument).to_owned())
      .collect::<Vec<String>>();

    let output = self.command(&arguments).unwrap();

    assert!(
      output.status.success(),
      "setup `wt {}` failed: {}",
      arguments.join(" "),
      String::from_utf8_lossy(&output.stderr)
    );

    self
  }

  fn without_git() -> Result<Self> {
    let tempdir = TempDir::new()?;

    let workdir = tempdir.path().to_path_buf();

    Ok(Self {
      arguments: Vec::new(),
      exists: Vec::new(),
      expected_status: 0,
      expected_stderr: String::new(),
      expected_stdout: String::new(),
      tempdir,
      workdir,
    })
  }
}

#[test]
fn create() -> Result {
  Test::new("project")?
    .argument("create")
    .argument("feature")
    .exists(&["project.feature"])
    .expected_stderr("created worktree feature at project.feature\n")
    .expected_stdout("[ROOT]/project.feature\n")
    .run()
}

#[test]
fn create_duplicate_branch() -> Result {
  Test::new("project")?
    .setup(&["create", "feature"])
    .argument("create")
    .argument("feature")
    .exists(&["project.feature"])
    .expected_status(1)
    .expected_stderr(indoc! {
      "
      error: failed to create worktree `feature`: Preparing worktree (new branch 'feature')
      fatal: a branch named 'feature' already exists
      "
    })
    .run()
}

#[test]
fn create_outside_git_repo() -> Result {
  Test::without_git()?
    .argument("create")
    .argument("feature")
    .expected_status(1)
    .expected_stderr("error: failed to get project name from ``\n")
    .run()
}

#[test]
fn create_slash_in_branch_name() -> Result {
  Test::new("project")?
    .argument("create")
    .argument("feat/my-branch")
    .exists(&["project.feat-my-branch"])
    .expected_stderr(
      "created worktree feat/my-branch \
       at project.feat-my-branch\n",
    )
    .expected_stdout("[ROOT]/project.feat-my-branch\n")
    .run()
}

#[test]
fn init_zsh() -> Result {
  Test::new("project")?
    .argument("init")
    .argument("zsh")
    .expected_stdout(include_str!("../src/subcommand/init.zsh"))
    .run()
}

#[test]
fn list() -> Result {
  Test::new("project")?
    .argument("list")
    .expected_stdout("* main  [HASH]  +0/-0  [ROOT]/project\n")
    .run()
}

#[test]
fn list_after_create() -> Result {
  Test::new("project")?
    .setup(&["create", "feature"])
    .argument("list")
    .expected_stdout(indoc! {
      "
      * main     [HASH]  +0/-0  [ROOT]/project
        feature  [HASH]  +0/-0  [ROOT]/project.feature
      "
    })
    .run()
}

#[test]
fn list_outside_git_repo() -> Result {
  Test::without_git()?
    .argument("list")
    .expected_status(1)
    .expected_stderr("error: failed to list worktrees\n")
    .run()
}

#[test]
fn version() -> Result {
  Test::without_git()?
    .argument("--version")
    .expected_stdout("wt-cli 0.1.1\n")
    .run()
}
