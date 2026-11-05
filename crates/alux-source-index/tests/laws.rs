//! `Index` obeys the laws of the source algebra, beside a second interpreter that only counts, so
//! the laws do not depend on how declarations are kept.

use alux_laws_check::SubjectLawsExt;
use alux_source::{SourceAlg, SourceLaws, source_laws};
use alux_source_index::{Declared, Index};

/// Some declarations, placed in a module and a file.
fn declared() -> Vec<Vec<Declared>> {
    let bound = Index.bound(Index.ty("This"), vec![Index.name("CounterAlg")]);
    vec![
        Index.source_empty(),
        Index.trait_def(Index.name("CounterAlg"), Vec::new(), Vec::new()),
        Index.module(Index.name("tests"), Index.trait_impl(Index.name("CounterAlg"), Index.ty("Good"), Vec::new())),
        Index.file(
            "ext.rs",
            Index.extension(Index.name("CounterLaws"), Index.ty("This"), vec![bound], None, Vec::new()),
        ),
    ]
}

/// Every law holds on the index, over every pair and triple of the declarations above.
#[test]
fn the_index_obeys_the_source_laws() {
    let all = declared();
    let pairs = || all.iter().flat_map(|a| all.iter().map(move |b| (a.clone(), b.clone())));
    let triples: Vec<_> =
        pairs().flat_map(|(a, b)| all.iter().map(move |c| (a.clone(), b.clone(), c.clone()))).collect();
    let placed: Vec<_> = pairs().map(|(a, b)| ("outer".to_owned(), a, b)).collect();
    (|| Index)
        .check()
        .inputs(|| all.clone())
        .inputs(|| triples)
        .inputs(|| placed)
        .input("outer".to_owned())
        .obeys(source_laws());
}

/// Counts declarations and nothing else: names, types and places mean nothing to it.
#[derive(Debug, Clone, Copy)]
struct Count;

impl SourceAlg for Count {
    type Name = ();
    type Type = ();
    type Bound = ();
    type Source = usize;

    fn name(&self, _: &str) {}
    fn ty(&self, _: &str) {}
    fn bound(&self, (): (), _: Vec<()>) {}
    fn source_empty(&self) -> usize {
        0
    }
    fn source_merge(&self, left: usize, right: usize) -> usize {
        left + right
    }
    fn file(&self, _: &str, source: usize) -> usize {
        source
    }
    fn module(&self, (): (), source: usize) -> usize {
        source
    }
    fn trait_def(&self, (): (), _: Vec<()>, _: Vec<()>) -> usize {
        1
    }
    fn trait_impl(&self, (): (), (): (), _: Vec<()>) -> usize {
        1
    }
    fn extension(&self, (): (), (): (), _: Vec<()>, _: Option<()>, _: Vec<()>) -> usize {
        1
    }
}

/// The same laws hold on the counting interpreter.
#[test]
fn a_counting_interpreter_obeys_the_source_laws() {
    (|| Count)
        .check()
        .inputs(|| vec![0_usize, 1, 3])
        .input((1_usize, 2_usize, 3_usize))
        .input(((), 2_usize, 5_usize))
        .input(())
        .obeys(source_laws());
}

/// A law is an ordinary method too, callable on any interpreter.
#[test]
fn a_law_is_called_directly() {
    assert_eq!(Index.merge_is_associative(&(Vec::new(), Vec::new(), Vec::new())), Ok(()));
}
