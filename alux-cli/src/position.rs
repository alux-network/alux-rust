//! The type of one operation argument, by its position.
//!
//! A declaration states one source per operation argument, in order; reading each argument's type
//! by position lets a lowering check every source against its own argument, where it is written.

use crate::CliSourceFor;
use alux_ext::{ContextArg, StatedArg};

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
            Err(_) => "an argument is not the one `.arg_for(..)` names",
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

impl<A0> CliArgAt<0> for (A0,) {
    type Arg = A0;
}

impl<A0, A1> CliArgAt<0> for (A0, A1) {
    type Arg = A0;
}

impl<A0, A1> CliArgAt<1> for (A0, A1) {
    type Arg = A1;
}

impl<A0, A1, A2> CliArgAt<0> for (A0, A1, A2) {
    type Arg = A0;
}

impl<A0, A1, A2> CliArgAt<1> for (A0, A1, A2) {
    type Arg = A1;
}

impl<A0, A1, A2> CliArgAt<2> for (A0, A1, A2) {
    type Arg = A2;
}

impl<A0, A1, A2, A3> CliArgAt<0> for (A0, A1, A2, A3) {
    type Arg = A0;
}

impl<A0, A1, A2, A3> CliArgAt<1> for (A0, A1, A2, A3) {
    type Arg = A1;
}

impl<A0, A1, A2, A3> CliArgAt<2> for (A0, A1, A2, A3) {
    type Arg = A2;
}

impl<A0, A1, A2, A3> CliArgAt<3> for (A0, A1, A2, A3) {
    type Arg = A3;
}

impl<A0, A1, A2, A3, A4> CliArgAt<0> for (A0, A1, A2, A3, A4) {
    type Arg = A0;
}

impl<A0, A1, A2, A3, A4> CliArgAt<1> for (A0, A1, A2, A3, A4) {
    type Arg = A1;
}

impl<A0, A1, A2, A3, A4> CliArgAt<2> for (A0, A1, A2, A3, A4) {
    type Arg = A2;
}

impl<A0, A1, A2, A3, A4> CliArgAt<3> for (A0, A1, A2, A3, A4) {
    type Arg = A3;
}

impl<A0, A1, A2, A3, A4> CliArgAt<4> for (A0, A1, A2, A3, A4) {
    type Arg = A4;
}

impl<A0, A1, A2, A3, A4, A5> CliArgAt<0> for (A0, A1, A2, A3, A4, A5) {
    type Arg = A0;
}

impl<A0, A1, A2, A3, A4, A5> CliArgAt<1> for (A0, A1, A2, A3, A4, A5) {
    type Arg = A1;
}

impl<A0, A1, A2, A3, A4, A5> CliArgAt<2> for (A0, A1, A2, A3, A4, A5) {
    type Arg = A2;
}

impl<A0, A1, A2, A3, A4, A5> CliArgAt<3> for (A0, A1, A2, A3, A4, A5) {
    type Arg = A3;
}

impl<A0, A1, A2, A3, A4, A5> CliArgAt<4> for (A0, A1, A2, A3, A4, A5) {
    type Arg = A4;
}

impl<A0, A1, A2, A3, A4, A5> CliArgAt<5> for (A0, A1, A2, A3, A4, A5) {
    type Arg = A5;
}

impl<A0, A1, A2, A3, A4, A5, A6> CliArgAt<0> for (A0, A1, A2, A3, A4, A5, A6) {
    type Arg = A0;
}

impl<A0, A1, A2, A3, A4, A5, A6> CliArgAt<1> for (A0, A1, A2, A3, A4, A5, A6) {
    type Arg = A1;
}

impl<A0, A1, A2, A3, A4, A5, A6> CliArgAt<2> for (A0, A1, A2, A3, A4, A5, A6) {
    type Arg = A2;
}

impl<A0, A1, A2, A3, A4, A5, A6> CliArgAt<3> for (A0, A1, A2, A3, A4, A5, A6) {
    type Arg = A3;
}

impl<A0, A1, A2, A3, A4, A5, A6> CliArgAt<4> for (A0, A1, A2, A3, A4, A5, A6) {
    type Arg = A4;
}

impl<A0, A1, A2, A3, A4, A5, A6> CliArgAt<5> for (A0, A1, A2, A3, A4, A5, A6) {
    type Arg = A5;
}

