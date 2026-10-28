//! Runs the notes application of `alux-cli-direct`'s tests through `clap`, end to end.

use alux_cli::{CliCommandsAlg, CliFailureAlg, CliNestAlg, cli};
use alux_cli_clap::CliClap;
use alux_cli_direct::Exit;
use alux_ext::ext;
use alux_shape::Shape;
use core::{fmt, num::NonZeroU8};
use std::sync::{Arc, Mutex};

trait Notes {
    fn notes(&self) -> Vec<String>;
    fn notes_set(&self, notes: Vec<String>);
    fn notes_tagged(&self, tag: &str) -> Arc<Store>;
}

/// A note that does not exist.
#[derive(Debug)]
struct NoSuchNote(usize);

impl fmt::Display for NoSuchNote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "no note {}", self.0)
    }
}

impl CliFailureAlg for NoSuchNote {
    fn cli_status(&self) -> NonZeroU8 {
        NonZeroU8::new(3).unwrap()
    }
}

#[ext(name = NotesExt, defunc)]
impl<This> This
where
    This: Notes,
{
    /// Adds a note, tagged, and answers its number.
    fn add_note(&self, text: String, tags: Vec<String>) -> usize {
        let mut notes = self.notes();
        let note = tags.iter().fold(text, |note, tag| note + " #" + tag);
        notes.push(note);
        self.notes_set(notes);
        self.notes().len()
    }

    /// Lists notes.
    ///
    /// Shows the first `limit` notes, numbered when verbose.
    fn list_notes(&self, limit: Option<usize>, verbose: bool) -> Vec<String> {
        let notes = self.notes().into_iter().take(limit.unwrap_or(usize::MAX));
        notes.enumerate().map(|(index, note)| if verbose { format!("{}. {note}", index + 1) } else { note }).collect()
    }

    /// Removes a note by its number.
    fn remove_note(&self, number: usize) -> Result<(), NoSuchNote> {
        let mut notes = self.notes();
        if number == 0 || number > notes.len() {
            return Err(NoSuchNote(number));
        }
        notes.remove(number - 1);
        self.notes_set(notes);
        Ok(())
    }

    /// Greets the owner of the notes.
    fn greet(&self, owner: String) -> String {
        format!("notes of {owner}")
    }

    /// Exports the notes to one target, signed in or not.
    fn export(&self, target: Target, auth: Option<Credentials>) -> String {
        let to = match target {
            Target::File(path) => format!("file {path}"),
            Target::Url(url) => format!("url {url}"),
            Target::Stdout => "stdout".to_owned(),
        };
        let user = auth.map(|auth| format!(" as {}:{}", auth.user, auth.password.len())).unwrap_or_default();
        format!("{} notes to {to}{user}", self.notes().len())
    }

    /// Narrows the notes to those with a tag, or keeps every note.
    fn scope(&self, only: Option<String>) -> Arc<Store> {
        self.notes_tagged(only.as_deref().unwrap_or_default())
    }
}

#[ext(name = NotesCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg + CliNestAlg<Commands = <This as CliCommandsAlg>::Commands>,
{
    /// Declares the notes commands.
    fn notes_cli<Store>(&self) {
        self.commands()
            .command(
                "add",
                self.op(Store::add_note)
                    .arg(self.positional("The note's text"))
                    .arg(self.named("A tag for the note").many().long("tag").short('t')),
            )
            .command(
                "list",
                self.op(Store::list_notes)
                    .arg(self.named("Maximum number of notes to show").optional().short('n'))
                    .arg(self.named("Number the notes").switch().short('v')),
            )
            .command("remove", self.op(Store::remove_note).arg(self.positional("The note's number")).fallible())
            .nest(
                "owner",
                self.commands().command(
                    "greet",
                    self.op(Store::greet).arg(self.named("Who owns the notes").env("CARGO_PKG_NAME").default("nobody")),
                ),
            )
    }

    /// Declares the notes commands, applied to the store `--store` opens from a library.
    fn library_cli<Shelf, Store>(&self) {
        self.commands().within(
            self.op(Shelf::open).arg(self.named("The store to use").short('s').default("main")).fallible(),
            self.notes_cli::<Store>(),
        )
    }

    /// Declares exporting: exactly one target, and the user with the password or neither.
    fn export_cli<Store>(&self) {
        self.commands().command(
            "export",
            self.op(Store::export)
                .arg(
                    self.sum()
                        .or(self.named("Write to a file").short('f'))
                        .or(self.named("Send to a URL"))
                        .or(self.named("Print the notes").switch()),
                )
                .arg(
                    self.product()
                        .field(self.named("User to sign in as"))
                        .field(self.named("Password to sign in with"))
                        .optional(),
                ),
        )
    }

    /// Declares the notes commands, applied to the notes `--only` narrows.
    fn scoped_cli<Store>(&self) {
        self.commands().within(
            self.op(Store::scope).arg(self.named("Only notes with this tag").optional().long("only")),
            self.notes_cli::<Store>(),
        )
    }

    /// Declares the scoped notes commands within the store `--store` opens: two contexts, nested.
    fn nested_cli<Shelf, Store>(&self) {
        self.commands().within(
            self.op(Shelf::open).arg(self.named("The store to use").short('s').default("main")).fallible(),
            self.scoped_cli::<Store>(),
        )
    }

    /// Declares the notes commands, applied to the main store, which takes no argument to open.
    fn main_cli<Shelf, Store>(&self) {
        self.commands().within(self.op(Shelf::main_store), self.notes_cli::<Store>())
    }
}

