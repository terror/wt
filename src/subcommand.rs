use super::*;

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  Create,
  Remove,
  Switch,
}
