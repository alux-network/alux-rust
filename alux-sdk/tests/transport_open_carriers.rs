//! An algebra whose carriers are still open is stated for every choice of them at once.
//!
//! Each associated type becomes a parameter of the impl, and the impl defines it as that parameter.
//! The capability form and the proxy are headed by a type parameter, so they read the carriers off
//! [`AlgebraCarrier`]; the carrier form reads them off the carrier's own type. Whoever holds the
//! algebra names the carriers it reads in, as `Registry<Peer = u32, Record = String>`, and nothing
//! of what carries it.

use alux_sdk::{AlgebraCall, AlgebraCarrier, AlgebraSend, trait_algebra};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A peer is carried by operations and a record by replies, so `RegistryOp<Peer>` and
/// `RegistryReply<Record>` are what a carrier carries.
#[trait_algebra(transport)]
trait Registry {
    /// Who is admitted.
    type Peer: Clone + Send;
    /// What is kept for one.
    type Record: Send;
    /// States a value, so a caller asks for it.
    async fn admit(&self, peer: Self::Peer) -> usize;
    /// States a value carrying a carrier of its own.
    async fn record_of(&self, peer: Self::Peer) -> Option<Self::Record>;
    /// States none, so a caller sends and does not stay.
    async fn forget(&self, peer: Self::Peer);
}

/// Keeps one record per admitted peer.
#[derive(Default)]
struct Book {
    records: HashMap<u32, String>,
}

impl RegistryInterpreter for Book {
    type Peer = u32;
    type Record = String;

    async fn admit(&mut self, peer: u32) -> usize {
        self.records.insert(peer, format!("peer {peer}"));
        self.records.len()
    }

    async fn record_of(&mut self, peer: u32) -> Option<String> {
        self.records.get(&peer).cloned()
    }

    async fn forget(&mut self, peer: u32) {
        self.records.remove(&peer);
    }
}

/// A carrier that interprets where it stands, carrying the registry of `u32` peers and `String`
/// records.
struct AtHand {
    book: Arc<Mutex<Book>>,
}

impl AlgebraCarrier for AtHand {
    type Operation = RegistryOp<u32>;
    type Reply = RegistryReply<String>;
}

impl AlgebraCall<RegistryOp<u32>, RegistryReply<String>> for AtHand {
    async fn ask(&self, operation: RegistryOp<u32>) -> Option<RegistryReply<String>> {
        let mut book = self.book.lock().await;

        Some(operation.interpret(&mut *book).await)
    }
}

impl AlgebraSend<RegistryOp<u32>> for AtHand {
    async fn send(&self, operation: RegistryOp<u32>) -> Option<()> {
        self.ask(operation).await.map(|_stated| ())
    }
}

/// Reads the algebra, naming the carriers it reads in and nothing of what carries it.
async fn admit_through(registry: impl Registry<Peer = u32, Record = String>) -> (usize, Option<String>) {
    let admitted = registry.admit(7).await;
    let record = registry.record_of(7).await;
    registry.forget(7).await;

    (admitted, record)
}

#[tokio::test]
async fn every_witness_of_the_carried_algebra_is_the_algebra() {
    let book = Arc::new(Mutex::new(Book::default()));
    let at_hand = Arc::new(AtHand { book: Arc::clone(&book) });

    assert_eq!(admit_through(AtHand { book: Arc::clone(&book) }).await, (1, Some("peer 7".to_owned())));
    assert_eq!(admit_through(&*at_hand).await, (1, Some("peer 7".to_owned())));
    assert_eq!(admit_through(Arc::clone(&at_hand)).await, (1, Some("peer 7".to_owned())));
    assert!(book.lock().await.records.is_empty());
}

#[tokio::test]
async fn the_carried_algebra_can_be_waited_on_elsewhere() {
    let at_hand = Arc::new(AtHand { book: Arc::default() });

    let admitted = tokio::spawn(async move { admit_through(at_hand).await });

    assert_eq!(admitted.await.expect("the task admitting is done"), (1, Some("peer 7".to_owned())));
}