impl<A0, A1, A2, A3, A4, A5, A6> CliArgAt<6> for (A0, A1, A2, A3, A4, A5, A6) {
    type Arg = A6;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7> CliArgAt<0> for (A0, A1, A2, A3, A4, A5, A6, A7) {
    type Arg = A0;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7> CliArgAt<1> for (A0, A1, A2, A3, A4, A5, A6, A7) {
    type Arg = A1;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7> CliArgAt<2> for (A0, A1, A2, A3, A4, A5, A6, A7) {
    type Arg = A2;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7> CliArgAt<3> for (A0, A1, A2, A3, A4, A5, A6, A7) {
    type Arg = A3;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7> CliArgAt<4> for (A0, A1, A2, A3, A4, A5, A6, A7) {
    type Arg = A4;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7> CliArgAt<5> for (A0, A1, A2, A3, A4, A5, A6, A7) {
    type Arg = A5;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7> CliArgAt<6> for (A0, A1, A2, A3, A4, A5, A6, A7) {
    type Arg = A6;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7> CliArgAt<7> for (A0, A1, A2, A3, A4, A5, A6, A7) {
    type Arg = A7;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8> CliArgAt<0> for (A0, A1, A2, A3, A4, A5, A6, A7, A8) {
    type Arg = A0;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8> CliArgAt<1> for (A0, A1, A2, A3, A4, A5, A6, A7, A8) {
    type Arg = A1;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8> CliArgAt<2> for (A0, A1, A2, A3, A4, A5, A6, A7, A8) {
    type Arg = A2;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8> CliArgAt<3> for (A0, A1, A2, A3, A4, A5, A6, A7, A8) {
    type Arg = A3;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8> CliArgAt<4> for (A0, A1, A2, A3, A4, A5, A6, A7, A8) {
    type Arg = A4;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8> CliArgAt<5> for (A0, A1, A2, A3, A4, A5, A6, A7, A8) {
    type Arg = A5;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8> CliArgAt<6> for (A0, A1, A2, A3, A4, A5, A6, A7, A8) {
    type Arg = A6;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8> CliArgAt<7> for (A0, A1, A2, A3, A4, A5, A6, A7, A8) {
    type Arg = A7;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8> CliArgAt<8> for (A0, A1, A2, A3, A4, A5, A6, A7, A8) {
    type Arg = A8;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9> CliArgAt<0> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) {
    type Arg = A0;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9> CliArgAt<1> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) {
    type Arg = A1;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9> CliArgAt<2> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) {
    type Arg = A2;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9> CliArgAt<3> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) {
    type Arg = A3;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9> CliArgAt<4> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) {
    type Arg = A4;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9> CliArgAt<5> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) {
    type Arg = A5;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9> CliArgAt<6> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) {
    type Arg = A6;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9> CliArgAt<7> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) {
    type Arg = A7;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9> CliArgAt<8> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) {
    type Arg = A8;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9> CliArgAt<9> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) {
    type Arg = A9;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10> CliArgAt<0> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) {
    type Arg = A0;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10> CliArgAt<1> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) {
    type Arg = A1;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10> CliArgAt<2> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) {
    type Arg = A2;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10> CliArgAt<3> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) {
    type Arg = A3;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10> CliArgAt<4> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) {
    type Arg = A4;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10> CliArgAt<5> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) {
    type Arg = A5;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10> CliArgAt<6> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) {
    type Arg = A6;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10> CliArgAt<7> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) {
    type Arg = A7;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10> CliArgAt<8> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) {
    type Arg = A8;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10> CliArgAt<9> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) {
    type Arg = A9;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10> CliArgAt<10> for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) {
    type Arg = A10;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11> CliArgAt<0>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)
{
    type Arg = A0;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11> CliArgAt<1>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)
{
    type Arg = A1;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11> CliArgAt<2>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)
{
    type Arg = A2;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11> CliArgAt<3>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)
{
    type Arg = A3;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11> CliArgAt<4>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)
{
    type Arg = A4;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11> CliArgAt<5>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)
{
    type Arg = A5;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11> CliArgAt<6>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)
{
    type Arg = A6;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11> CliArgAt<7>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)
{
    type Arg = A7;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11> CliArgAt<8>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)
{
    type Arg = A8;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11> CliArgAt<9>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)
{
    type Arg = A9;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11> CliArgAt<10>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)
{
    type Arg = A10;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11> CliArgAt<11>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)
{
    type Arg = A11;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12> CliArgAt<0>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)
{
    type Arg = A0;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12> CliArgAt<1>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)
{
    type Arg = A1;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12> CliArgAt<2>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)
{
    type Arg = A2;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12> CliArgAt<3>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)
{
    type Arg = A3;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12> CliArgAt<4>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)
{
    type Arg = A4;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12> CliArgAt<5>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)
{
    type Arg = A5;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12> CliArgAt<6>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)
{
    type Arg = A6;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12> CliArgAt<7>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)
{
    type Arg = A7;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12> CliArgAt<8>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)
{
    type Arg = A8;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12> CliArgAt<9>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)
{
    type Arg = A9;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12> CliArgAt<10>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)
{
    type Arg = A10;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12> CliArgAt<11>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)
{
    type Arg = A11;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12> CliArgAt<12>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)
{
    type Arg = A12;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13> CliArgAt<0>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13)
{
    type Arg = A0;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13> CliArgAt<1>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13)
{
    type Arg = A1;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13> CliArgAt<2>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13)
{
    type Arg = A2;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13> CliArgAt<3>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13)
{
    type Arg = A3;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13> CliArgAt<4>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13)
{
    type Arg = A4;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13> CliArgAt<5>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13)
{
    type Arg = A5;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13> CliArgAt<6>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13)
{
    type Arg = A6;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13> CliArgAt<7>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13)
{
    type Arg = A7;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13> CliArgAt<8>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13)
{
    type Arg = A8;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13> CliArgAt<9>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13)
{
    type Arg = A9;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13> CliArgAt<10>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13)
{
    type Arg = A10;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13> CliArgAt<11>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13)
{
    type Arg = A11;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13> CliArgAt<12>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13)
{
    type Arg = A12;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13> CliArgAt<13>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13)
{
    type Arg = A13;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14> CliArgAt<0>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14)
{
    type Arg = A0;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14> CliArgAt<1>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14)
{
    type Arg = A1;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14> CliArgAt<2>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14)
{
    type Arg = A2;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14> CliArgAt<3>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14)
{
    type Arg = A3;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14> CliArgAt<4>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14)
{
    type Arg = A4;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14> CliArgAt<5>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14)
{
    type Arg = A5;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14> CliArgAt<6>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14)
{
    type Arg = A6;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14> CliArgAt<7>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14)
{
    type Arg = A7;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14> CliArgAt<8>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14)
{
    type Arg = A8;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14> CliArgAt<9>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14)
{
    type Arg = A9;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14> CliArgAt<10>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14)
{
    type Arg = A10;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14> CliArgAt<11>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14)
{
    type Arg = A11;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14> CliArgAt<12>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14)
{
    type Arg = A12;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14> CliArgAt<13>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14)
{
    type Arg = A13;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14> CliArgAt<14>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14)
{
    type Arg = A14;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> CliArgAt<0>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15)
{
    type Arg = A0;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> CliArgAt<1>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15)
{
    type Arg = A1;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> CliArgAt<2>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15)
{
    type Arg = A2;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> CliArgAt<3>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15)
{
    type Arg = A3;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> CliArgAt<4>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15)
{
    type Arg = A4;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> CliArgAt<5>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15)
{
    type Arg = A5;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> CliArgAt<6>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15)
{
    type Arg = A6;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> CliArgAt<7>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15)
{
    type Arg = A7;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> CliArgAt<8>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15)
{
    type Arg = A8;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> CliArgAt<9>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15)
{
    type Arg = A9;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> CliArgAt<10>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15)
{
    type Arg = A10;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> CliArgAt<11>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15)
{
    type Arg = A11;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> CliArgAt<12>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15)
{
    type Arg = A12;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> CliArgAt<13>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15)
{
    type Arg = A13;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> CliArgAt<14>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15)
{
    type Arg = A14;
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> CliArgAt<15>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15)
{
    type Arg = A15;
}

/// States that a source can supply an argument as its method states it: a stated type is checked,
/// one the context decides is left to the interpretation.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot supply this argument as its method states it",
    note = "sources bind to the operation's arguments in order, one `.arg(..)` each; check the order of the `.arg(..)` calls"
)]
pub trait CliStatedFor<Stated> {}

impl<Source, Arg> CliStatedFor<StatedArg<Arg>> for Source where Source: CliSourceFor<Arg> {}

impl<Source> CliStatedFor<ContextArg> for Source {}

/// Checks, at compile time, that `Source` can supply the argument `stated` states.
pub const fn cli_source_stated<Source, Stated>(_stated: &Stated)
where
    Source: CliStatedFor<Stated>,
{
}
