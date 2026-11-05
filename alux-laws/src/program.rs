//! The two ways to combine law programs: join and projection.

use crate::{Failure, LawAlg, LawOp, LawsAlg, LawsProgramAlg};
use alux_ext::{OperationAlg, ext};
use core::marker::PhantomData;

/// Combines law programs.
#[ext(name = LawsCompositionExt)]
pub impl<This> This
where
    This: LawsProgram,
{
    /// Returns these laws and `other`'s, checked together.
    fn join<Other>(self, other: Other) -> Join<This, Other>
    where
        Other: LawsProgram,
    {
        Join { left: self, right: other }
    }

    /// Returns these laws over a root `Root`: each law gets `project(root)` as its subject.
    fn over<Root, Subject, Project>(self, project: Project) -> Over<Project, This, Root>
    where
        Project: Fn(&Root) -> Subject,
    {
        Over { project, program: self, root: PhantomData }
    }
}

/// Marks law programs: a suite's generated program, and the joins and projections built from them.
///
/// It is what [`LawsCompositionExt`] combines; compiling a program asks only [`LawsProgramAlg`].
pub trait LawsProgram {}

impl<Left, Right> LawsProgram for Join<Left, Right>
where
    Left: LawsProgram,
    Right: LawsProgram,
{
}

impl<Project, Program, Root> LawsProgram for Over<Project, Program, Root> where Program: LawsProgram {}

/// Two law programs checked together, `Left`'s laws first.
#[derive(Debug, Clone, Copy)]
pub struct Join<Left, Right> {
    left: Left,
    right: Right,
}

impl<Left, Right, Compiler> LawsProgramAlg<Compiler> for Join<Left, Right>
where
    Compiler: LawsAlg,
    Left: LawsProgramAlg<Compiler, Laws = Compiler::Laws>,
    Right: LawsProgramAlg<Compiler, Laws = Compiler::Laws>,
{
    type Laws = Compiler::Laws;

    fn compile_laws(&self, compiler: &Compiler) -> Self::Laws {
        compiler.laws_merge(self.left.compile_laws(compiler), self.right.compile_laws(compiler))
    }
}

/// A law program over a subject, read over a root: each law projects its subject from the root.
#[derive(Debug, Clone, Copy)]
pub struct Over<Project, Program, Root> {
    project: Project,
    program: Program,
    root: PhantomData<fn(&Root)>,
}

impl<Project, Program, Root, Compiler> LawsProgramAlg<Compiler> for Over<Project, Program, Root>
where
    Compiler: LawsAlg,
    Program: for<'a> LawsProgramAlg<Projecting<'a, Compiler, Project, Root>, Laws = Compiler::Laws>,
{
    type Laws = Compiler::Laws;

    fn compile_laws(&self, compiler: &Compiler) -> Self::Laws {
        self.program.compile_laws(&Projecting { compiler, project: &self.project, root: PhantomData })
    }
}

/// An interpreter over a root, seen as one over the subject a projection reads from it.
///
/// [`Over`] compiles its program through this; nothing else names it.
#[derive(Debug)]
pub struct Projecting<'a, Compiler, Project, Root> {
    compiler: &'a Compiler,
    project: &'a Project,
    root: PhantomData<fn(&Root)>,
}

impl<Compiler, Project, Root> LawsAlg for Projecting<'_, Compiler, Project, Root>
where
    Compiler: LawsAlg,
{
    type Laws = Compiler::Laws;

    fn laws_empty(&self) -> Self::Laws {
        self.compiler.laws_empty()
    }

    fn laws_merge(&self, left: Self::Laws, right: Self::Laws) -> Self::Laws {
        self.compiler.laws_merge(left, right)
    }
}

impl<Root, Subject, Input, Compiler, Project> LawAlg<Subject, Input> for Projecting<'_, Compiler, Project, Root>
where
    Compiler: LawsAlg + LawAlg<Root, Input>,
    Project: Fn(&Root) -> Subject,
{
    fn finish_law<Op>(&self, suite: &'static str, op: Op) -> <Self as LawsAlg>::Laws
    where
        Op: LawOp<Subject, Input = Input> + OperationAlg,
    {
        self.compiler.finish_law(suite, Projected { project: self.project, op, root: PhantomData })
    }
}

/// One law over a root, through the projection.
struct Projected<'a, Project, Op, Root> {
    project: &'a Project,
    op: Op,
    root: PhantomData<fn(&Root)>,
}

impl<Root, Subject, Project, Op> LawOp<Root> for Projected<'_, Project, Op, Root>
where
    Project: Fn(&Root) -> Subject,
    Op: LawOp<Subject>,
{
    type Input = Op::Input;

    fn check(&self, root: &mut Root, input: &Op::Input) -> Result<(), Failure> {
        let mut subject = (self.project)(root);
        self.op.check(&mut subject, input)
    }
}

impl<Project, Op, Root> OperationAlg for Projected<'_, Project, Op, Root>
where
    Op: OperationAlg,
{
    type Context = Op::Context;
    type Args = Op::Args;

    const DOC: &'static str = Op::DOC;
    const NAME: &'static str = Op::NAME;
    const ARG_NAMES: &'static [&'static str] = Op::ARG_NAMES;
}