/// The carrier form names one carrier, whose own parameters fix the algebra's.
///
/// `Entry` is bounded by nothing: `Kept` asks only for entries that are `Send`, and the impl is
/// conditioned on what `Kept` witnesses rather than on a bound the algebra would have to repeat.
#[trait_algebra(transport = Kept)]
trait Ledger {
    /// What is entered.
    type Entry;
    /// States a value carrying the carrier.
    async fn enter(&self, entry: Self::Entry) -> Self::Entry;
}

/// Holds nothing and states each entry back, for whatever entries it is built for.
struct Kept<Operation, Reply> {
    stated: PhantomData<fn(Operation) -> Reply>,
}

/// Interprets entering as stating the entry back.
struct Echo<Entry>(PhantomData<Entry>);

impl<Entry> LedgerInterpreter for Echo<Entry> {
    type Entry = Entry;

    async fn enter(&mut self, entry: Entry) -> Entry {
        entry
    }
}

impl<Entry> AlgebraCall<LedgerOp<Entry>, LedgerReply<Entry>> for Kept<LedgerOp<Entry>, LedgerReply<Entry>>
where
    Entry: Send,
{
    async fn ask(&self, operation: LedgerOp<Entry>) -> Option<LedgerReply<Entry>> {
        Some(operation.interpret(&mut Echo(PhantomData)).await)
    }
}

#[tokio::test]
async fn the_named_carrier_states_the_algebra_for_each_choice_of_its_carriers() {
    let numbers = Kept::<LedgerOp<u64>, LedgerReply<u64>> { stated: PhantomData };
    let words = Kept::<LedgerOp<&str>, LedgerReply<&str>> { stated: PhantomData };

    assert_eq!(numbers.enter(42).await, 42);
    assert_eq!(words.enter("entered").await, "entered");
}

/// The proxy holds a caller, and reads the carriers off what that caller carries.
///
/// The bound on `Name` is stated in the trait's where clause, which every impl restates.
#[trait_algebra(proxy)]
trait Directory
where
    Self::Name: Send,
{
    /// Who is looked up.
    type Name;
    /// States a value, so the proxy asks for it.
    async fn look_up(&self, name: Self::Name) -> usize;
}

/// Interprets looking up as counting the name's bytes.
struct Lengths;

impl DirectoryInterpreter for Lengths {
    type Name = String;

    async fn look_up(&mut self, name: String) -> usize {
        name.len()
    }
}

/// A caller carrying the directory of `String` names.
struct Here;

impl AlgebraCarrier for Here {
    type Operation = DirectoryOp<String>;
    type Reply = DirectoryReply;
}

impl AlgebraCall<DirectoryOp<String>, DirectoryReply> for Here {
    async fn ask(&self, operation: DirectoryOp<String>) -> Option<DirectoryReply> {
        Some(operation.interpret(&mut Lengths).await)
    }
}

/// Reads the algebra, naming only the carrier it reads in.
async fn look_up_through(directory: &impl Directory<Name = String>) -> usize {
    directory.look_up("alux".to_owned()).await
}

#[tokio::test]
async fn a_proxy_states_the_algebra_its_caller_carries() {
    assert_eq!(look_up_through(&DirectoryProxy::new(Here)).await, 4);
    assert_eq!(look_up_through(&DirectoryProxy::new(Arc::new(Here))).await, 4);
}

/// The impl's parameters are named apart from every name the trait states.
///
/// Each carrier here is bounded by `Error`, the trait imported under the carrier's own name, and
/// is reached as `<Self as Trait>::Error` as well as `Self::Error`: every impl still reads `Error`
/// as that trait and restates the qualified predicate on its parameter.
mod named_apart {
    use alux_sdk::{AlgebraCall, AlgebraCarrier, trait_algebra};
    use std::error::Error;
    use std::fmt;
    use std::marker::PhantomData;

    /// What is refused, by its code.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Refused(u8);