/// Where notes are exported.
#[derive(Shape)]
enum Target {
    File(String),
    Url(String),
    Stdout,
}

/// Who exports the notes.
#[derive(Shape)]
struct Credentials {
    user: String,
    password: String,
}

trait Library {
    fn store(&self, name: &str) -> Option<Arc<Store>>;
}

/// A store the library does not have.
#[derive(Debug)]
struct NoSuchStore(String);

impl fmt::Display for NoSuchStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "no store `{}`", self.0)
    }
}

impl CliFailureAlg for NoSuchStore {
    fn cli_status(&self) -> NonZeroU8 {
        NonZeroU8::new(4).unwrap()
    }
}

#[ext(name = LibraryExt, defunc)]
impl<This> This
where
    This: Library,
{
    /// Opens a store by name.
    fn open(&self, store: String) -> Result<Arc<Store>, NoSuchStore> {
        self.store(&store).ok_or(NoSuchStore(store))
    }

    /// Opens the main store.
    fn main_store(&self) -> Arc<Store> {
        self.store("main").expect("a library has a main store")
    }
}

/// Two stores, by name.
#[derive(Default)]
struct Shelf {
    main: Arc<Store>,
    work: Arc<Store>,
}

impl Library for Shelf {
    fn store(&self, name: &str) -> Option<Arc<Store>> {
        match name {
            "main" => Some(Arc::clone(&self.main)),
            "work" => Some(Arc::clone(&self.work)),
            _ => None,
        }
    }
}

#[derive(Default)]
struct Store(Mutex<Vec<String>>);

impl Notes for Store {
    fn notes(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }

    fn notes_set(&self, notes: Vec<String>) {
        *self.0.lock().unwrap() = notes;
    }

    fn notes_tagged(&self, tag: &str) -> Arc<Store> {
        let tag = format!("#{tag}");
        let notes =
            self.notes().into_iter().filter(|note| note.split(' ').any(|word| word == tag || tag == "#")).collect();
        Arc::new(Store(Mutex::new(notes)))
    }
}

fn out(stdout: &str) -> Exit {
    Exit { status: 0, stdout: stdout.into(), stderr: String::new() }
}

fn first_line(exit: &Exit) -> (u8, &str) {
    (exit.status, exit.stderr.lines().next().unwrap_or_default())
}

#[test]
fn runs_commands_with_every_spelling() {
    let cli = CliClap::new(Arc::<Store>::default(), alux_cli::CliProgramBuilder.notes_cli::<Store>());
    let notes = |line: &str| cli.run_blocking(line.split_whitespace());
    assert_eq!(notes("notes add milk"), out("1\n"));
    assert_eq!(notes("notes add bread --tag shop -t=today"), out("2\n"));
    assert_eq!(notes("notes add -- -eggs"), out("3\n"));
    assert_eq!(notes("notes list -vn 2"), out("1. milk\n2. bread #shop #today\n"));
    assert_eq!(notes("notes list --limit=1 --verbose"), out("1. milk\n"));
    assert_eq!(notes("notes remove 1"), out(""));
    assert_eq!(notes("notes list"), out("bread #shop #today\n-eggs\n"));
    // A failure exits with its own status; the environment answers before the default.
    assert_eq!(notes("notes remove 7"), Exit { status: 3, stdout: String::new(), stderr: "error: no note 7\n".into() });
    assert_eq!(notes("notes owner greet"), out("notes of alux-cli-clap\n"));
}

