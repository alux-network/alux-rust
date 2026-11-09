//! The type of one operation argument, by its position.
//!
//! A declaration states one source per operation argument, in order; reading each argument's type
//! by position lets a lowering check every source against its own argument, where it is written.

use crate::{CliSourceFor, Optional, ProductSource, SumSource, Whole};
use alux_ext::for_tuple_positions;
use alux_ext::{ContextArg, StatedArg};
use alux_shape::{Product, Sum};

/// Says whether the argument at `position` among `names` is named `name`, at compile time.
#[must_use]
pub const fn cli_arg_named(names: &[&str], position: usize, name: &str) -> bool {
    if position >= names.len() {
        return false;
    }
    let (own, name) = (names[position].as_bytes(), name.as_bytes());
    if own.len() != name.len() {
        return false;
    }
    let mut at = 0;
    while at < own.len() {
        if own[at] != name[at] {
            return false;
        }
        at += 1;
    }
    true
}

/// Holds the message a misnamed argument is reported with, written at compile time.
#[derive(Clone, Copy, Debug)]
pub struct CliArgMessage {
    bytes: [u8; 256],
    len: usize,
}

impl CliArgMessage {
    /// Whether there is nothing to report.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Reads the message.
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match core::str::from_utf8(self.bytes.split_at(self.len).0) {
            Ok(message) => message,
            Err(_) => "an argument is not the one `.arg(..)` names",
        }
    }

    const fn push(mut self, text: &str) -> Self {
        let text = text.as_bytes();
        let mut at = 0;
        while at < text.len() && self.len < self.bytes.len() {
            self.bytes[self.len] = text[at];
            self.len += 1;
            at += 1;
        }
        self
    }

    const fn push_number(self, number: usize) -> Self {
        let mut digits = [0u8; 20];
        let (mut count, mut rest) = (0, number);
        loop {
            digits[19 - count] = b"0123456789"[rest % 10];
            count += 1;
            rest /= 10;
            if rest == 0 {
                break;
            }
        }
        match core::str::from_utf8(digits.split_at(20 - count).1) {
            Ok(digits) => self.push(digits),
            Err(_) => self,
        }
    }
}

/// Answers why the argument at `position` of `method` is not named `name`, or an empty message when
/// it is, at compile time: "argument 0 of `start_node` is `name`, not `fff`".
#[must_use]
pub const fn cli_arg_mismatch(method: &str, names: &[&str], position: usize, name: &str) -> CliArgMessage {
    let message = CliArgMessage { bytes: [0; 256], len: 0 };
    if cli_arg_named(names, position, name) {
        return message;
    }
    if position >= names.len() {
        return message
            .push("`")
            .push(method)
            .push("` takes ")
            .push_number(names.len())
            .push(" arguments, so `")
            .push(name)
            .push("` has no source to bind");
    }
    message
        .push("argument ")
        .push_number(position)
        .push(" of `")
        .push(method)
        .push("` is `")
        .push(names[position])
        .push("`, not `")
        .push(name)
        .push("`: sources bind to the operation's arguments in order")
}

/// States that a source can supply the argument at position `INDEX` of the argument product `Args`.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot supply argument {INDEX} of the operation",
    note = "sources bind to the operation's arguments in order, one `.arg(..)` each; check the order of the `.arg(..)` calls"
)]
pub trait CliSourceAt<Args, const INDEX: usize> {}

impl<Source, Args, const INDEX: usize> CliSourceAt<Args, INDEX> for Source
where
    Args: CliArgAt<INDEX>,
    Source: CliSourceFor<Args::Arg>,
{
}

/// Names the type of the argument at position `INDEX` of an operation's argument product.
pub trait CliArgAt<const INDEX: usize> {
    /// The argument's type.
    type Arg;
}

/// States `CliArgAt` for one position of one width.
macro_rules! cli_arg_at {
    ($at:tt => $arg:ident; $($all:ident => $index:tt),+) => {
        impl<$($all),+> CliArgAt<$at> for ($($all,)+) {
            type Arg = $arg;
        }
    };
}

