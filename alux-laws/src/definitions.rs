//! The catalog interpreter of a law program: what every law is, with nothing run.

use crate::{LawAlg, LawOp, LawsAlg, LawsProgramAlg};
use alux_ext::{OperationAlg, ext};
use core::any::type_name;

/// Reads a law program through [`Definitions`].
#[ext(name = LawDefinitionsExt)]
pub impl<This> This
where
    This: LawsProgramAlg<Definitions, Laws = Vec<LawDefinition>>,
{
    /// Returns what every law of this program is, in order.
    fn definitions(&self) -> Vec<LawDefinition> {
        self.compile_laws(&Definitions)
    }
}

/// What one law is, as its suite defines it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LawDefinition {
    /// The suite the law belongs to.
    pub suite: &'static str,
    /// The law's name.
    pub law: &'static str,
    /// The law's documentation, as written on its method.
    pub doc: &'static str,
    /// The name of the law's input argument; empty for a law over the subject alone.
    pub arg_names: &'static [&'static str],
    /// The type the law is checked on.
    pub subject: &'static str,
    /// The type of the law's input; `()` for a law over the subject alone.
    pub input: &'static str,
}

/// Lists what every law of a program is, in order, for documentation and coverage reports.
#[derive(Debug, Clone, Copy, Default)]
pub struct Definitions;

impl LawsAlg for Definitions {
    type Laws = Vec<LawDefinition>;

    fn laws_empty(&self) -> Self::Laws {
        Vec::new()
    }

    fn laws_merge(&self, mut left: Self::Laws, right: Self::Laws) -> Self::Laws {
        left.extend(right);
        left
    }
}

impl<Subject, Input> LawAlg<Subject, Input> for Definitions {
    fn finish_law<Op>(&self, suite: &'static str, _: Op) -> Vec<LawDefinition>
    where
        Op: LawOp<Subject, Input = Input> + OperationAlg,
    {
        vec![LawDefinition {
            suite,
            law: Op::NAME,
            doc: Op::DOC,
            arg_names: Op::ARG_NAMES,
            subject: type_name::<Subject>(),
            input: type_name::<Input>(),
        }]
    }
}
