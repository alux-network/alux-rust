//! Runs a small notes application from command lines, end to end.

use alux_cli::{CliCommandsAlg, CliFailureAlg, CliNestAlg, cli};
use alux_cli_direct::{CliDirect, Exit};
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

fn notes(store: &Arc<Store>, words: &str) -> Exit {
    let program = alux_cli::CliProgramBuilder.notes_cli::<Store>();
    CliDirect::new(Arc::clone(store)).run_blocking(program, words.split_whitespace())
}

/// Waits for `running` on the calling thread, as any executor might.
fn wait(running: impl Future<Output = Exit>) -> Exit {
    let mut running = std::pin::pin!(running);
    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
    loop {
        if let std::task::Poll::Ready(exit) = running.as_mut().poll(&mut context) {
            return exit;
        }
        std::thread::yield_now();
    }
}

fn out(stdout: &str) -> Exit {
    Exit { status: 0, stdout: stdout.into(), stderr: String::new() }
}

#[test]
fn runs_commands_with_every_spelling() {
    let store = Arc::default();
    assert_eq!(notes(&store, "notes add milk"), out("1\n"));
    assert_eq!(notes(&store, "notes add bread --tag shop -ttoday"), out("2\n"));
    assert_eq!(notes(&store, "notes add -- -eggs"), out("3\n"));
    assert_eq!(notes(&store, "notes list"), out("milk\nbread #shop #today\n-eggs\n"));
    assert_eq!(notes(&store, "notes list -vn 2"), out("1. milk\n2. bread #shop #today\n"));
    assert_eq!(notes(&store, "notes list --limit=1 --verbose"), out("1. milk\n"));
    assert_eq!(notes(&store, "notes remove 1"), out(""));
    assert_eq!(notes(&store, "notes list"), out("bread #shop #today\n-eggs\n"));
}

#[test]
fn reads_short_values_as_getopt_does() {
    let store = Arc::default();
    assert_eq!(notes(&store, "notes add milk"), out("1\n"));
    assert_eq!(notes(&store, "notes add bread"), out("2\n"));
    assert_eq!(notes(&store, "notes add eggs -t: -t=: -tshop"), out("3\n"));
    // A letter that takes a value takes the rest of its word, after any switches before it.
    assert_eq!(notes(&store, "notes list -n1"), out("milk\n"));
    assert_eq!(notes(&store, "notes list -vn2"), out("1. milk\n2. bread\n"));
    assert_eq!(notes(&store, "notes list -vn 1"), out("1. milk\n"));
    // `=` is no separator for a short option: `-t=:` gives `=:`.
    assert_eq!(notes(&store, "notes list -n 3 -v").stdout.lines().last(), Some("3. eggs #: #=: #shop"));
}

#[test]
fn runs_command_lines_at_once_on_many_threads() {
    fn spawnable<Running: Future<Output = Exit> + Send + 'static>(running: Running) -> Running {
        running
    }
    let store = Arc::default();
    for note in ["milk", "bread", "eggs"] {
        assert_eq!(notes(&store, &format!("notes add {note}")).status, 0);
    }
    let cli = CliDirect::new(Arc::clone(&store));
    let running = (1..=8)
        .map(|limit| {
            let words = ["notes".to_owned(), "list".to_owned(), "-n".to_owned(), (limit % 3 + 1).to_string()];
            (limit % 3 + 1, spawnable(cli.run(alux_cli::CliProgramBuilder.notes_cli::<Store>(), words)))
        })
        .map(|(limit, running)| (limit, std::thread::spawn(move || wait(running))))
        .collect::<Vec<_>>();
    // Each command line runs on its own thread against the one shared store.
    for (limit, thread) in running {
        assert_eq!(thread.join().unwrap().stdout.lines().count(), limit);
    }
}

#[test]
fn a_failing_command_exits_with_its_status() {
    let store = Arc::default();
    assert_eq!(
        notes(&store, "notes remove 7"),
        Exit { status: 3, stdout: String::new(), stderr: "error: no note 7\n".into() }
    );
}

