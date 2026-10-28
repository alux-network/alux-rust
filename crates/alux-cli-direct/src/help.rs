use alux_cli::{ArgumentInfo, Arity, CommandDescription, Fallback, Part, SourceInfo};
use core::fmt::Write;

/// Renders the help of a whole program: its commands with the first line of each one's documentation.
pub(crate) fn program(name: &str, commands: &[&CommandDescription], version: Option<&str>) -> String {
    let root = commands.iter().find(|command| command.path.is_empty());
    // A program that is only its own command is helped as that command.
    let version = version.map(|version| ("-V, --version".to_owned(), format!("Prints {name} {version}")));
    if let Some(root) = root.filter(|_| commands.len() == 1) {
        return described_command(name, root, version.into_iter().collect());
    }
    let rows = commands
        .iter()
        .filter(|command| !command.path.is_empty())
        .map(|command| (command.path.join(" "), command.doc.lines().next().unwrap_or_default().to_owned()))
        .collect::<Vec<_>>();
    // The global options every command takes are the program's own, so they are listed with it.
    let shared = commands.first().map_or_else(Vec::new, |first| {
        first
            .context
            .iter()
            .filter(|argument| commands.iter().all(|command| command.context.contains(argument)))
            .collect()
    });
    let options = if shared.is_empty() { "" } else { " [options]" };
    let mut out = root.map(|root| usage(name, root)).unwrap_or_default();
    let _ = writeln!(out, "{}{name}{options} <command>", if root.is_some() { "       " } else { "Usage: " });
    if !rows.is_empty() {
        out.push_str("\nCommands:\n");
        table(&mut out, &rows);
    }
    if !shared.is_empty() {
        out.push_str("\nGlobal options:\n");
        table(&mut out, &shared.iter().map(|argument| (spelling(argument), described(argument))).collect::<Vec<_>>());
    }
    // The program's own command's arguments and options are the program's.
    match root {
        Some(root) => tables(&mut out, root, version.into_iter().collect()),
        None => {
            if let Some(version) = version {
                out.push_str("\nOptions:\n");
                table(&mut out, &[version]);
            }
        }
    }
    out
}

/// Writes a command's arguments and its options, `more` options after its own.
fn tables(out: &mut String, command: &CommandDescription, more: Vec<(String, String)>) {
    let (assigned, rest): (Vec<_>, Vec<_>) =
        command.arguments.iter().partition(|argument| matches!(argument.source, SourceInfo::Assigned { .. }));
    let (positional, named): (Vec<_>, Vec<_>) =
        rest.into_iter().partition(|argument| matches!(argument.source, SourceInfo::Positional { .. }));
    if !positional.is_empty() {
        out.push_str("\nArguments:\n");
        table(out, &positional.iter().map(|argument| (placeholder(argument), described(argument))).collect::<Vec<_>>());
    }
    let mut options = named.iter().map(|argument| (spelling(argument), described(argument))).collect::<Vec<_>>();
    options.extend(more);
    if !options.is_empty() {
        out.push_str("\nOptions:\n");
        table(out, &options);
    }
    if !assigned.is_empty() {
        out.push_str("\nAssignments:\n");
        table(out, &assigned.iter().map(|argument| (placeholder(argument), described(argument))).collect::<Vec<_>>());
    }
}

/// Renders the help of one command: its documentation, usage, arguments, and options.
pub(crate) fn command(name: &str, command: &CommandDescription) -> String {
    described_command(name, command, Vec::new())
}

/// Renders the help of one command, with `more` options after its own.
fn described_command(name: &str, command: &CommandDescription, more: Vec<(String, String)>) -> String {
    let mut out = String::new();
    if !command.doc.is_empty() {
        out.push_str(command.doc);
        out.push_str("\n\n");
    }
    out.push_str(&usage(name, command));
    tables(&mut out, command, more);
    if !command.context.is_empty() {
        out.push_str("\nGlobal options:\n");
        table(
            &mut out,
            &command.context.iter().map(|argument| (spelling(argument), described(argument))).collect::<Vec<_>>(),
        );
    }
    out
}

