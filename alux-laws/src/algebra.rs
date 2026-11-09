//! What an interpreter of laws answers: law collections, one law, and the programs it compiles.

use crate::LawOp;
use alux_ext::{OperationAlg, ext};

/// Compiles law programs with an interpreter.
#[ext(name = LawsProgramExt)]
pub impl<This> This {
    /// Compiles a law program with this interpreter.
    fn compile_laws<Program>(&self, program: &Program) -> Program::Laws
    where
        Program: LawsProgramAlg<This>,
    {
        program.compile_laws(self)
    }
}

/// Describes construction and composition of law collections.
pub trait LawsAlg {
    /// The interpreter's law collection.
    type Laws;

    /// Returns the empty law collection.
    fn laws_empty(&self) -> Self::Laws;

    /// Combines two law collections, `left`'s laws first.
    fn laws_merge(&self, left: Self::Laws, right: Self::Laws) -> Self::Laws;
}

/// Interprets one law over `Subject` that takes an `Input`.
///
/// It is stated per subject and input, as a JSON-RPC method is per signature, so an interpreter
/// states what it needs of each (a checker shows an input and finds it by type) where it implements
/// this, not on the law.
pub trait LawAlg<Subject, Input> {
    /// Interprets the law `op` of the suite named `suite`.
    fn finish_law<Op>(&self, suite: &'static str, op: Op) -> Self::Laws
    where
        Self: LawsAlg,
        Op: LawOp<Subject, Input = Input> + OperationAlg;
}

/// Compiles a first-order law program with an interpreter.
pub trait LawsProgramAlg<Compiler> {
    /// The law collection produced by `Compiler`.
    type Laws;

    /// Folds the program through `compiler`.
    fn compile_laws(&self, compiler: &Compiler) -> Self::Laws;
}