#[test]
fn ends_a_usage_error_with_the_status_it_is_given() {
    let program = || alux_cli::CliProgramBuilder.notes_cli::<Store>();
    let cli = CliDirect::new(Arc::<Store>::default()).usage_status(1);
    assert_eq!(cli.run_blocking(program(), ["notes", "list", "-x"]).status, 1);
    assert_eq!(cli.run_blocking(program(), ["notes", "pop"]).status, 1);
    assert_eq!(cli.run_blocking(program(), ["notes", "list"]).status, 0);
}

#[test]
fn runs_a_selected_command_as_among_every_command() {
    let store = Arc::<Store>::default();
    let run = |selected, words: &str| {
        CliDirect::new(Arc::clone(&store))
            .selecting(selected)
            .run_blocking(alux_cli::CliProgramBuilder.notes_cli::<Store>(), words.split_whitespace())
    };
    assert_eq!(run(Some("add"), "notes add milk"), out("1\n"));
    assert_eq!(run(Some("list"), "notes list -v"), run(None, "notes list -v"));
    // A nest's subtree compiles in full beneath its selected name.
    assert_eq!(run(Some("owner"), "notes owner greet --owner ana"), out("notes of ana\n"));
    // No other command is compiled, so none is selected.
    assert_eq!(run(Some("list"), "notes add bread").status, 2);
    // A context's subtree is selected alike: its first layer is the program's.
    let shelf = Arc::<Shelf>::default();
    let library = |selected, words: &str| {
        CliDirect::new(Arc::clone(&shelf))
            .selecting(selected)
            .run_blocking(alux_cli::CliProgramBuilder.library_cli::<Shelf, Store>(), words.split_whitespace())
    };
    assert_eq!(library(Some("add"), "notes add report -s work"), out("1\n"));
    assert_eq!(library(Some("list"), "notes -s work list"), out("report\n"));
}

#[test]
fn reads_the_environment_before_the_default() {
    // Cargo sets the package name for every test it runs.
    assert_eq!(notes(&Arc::default(), "notes owner greet"), out("notes of alux-cli-direct\n"));
    assert_eq!(notes(&Arc::default(), "notes owner greet --owner ana"), out("notes of ana\n"));
}

#[test]
fn rejects_words_that_do_not_make_the_command() {
    let store = Arc::default();
    let usage = |error: &str, usage: &str| Exit {
        status: 2,
        stdout: String::new(),
        stderr: format!("error: {error}\n\n{usage}"),
    };
    assert_eq!(notes(&store, "notes add"), usage("missing <text>", "Usage: notes add [options] <text>\n"));
    assert_eq!(notes(&store, "notes add a b"), usage("unexpected argument `b`", "Usage: notes add [options] <text>\n"));
    assert_eq!(notes(&store, "notes list -x"), usage("unknown option `-x`", "Usage: notes list [options]\n"));
    assert_eq!(notes(&store, "notes list -n"), usage("`-n` needs a value", "Usage: notes list [options]\n"));
    assert_eq!(
        notes(&store, "notes list --verbose=1"),
        usage("`--verbose` takes no value", "Usage: notes list [options]\n")
    );
    // As in `getopt`, `=` is no separator for a short option, so it is the next letter.
    assert_eq!(
        notes(&store, "notes list -v=1"),
        usage("unknown option `-=` in `-v=1`", "Usage: notes list [options]\n")
    );
    assert_eq!(
        notes(&store, "notes list -n x"),
        usage("invalid <limit>: invalid digit found in string", "Usage: notes list [options]\n")
    );
    assert_eq!(notes(&store, "notes remove 1 --verbose").status, 2);
    let unknown = notes(&store, "notes pop");
    assert_eq!((unknown.status, unknown.stderr.lines().next()), (2, Some("error: unknown command `pop`")));
}