#[test]
fn leaves_usage_errors_and_help_to_clap() {
    let cli = CliClap::new(Arc::<Store>::default(), alux_cli::CliProgramBuilder.notes_cli::<Store>());
    let notes = |line: &str| cli.run_blocking(line.split_whitespace());
    assert_eq!(first_line(&notes("notes add")), (2, "error: the following required arguments were not provided:"));
    assert_eq!(first_line(&notes("notes list -x")), (2, "error: unexpected argument '-x' found"));
    // A word `clap` accepts but the value does not decode from is reported in `clap`'s form.
    assert_eq!(first_line(&notes("notes list -n x")), (2, "error: invalid <limit>: invalid digit found in string"));
    assert!(notes("notes list -n x").stderr.contains("Usage: notes list"));
    let help = notes("notes list --help");
    assert_eq!(help.status, 0);
    assert!(
        help.stdout.starts_with("Lists notes.\n\nShows the first `limit` notes, numbered when verbose."),
        "{}",
        help.stdout
    );
    assert!(help.stdout.contains("-n, --limit <limit>"), "{}", help.stdout);
    assert!(
        notes("notes owner greet --help").stdout.contains("Who owns the notes [env: CARGO_PKG_NAME] [default: nobody]")
    );
    assert_eq!(notes("notes").status, 2);
}

#[test]
fn declares_global_arguments_where_their_context_begins() {
    let shelf = Arc::<Shelf>::default();
    let library = CliClap::new(Arc::clone(&shelf), alux_cli::CliProgramBuilder.library_cli::<Shelf, Store>());
    assert_eq!(library.run_blocking("notes add milk -t shop".split_whitespace()), out("1\n"));
    assert_eq!(library.run_blocking("notes add plan -s work".split_whitespace()), out("1\n"));
    let cli = CliClap::new(shelf, alux_cli::CliProgramBuilder.nested_cli::<Shelf, Store>());
    let notes = |line: &str| cli.run_blocking(line.split_whitespace());
    // Both contexts' global options, before or after the path.
    assert_eq!(notes("notes -s work list"), out("plan\n"));
    assert_eq!(notes("notes --only shop list"), out("milk #shop\n"));
    assert_eq!(notes("notes --only none list -s main"), out(""));
    assert_eq!(notes("notes owner --store work greet --owner bo"), out("notes of bo\n"));
    assert_eq!(
        notes("notes list -s home"),
        Exit { status: 4, stdout: String::new(), stderr: "error: no store `home`\n".into() }
    );
    assert!(notes("notes --help").stdout.contains("--store <store>"));
}

#[test]
fn rejects_two_alternatives_of_a_sum_as_clap_does() {
    let cli = CliClap::new(Arc::<Store>::default(), alux_cli::CliProgramBuilder.export_cli::<Store>());
    let export = |line: &str| cli.run_blocking(line.split_whitespace());
    assert_eq!(export("notes export -f out.txt"), out("0 notes to file out.txt\n"));
    assert_eq!(
        export("notes export --url https://x --user ana --password secret"),
        out("0 notes to url https://x as ana:6\n")
    );
    assert_eq!(first_line(&export("notes export --file a --url b")).0, 2);
    assert!(export("notes export --file a --url b").stderr.contains("cannot be used with"));
    assert_eq!(first_line(&export("notes export")), (2, "error: missing <target>"));
    assert_eq!(first_line(&export("notes export --stdout --user ana")), (2, "error: missing <password>"));
}

#[test]
fn dispatches_only_the_programs_commands_from_an_applications_matches() {
    let cli = CliClap::new(Arc::<Store>::default(), alux_cli::CliProgramBuilder.notes_cli::<Store>());
    let app = clap::Command::new("notes")
        .subcommand(clap::Command::new("version"))
        .subcommands(cli.command().get_subcommands().cloned());
    let dispatch = |line: &str| cli.dispatch_blocking(&app.clone().get_matches_from(line.split_whitespace()));
    assert_eq!(dispatch("notes add milk"), Some(out("1\n")));
    assert_eq!(dispatch("notes owner greet --owner ana"), Some(out("notes of ana\n")));
    assert_eq!(dispatch("notes version"), None);
}