for_tuple_positions!(cli_arg_at, 1..=16);

/// States that a source can supply an argument as its method states it: a stated type is checked,
/// one the context decides is left to the interpretation.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot supply this argument as its method states it",
    note = "sources bind to the operation's arguments in order, one `.arg(..)` each; check the order of the `.arg(..)` calls"
)]
pub trait CliStatedFor<Stated> {}

impl<Source, Arg> CliStatedFor<StatedArg<Arg>> for Source where Source: CliSourceFor<Arg> {}

impl<Source> CliStatedFor<ContextArg> for Source {}

/// Names the parts a compound source supplies an argument's parts to, as the argument's type names
/// them: a product's fields or a sum's alternatives; none for an argument the context decides.
pub trait CliPartsFor<Stated> {
    /// The argument's part names, in order.
    const PARTS: &'static [&'static str];
    /// What one part is called: `field` or `alternative`.
    const PART: &'static str;
    /// What the whole is called: `product` or `sum`.
    const WHOLE: &'static str;
}

impl<Fields, Value> CliPartsFor<StatedArg<Value>> for ProductSource<Fields, Whole>
where
    Value: Product,
{
    const PARTS: &'static [&'static str] = Value::FIELDS;
    const PART: &'static str = "field";
    const WHOLE: &'static str = "product";
}

impl<Fields, Value> CliPartsFor<StatedArg<Option<Value>>> for ProductSource<Fields, Optional>
where
    Value: Product,
{
    const PARTS: &'static [&'static str] = Value::FIELDS;
    const PART: &'static str = "field";
    const WHOLE: &'static str = "product";
}

impl<Alternatives, Value> CliPartsFor<StatedArg<Value>> for SumSource<Alternatives, Whole>
where
    Value: Sum,
{
    const PARTS: &'static [&'static str] = Value::ALTERNATIVES;
    const PART: &'static str = "alternative";
    const WHOLE: &'static str = "sum";
}

impl<Alternatives, Value> CliPartsFor<StatedArg<Option<Value>>> for SumSource<Alternatives, Optional>
where
    Value: Sum,
{
    const PARTS: &'static [&'static str] = Value::ALTERNATIVES;
    const PART: &'static str = "alternative";
    const WHOLE: &'static str = "sum";
}

impl<Source> CliPartsFor<ContextArg> for Source {
    const PARTS: &'static [&'static str] = &[];
    const PART: &'static str = "part";
    const WHOLE: &'static str = "argument";
}

/// Answers why the part at `position` of the argument `stated` states, which `Source` supplies, is
/// not named `name`, or an empty message when it is or when no names are known, at compile time:
/// "field 1 of the product is `port`, not `addr`".
#[must_use]
pub const fn cli_part_mismatch<Source, Stated>(_stated: &Stated, position: usize, name: &str) -> CliArgMessage
where
    Source: CliPartsFor<Stated>,
{
    let (names, part, whole) = (Source::PARTS, Source::PART, Source::WHOLE);
    let message = CliArgMessage { bytes: [0; 256], len: 0 };
    if names.is_empty() || cli_arg_named(names, position, name) {
        return message;
    }
    if position >= names.len() {
        return message
            .push("the ")
            .push(whole)
            .push(" has ")
            .push_number(names.len())
            .push(" ")
            .push(part)
            .push("s, so `")
            .push(name)
            .push("` has no ")
            .push(part)
            .push(" to bind");
    }
    message
        .push(part)
        .push(" ")
        .push_number(position)
        .push(" of the ")
        .push(whole)
        .push(" is `")
        .push(names[position])
        .push("`, not `")
        .push(name)
        .push("`: sources bind to the ")
        .push(whole)
        .push("'s ")
        .push(part)
        .push("s in order")
}

/// Checks, at compile time, that `Source` can supply the argument `stated` states.
pub const fn cli_source_stated<Source, Stated>(_stated: &Stated)
where
    Source: CliStatedFor<Stated>,
{
}
