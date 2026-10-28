//! Declares commands whose operations borrow their arguments.

use alux_cli::{CliCommandsAlg, CliProgramBuilder, cli};
use alux_ext::{ApplyAlg, OperationAlg, ext};

trait Bytes {
    fn bytes(&self) -> &[u8];
}

#[ext(name = BytesExt, defunc)]
impl<This> This
where
    This: Bytes + ?Sized,
{
    /// Compares the bytes with borrowed ones.
    fn cmp_bytes(&self, other: &[u8]) -> bool {
        self.bytes() == other
    }
}

#[ext(name = TextCommandsExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    fn text_commands<Word>(&self) {
        self.commands().command("cmp", self.op(Word::cmp_bytes))
    }
}

struct Word(Vec<u8>);

impl Bytes for Word {
    fn bytes(&self) -> &[u8] {
        &self.0
    }
}

#[test]
fn names_a_borrowing_operation_like_any_other() {
    let _program = CliProgramBuilder.text_commands::<Word>();
    assert_eq!(<CmpBytesOperation<Word> as OperationAlg>::ARG_NAMES, &["other"]);
    let word = Word(b"abc".to_vec());
    let other = b"abc".to_vec();
    let cmp = CmpBytesOperation::<Word>::default();
    assert!(futures_block(cmp.apply(&word, (&other[..],))));
}

fn futures_block<Output>(future: impl Future<Output = Output>) -> Output {
    let mut future = std::pin::pin!(future);
    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
    loop {
        if let std::task::Poll::Ready(output) = future.as_mut().poll(&mut context) {
            return output;
        }
    }
}
