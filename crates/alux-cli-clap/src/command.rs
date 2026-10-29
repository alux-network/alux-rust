use crate::words::id;
use alux_cli::{ArgumentInfo, Arity, CommandDescription, Part, SourceInfo};
use clap::{Arg, ArgAction, ArgGroup, Command};

/// Builds the `clap` command named `name` for the commands under `prefix`.
///
/// A command is a subcommand per path segment. A global argument is declared once, `global`, on the
/// command its context begins at: the longest path every command applied within it shares.
pub(crate) fn command(name: String, descriptions: &[&CommandDescription], prefix: &[&'static str]) -> Command {
    let mut command = Command::new(name);
    for argument in globals(descriptions, prefix) {
        command = command.arg(arg(&argument, true));
    }
    let own = descriptions.iter().find(|description| description.path.len() == prefix.len());
    if let Some(description) = own {
        command = command.about(description.doc.lines().next().unwrap_or_default().trim().to_owned());
        if description.doc.lines().count() > 1 {
            command = command.long_about(description.doc.trim().to_owned());
        }
        command = arguments(command, &description.arguments);
    }
    let mut children: Vec<&'static str> = Vec::new();
    for description in descriptions {
        if let Some(&segment) = description.path.get(prefix.len())
            && !children.contains(&segment)
        {
            children.push(segment);
        }
    }
    for child in &children {
        let path = [prefix, &[*child]].concat();
        let beneath =
            descriptions.iter().copied().filter(|description| description.path.starts_with(&path)).collect::<Vec<_>>();
        command = command.subcommand(self::command((*child).to_owned(), &beneath, &path));
    }
    if own.is_none() {
        command = command.subcommand_required(true).arg_required_else_help(true);
    } else if !children.is_empty() {
        // Selecting a command selects only it, so the program's own arguments are not required.
        command = command.subcommand_negates_reqs(true);
    }

    command
}

/// The global arguments whose context begins at `prefix`.
fn globals(descriptions: &[&CommandDescription], prefix: &[&'static str]) -> Vec<ArgumentInfo> {
    let mut globals: Vec<ArgumentInfo> = Vec::new();
    for argument in descriptions.iter().flat_map(|description| &description.context) {
        let applied = descriptions.iter().filter(|description| description.context.contains(argument));
        let shared = applied.map(|description| &description.path[..]).reduce(common).unwrap_or_default();
        if shared == prefix && !globals.contains(argument) {
            globals.push(argument.clone());
        }
    }
    globals
}

fn common<'a>(left: &'a [&'static str], right: &'a [&'static str]) -> &'a [&'static str] {
    let shared = left.iter().zip(right).take_while(|(left, right)| left == right).count();
    &left[..shared]
}

/// Declares a command's own arguments, a sum's alternatives grouped so `clap` rejects two of them.
fn arguments(mut command: Command, arguments: &[ArgumentInfo]) -> Command {
    let mut sums: Vec<(&'static str, Vec<String>)> = Vec::new();
    for argument in arguments {
        command = command.arg(arg(argument, false));
        if let Some(Part::Sum(sum)) = argument.part_of {
            match sums.iter_mut().find(|(name, _)| *name == sum) {
                Some((_, ids)) => ids.push(id(argument)),
                None => sums.push((sum, vec![id(argument)])),
            }
        }
    }
    for (sum, ids) in sums {
        command = command.group(ArgGroup::new(sum).args(ids).multiple(false));
    }
    command
}

/// Names the value an argument takes as the command line spells it, as `alux-cli-direct` does.
fn value_name(argument: &ArgumentInfo) -> String {
    match &argument.source {
        SourceInfo::Named { long, .. } => long.clone(),
        SourceInfo::Assigned { .. } => "VALUE".to_owned(),
        SourceInfo::Positional { .. } => argument.name.replace('_', "-"),
    }
}

/// Declares one argument as `clap` reads it.
///
/// `clap` requires only what has no fallback and is no part of a compound argument; everything else
/// is left to decoding, which reports what is missing in the program's own terms.
fn arg(argument: &ArgumentInfo, global: bool) -> Arg {
    let env = argument.fallback.env.map(|env| format!(" [env: {env}]")).unwrap_or_default();
    let default = argument.fallback.default.map(|default| format!(" [default: {default}]")).unwrap_or_default();
    let help = format!("{}{env}{default}", argument.doc);
    let mut arg = Arg::new(id(argument)).help(help).global(global);
    arg = match &argument.source {
        SourceInfo::Positional { index } => arg.index(index + 1),
        // `clap` has no operand `name=value`; the run rewrites each into this option first.
        SourceInfo::Assigned { name } => arg.long(assigned_long(name)).require_equals(true),
        SourceInfo::Named { long, short, short_aliases } => {
            let arg = arg.long(long.clone()).visible_short_aliases(short_aliases.iter().copied());
            match short {
                Some(short) => arg.short(*short),
                None => arg,
            }
        }
    };
    arg = match argument.arity {
        Arity::Presence => arg.action(ArgAction::Count),
        Arity::One | Arity::Optional => arg.action(ArgAction::Set).value_name(value_name(argument)),
        Arity::Many => arg.action(ArgAction::Append).value_name(value_name(argument)),
        // clap reads an optional value after `=` only, and a bare occurrence as the empty word.
        Arity::OptionalValue => arg
            .action(ArgAction::Set)
            .num_args(0..=1)
            .require_equals(true)
            .default_missing_value("")
            .value_name(value_name(argument)),
    };
    // A value is read as its raw word, whatever its bytes, and decoded by the argument's own type.
    if argument.arity != Arity::Presence {
        arg = arg.value_parser(clap::builder::OsStringValueParser::new());
    }
    // A list one word can give is split by `clap` as it reads the command line.
    if let Some(delimiter) = argument.delimiter {
        arg = arg.value_delimiter(delimiter);
    }
    // A repeated positional argument takes every remaining word; a repeated option one per occurrence.
    if argument.arity == Arity::Many && matches!(argument.source, SourceInfo::Positional { .. }) {
        arg = arg.num_args(1..);
    }
    let fallback = argument.fallback.env.is_some() || argument.fallback.default.is_some();
    arg.required(argument.arity == Arity::One && !fallback && !global && argument.part_of.is_none())
}

/// The option an operand assigning `name` is read as by `clap`: one no declaration spells.
pub(crate) fn assigned_long(name: &str) -> String {
    format!("assign-{name}")
}