/// Renders the one-line usage of a command.
pub(crate) fn usage(name: &str, command: &CommandDescription) -> String {
    // A program invoked by a command's own name shows that command beneath no program name.
    let spelled = [name].into_iter().chain(command.path.iter().copied()).filter(|part| !part.is_empty());
    let mut out = format!("Usage: {}", spelled.collect::<Vec<_>>().join(" "));
    if command
        .context
        .iter()
        .chain(&command.arguments)
        .any(|argument| matches!(argument.source, SourceInfo::Named { .. }))
    {
        out.push_str(" [options]");
    }
    for argument in &command.arguments {
        if !matches!(argument.source, SourceInfo::Named { .. }) {
            let _ = write!(out, " {}", placeholder(argument));
        }
    }
    out.push('\n');
    out
}

/// Writes a positional argument as its usage spells it: `<name>`, `[name]`, or `[name]...`, and an
/// assigned one as `if=VALUE`, `[if=VALUE]`, or `[if=VALUE]...`.
fn placeholder(argument: &ArgumentInfo) -> String {
    let given = argument.arity == Arity::One && argument.fallback == Fallback::default();
    let spelled = match &argument.source {
        SourceInfo::Assigned { name } => {
            return match argument.arity {
                Arity::Many => format!("[{name}=VALUE]..."),
                _ if given => format!("{name}=VALUE"),
                _ => format!("[{name}=VALUE]"),
            };
        }
        _ => value_name(argument),
    };
    match argument.arity {
        Arity::Many => format!("[{spelled}]..."),
        _ if given => format!("<{spelled}>"),
        _ => format!("[{spelled}]"),
    }
}

/// Names the value an argument takes as the command line spells it: an option by its long
/// spelling, `--base-path <base-path>`, and a positional argument by its name with `-` for `_`.
pub(crate) fn value_name(argument: &ArgumentInfo) -> String {
    match &argument.source {
        SourceInfo::Named { long, .. } => long.clone(),
        SourceInfo::Assigned { name } => name.clone(),
        SourceInfo::Positional { .. } => argument.name.replace('_', "-"),
    }
}

/// Writes a named argument as its spellings and the value it takes.
fn spelling(argument: &ArgumentInfo) -> String {
    let SourceInfo::Named { long, short, short_aliases } = &argument.source else { return placeholder(argument) };
    let mut out = short.map_or_else(|| "    ".to_owned(), |short| format!("-{short}, "));
    for alias in *short_aliases {
        let _ = write!(out, "-{alias}, ");
    }
    let _ = write!(out, "--{long}");
    match argument.arity {
        Arity::Presence => {}
        // A list one word can give shows its delimiter.
        Arity::Many => match argument.delimiter {
            Some(delimiter) => {
                let _ = write!(out, " <{}>{delimiter}...", value_name(argument));
            }
            None => {
                let _ = write!(out, " <{}>...", value_name(argument));
            }
        },
        Arity::One | Arity::Optional => {
            let _ = write!(out, " <{}>", value_name(argument));
        }
        // The value is written in the option's own word, so help shows it attached.
        Arity::OptionalValue => {
            let _ = write!(out, "[=<{}>]", value_name(argument));
        }
    }
    out
}

/// Writes an argument's documentation followed by its fallbacks.
fn described(argument: &ArgumentInfo) -> String {
    let mut out = argument.doc.to_owned();
    if let Some(env) = argument.fallback.env {
        let _ = write!(out, " [env: {env}]");
    }
    if let Some(default) = argument.fallback.default {
        let _ = write!(out, " [default: {default}]");
    }
    match argument.part_of {
        Some(Part::Product(product)) => {
            let _ = write!(out, " [with <{product}>]");
        }
        Some(Part::Sum(sum)) => {
            let _ = write!(out, " [one of <{sum}>]");
        }
        None => {}
    }
    out
}

/// Writes rows as two aligned columns.
fn table(out: &mut String, rows: &[(String, String)]) {
    let width = rows.iter().map(|(left, _)| left.chars().count()).max().unwrap_or_default();
    for (left, right) in rows {
        let _ = writeln!(out, "  {left:width$}  {right}");
    }
}