#[test]
fn renders_help_from_the_program() {
    let store = Arc::default();
    assert_eq!(
        notes(&store, "notes --help").stdout,
        "Usage: notes <command>\n\
         \n\
         Commands:\n  \
           add          Adds a note, tagged, and answers its number.\n  \
           list         Lists notes.\n  \
           remove       Removes a note by its number.\n  \
           owner greet  Greets the owner of the notes.\n"
    );
    assert_eq!(
        notes(&store, "notes list -h").stdout,
        "Lists notes.\n\
         \n\
         Shows the first `limit` notes, numbered when verbose.\n\
         \n\
         Usage: notes list [options]\n\
         \n\
         Options:\n  \
           -n, --limit <limit>  Maximum number of notes to show\n  \
           -v, --verbose        Number the notes\n"
    );
    assert_eq!(
        notes(&store, "notes owner greet --help").stdout,
        "Greets the owner of the notes.\n\
         \n\
         Usage: notes owner greet [options]\n\
         \n\
         Options:\n      \
               --owner <owner>  Who owns the notes [env: CARGO_PKG_NAME] [default: nobody]\n"
    );
    // A bare command line asks for a command, so it is a usage error that shows the help.
    let bare = notes(&store, "notes");
    assert_eq!((bare.status, bare.stderr.starts_with("Usage: notes <command>")), (2, true));
}

fn library(shelf: &Arc<Shelf>, words: &str) -> Exit {
    let program = alux_cli::CliProgramBuilder.library_cli::<Shelf, Store>();
    CliDirect::new(Arc::clone(shelf)).run_blocking(program, words.split_whitespace())
}

#[test]
fn applies_every_command_to_the_context_its_global_arguments_open() {
    let shelf = Arc::default();
    assert_eq!(library(&shelf, "notes add milk"), out("1\n"));
    assert_eq!(library(&shelf, "notes add report --store work"), out("1\n"));
    assert_eq!(library(&shelf, "notes add slides -s work"), out("2\n"));
    // Every command of the subtree takes the global argument, a nested one included.
    assert_eq!(library(&shelf, "notes list -s work"), out("report\nslides\n"));
    assert_eq!(library(&shelf, "notes list"), out("milk\n"));
    assert_eq!(library(&shelf, "notes owner greet -s work --owner ana"), out("notes of ana\n"));
    // As in `clap`, a global option can also come before the command path.
    assert_eq!(library(&shelf, "notes -s work list"), out("report\nslides\n"));
    assert_eq!(library(&shelf, "notes -swork list"), out("report\nslides\n"));
    assert_eq!(library(&shelf, "notes --store=work owner greet --owner ana"), out("notes of ana\n"));
    assert_eq!(library(&shelf, "notes owner --store work greet --owner bo"), out("notes of bo\n"));
    let unknown = library(&shelf, "notes --nope list");
    assert_eq!((unknown.status, unknown.stderr.lines().next()), (2, Some("error: unknown option `--nope`")));
    // A context that fails ends the run with its own status, and applies no command.
    assert_eq!(
        library(&shelf, "notes add lost -s home"),
        Exit { status: 4, stdout: String::new(), stderr: "error: no store `home`\n".into() }
    );
    assert_eq!(library(&shelf, "notes list -s main"), out("milk\n"));
    // Words that do not make the command are reported before the context is opened.
    assert_eq!(library(&shelf, "notes add -s home").status, 2);
    assert_eq!(
        library(&shelf, "notes list --help").stdout,
        "Lists notes.\n\
         \n\
         Shows the first `limit` notes, numbered when verbose.\n\
         \n\
         Usage: notes list [options]\n\
         \n\
         Options:\n  \
           -n, --limit <limit>  Maximum number of notes to show\n  \
           -v, --verbose        Number the notes\n\
         \n\
         Global options:\n  \
           -s, --store <store>  The store to use [default: main]\n"
    );
}

fn run_cli<Program>(shelf: &Arc<Shelf>, program: Program, words: &str) -> Exit
where
    Program: alux_cli::CliProgramAlg<
            alux_cli_direct::DirectCompiler<Arc<Shelf>>,
            Commands = Vec<alux_cli_direct::Entry<Arc<Shelf>>>,
        >,
{
    CliDirect::new(Arc::clone(shelf)).run_blocking(program, words.split_whitespace())
}

