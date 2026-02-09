use super::*;

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Config {
  #[serde(default)]
  pub(crate) hooks: Hooks,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct HookEntry {
  pub(crate) command: String,
  #[serde(default)]
  pub(crate) only_if: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Hooks {
  #[serde(default)]
  pub(crate) post_worktree_change: Vec<HookEntry>,
}

impl Config {
  pub(crate) fn load() -> Result<Self> {
    Ok(confy::load("wt", "config")?)
  }
}

impl HookEntry {
  pub(crate) fn matches(&self) -> Result<bool> {
    let Some(pattern) = &self.only_if else {
      return Ok(true);
    };

    let cwd = env::current_dir()?;

    Ok(
      glob::glob(cwd.join(pattern).to_string_lossy().as_ref())?
        .next()
        .is_some(),
    )
  }
}
