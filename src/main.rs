use anyhow::{bail, Context, Result};
use std::io;

#[derive(Debug, Clone)]
enum ActionType {
  Create,
  Update,
  Destroy,
  DestroyThenCreate,
  DuplicateThenRemove,
}

impl Default for ActionType {
  fn default() -> Self {
    Self::Create
  }
}

#[derive(Debug, Default, Clone)]
struct Action {
  typ: ActionType,
  // example: module.abc.aws_iam_policy.name[key]
  reference: String,
  // example: aws_iam_policy
  resource: String,
  // example: name
  name: String,
  content: String,
}

fn main() -> Result<()> {
  let mut line = String::new();
  // skip to actions start or no changes
  while io::stdin().read_line(&mut line)? > 0 {
    if line.starts_with("Terraform will perform the following actions:") {
      break;
    }

    if line.starts_with("No changes. Infrastructure is up-to-date.") {
      return Ok(());
    }
    line.clear();
  }

  let mut actions = Vec::new();

  // read first action
  while io::stdin().read_line(&mut line)? > 0 {
    if line.starts_with("  # ") {
      break;
    }
    line.clear();
  }

  let mut this_action = read_action_header(&mut line).context("expect first Action")?;

  loop {
    if line.starts_with("  # ") {
      this_action = read_action_header(&mut line).context("expect header for Action")?;
    } else if line.starts_with("Plan: ") {
      break;
    } else {
      // end of an action
      if line.starts_with("    }") {
        // the `clone` is just for compiler
        actions.push(this_action.clone());
      } else {
        this_action.content.push_str(&line);
      }
    }

    line.clear();

    if io::stdin().read_line(&mut line)? == 0 {
      break;
    }
  }

  Ok(())
}

fn read_action_header(mut line: &mut String) -> Result<Action> {
  // TODO: handle whitespace in key
  let reference = line
    .split_whitespace()
    .nth(1)
    .context("expecting resource reference")?
    .to_string();

  line.clear();
  io::stdin().read_line(&mut line).context("expecting Action detail")?;

  let (typ_text, rest) = line.split_at(4);

  let typ = match typ_text {
    "  + " => ActionType::Create,
    "  ~ " => ActionType::Update,
    "  - " => ActionType::Destroy,
    "-/+ " => ActionType::DestroyThenCreate,
    "+/- " => ActionType::DuplicateThenRemove,
    _ => bail!("unexpected Action type: {}", typ_text),
  };

  // remove 'resources '
  let (_, rest) = rest.split_at(9);

  let rest = rest.trim();

  let rest = rest.strip_prefix("\"").context("expecting beginning quote")?;
  let rest = rest.strip_suffix("\" {").context("expecting ending quote")?;
  let mut parts = rest.split("\" \"");
  let resource = parts.next().context("expecting resource")?.to_string();
  let name = parts.next().context("expecting name")?.to_string();

  Ok(Action {
    typ,
    reference,
    resource,
    name,
    content: String::new(),
  })
}
