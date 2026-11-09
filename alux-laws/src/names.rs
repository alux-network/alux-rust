//! The smallest interpreter of a law program: the names of its laws.

use crate::{LawAlg, LawOp, LawsAlg, LawsProgramAlg};
use alux_ext::{OperationAlg, ext};

/// Reads a law program through [`Names`].
#[ext(name = LawNamesExt)]
pub impl<This> This
where
    This: LawsProgramAlg<Names, Laws = Vec<(&'static str, &'static str)>>,
{
    /// Returns the laws of this program, each with the suite it belongs to.
    fn names(&self) -> Vec<(&'static str, &'static str)> {
        self.compile_laws(&Names)
    }
}

/// Lists the laws of a program as `(suite, law)` pairs, in order.
#[derive(Debug, Clone, Copy, Default)]
pub struct Names;

impl LawsAlg for Names {
    type Laws = Vec<(&'static str, &'static str)>;

    fn laws_empty(&self) -> Self::Laws {
        Vec::new()
    }

    fn laws_merge(&self, mut left: Self::Laws, right: Self::Laws) -> Self::Laws {
        left.extend(right);
        left
    }
}

impl<Subject, Input> LawAlg<Subject, Input> for Names {
    fn finish_law<Op>(&self, suite: &'static str, _: Op) -> Vec<(&'static str, &'static str)>
    where
        Op: LawOp<Subject, Input = Input> + OperationAlg,
    {
        vec![(suite, Op::NAME)]
    }
}