    impl fmt::Display for Refused {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "refused {}", self.0)
        }
    }

    impl Error for Refused {}

    /// States a refusal, and asks of it through the trait's own qualified predicate.
    #[trait_algebra(transport)]
    trait Fallible
    where
        <Self as Fallible>::Error: Clone + Send,
    {
        /// What is refused.
        type Error: Error;
        /// States the refusal of one code.
        async fn fail(&self, code: u8) -> Self::Error;
    }

    /// Interprets failing as refusing the code.
    struct Refusing;

    impl FallibleInterpreter for Refusing {
        type Error = Refused;

        async fn fail(&mut self, code: u8) -> Refused {
            Refused(code)
        }
    }

    /// A carrier that interprets where it stands.
    struct AtHand;

    impl AlgebraCarrier for AtHand {
        type Operation = FallibleOp;
        type Reply = FallibleReply<Refused>;
    }

    impl AlgebraCall<FallibleOp, FallibleReply<Refused>> for AtHand {
        async fn ask(&self, operation: FallibleOp) -> Option<FallibleReply<Refused>> {
            Some(operation.interpret(&mut Refusing).await)
        }
    }

    #[tokio::test]
    async fn the_capability_impl_reads_the_imported_trait_and_the_qualified_predicate() {
        assert_eq!(AtHand.fail(3).await, Refused(3));
    }

    /// Names one carrier, and carries a refusal reached by its qualified projection.
    #[trait_algebra(transport = Kept)]
    trait Reported {
        /// What is reported.
        type Error: Error;
        /// States the length of one report.
        async fn report(&self, error: <Self as Reported>::Error) -> usize;
    }

    /// Holds nothing and measures each report, for whatever reports it is built for.
    struct Kept<Operation, Reply> {
        stated: PhantomData<fn(Operation) -> Reply>,
    }

    /// Interprets reporting as measuring the report.
    struct Measuring<Reporting>(PhantomData<Reporting>);

    impl<Reporting> ReportedInterpreter for Measuring<Reporting>
    where
        Reporting: Error,
    {
        type Error = Reporting;

        async fn report(&mut self, error: Reporting) -> usize {
            error.to_string().len()
        }
    }

    impl<Reporting> AlgebraCall<ReportedOp<Reporting>, ReportedReply> for Kept<ReportedOp<Reporting>, ReportedReply>
    where
        Reporting: Error + Send,
    {
        async fn ask(&self, operation: ReportedOp<Reporting>) -> Option<ReportedReply> {
            Some(operation.interpret(&mut Measuring(PhantomData)).await)
        }
    }

    #[tokio::test]
    async fn the_carrier_impl_reads_the_imported_trait_and_the_qualified_carrier() {
        let kept = Kept::<ReportedOp<Refused>, ReportedReply> { stated: PhantomData };

        assert_eq!(kept.report(Refused(7)).await, "refused 7".len());
    }

    /// The proxy states the same algebra for whatever its caller carries.
    #[trait_algebra(proxy)]
    trait Excused {
        /// What is excused.
        type Error: Error;
        /// States the length of one excuse.
        async fn excuse(&self, error: <Self as Excused>::Error) -> usize;
    }

    impl ExcusedInterpreter for Measuring<Refused> {
        type Error = Refused;

        async fn excuse(&mut self, error: Refused) -> usize {
            error.to_string().len()
        }
    }

    /// A caller carrying excuses of `Refused`.
    struct Here;

    impl AlgebraCarrier for Here {
        type Operation = ExcusedOp<Refused>;
        type Reply = ExcusedReply;
    }

    impl AlgebraCall<ExcusedOp<Refused>, ExcusedReply> for Here {
        async fn ask(&self, operation: ExcusedOp<Refused>) -> Option<ExcusedReply> {
            Some(operation.interpret(&mut Measuring(PhantomData)).await)
        }
    }

    #[tokio::test]
    async fn the_proxy_reads_the_imported_trait_and_the_qualified_carrier() {
        assert_eq!(ExcusedProxy::new(Here).excuse(Refused(9)).await, "refused 9".len());
    }
}