#[test]
fn nests_contexts_each_opening_from_the_one_around_it() {
    let shelf = Arc::default();
    let builder = alux_cli::CliProgramBuilder;
    for line in ["notes add milk -t shop", "notes add mail", "notes add plan -s work -t shop"] {
        assert_eq!(library(&shelf, line).status, 0);
    }
    let nested = |line| run_cli(&shelf, builder.nested_cli::<Shelf, Store>(), line);
    assert_eq!(nested("notes list"), out("milk #shop\nmail\n"));
    assert_eq!(nested("notes list --only shop"), out("milk #shop\n"));
    // Both contexts' global options, in any order and on either side of the path.
    assert_eq!(nested("notes --only shop list -s work"), out("plan #shop\n"));
    assert_eq!(nested("notes list --only none"), out(""));
    // The outer context still fails first, before the inner one is opened.
    assert_eq!(nested("notes list -s home --only shop").status, 4);
    // Help lists the global options outermost first.
    assert!(
        nested("notes list --help").stdout.ends_with(
            "Global options:\n  \
               -s, --store <store>  The store to use [default: main]\n      \
                   --only <only>    Only notes with this tag\n"
        ),
        "{}",
        nested("notes list --help").stdout
    );
}

#[test]
fn applies_a_total_context_that_takes_no_argument() {
    let shelf = Arc::default();
    let main = |line| run_cli(&shelf, alux_cli::CliProgramBuilder.main_cli::<Shelf, Store>(), line);
    assert_eq!(main("notes add milk"), out("1\n"));
    assert_eq!(library(&shelf, "notes list -s main"), out("milk\n"));
    assert!(!main("notes list --help").stdout.contains("Global options"));
}

#[test]
fn lists_the_global_options_every_command_shares_in_the_programs_help() {
    let help = library(&Arc::default(), "notes --help").stdout;
    assert!(help.starts_with("Usage: notes [options] <command>\n"), "{help}");
    assert!(help.ends_with("Global options:\n  -s, --store <store>  The store to use [default: main]\n"), "{help}");
}

#[test]
fn reads_one_alternative_of_a_sum_and_a_whole_product() {
    let store = Arc::new(Store::default());
    let export = |line: &str| {
        CliDirect::new(Arc::clone(&store))
            .run_blocking(alux_cli::CliProgramBuilder.export_cli::<Store>(), line.split_whitespace())
    };
    assert_eq!(export("notes export --file out.txt"), out("0 notes to file out.txt\n"));
    assert_eq!(export("notes export -f out.txt"), out("0 notes to file out.txt\n"));
    assert_eq!(export("notes export --stdout"), out("0 notes to stdout\n"));
    assert_eq!(
        export("notes export --url https://x --user ana --password secret"),
        out("0 notes to url https://x as ana:6\n")
    );
    let usage = |error: &str| Exit {
        status: 2,
        stdout: String::new(),
        stderr: format!("error: {error}\n\nUsage: notes export [options]\n"),
    };
    // Exactly one alternative: two conflict, and none is missing.
    assert_eq!(export("notes export --file a --url b"), usage("<file> and <url> cannot be given together"));
    assert_eq!(export("notes export --stdout --file a"), usage("<file> and <stdout> cannot be given together"));
    assert_eq!(export("notes export"), usage("missing <target>"));
    // An optional product takes all of its fields or none of them.
    assert_eq!(export("notes export --stdout --user ana"), usage("missing <password>"));
    assert_eq!(
        export("notes export --help").stdout,
        "Exports the notes to one target, signed in or not.\n\
         \n\
         Usage: notes export [options]\n\
         \n\
         Options:\n  \
           -f, --file <file>          Write to a file [one of <target>]\n      \
               --url <url>            Send to a URL [one of <target>]\n      \
               --stdout               Print the notes [one of <target>]\n      \
               --user <user>          User to sign in as [with <auth>]\n      \
               --password <password>  Password to sign in with [with <auth>]\n"
    );
}
