//! What the reader declares from source, read back through the index, and that it reads this
//! workspace.

use alux_source_index::{Declaration, Index, IndexQuestionsExt};
use alux_source_syn::ReadSourceExt;
use std::path::Path;

/// A trait definition keeps its supertraits apart from its other bounds, so a capability alias
/// states none, and a bound on a parameter, in a `where` clause, on a method or on an associated
/// type is one; `Sized`, `Send`, `Sync`, `Clone` and `Debug` are bounds any definition may state.
#[test]
fn a_trait_definition_keeps_its_bounds_apart_from_its_supertraits() {
    let source = r"
        trait Alias: Read + Write {}
        trait OnParam<K: Ord> {}
        trait Shared { type Handle: Clone + Send + Sync; fn sized(self) where Self: Sized; }
        trait OnMethod { fn get<K>(&self) where K: Default; }
        trait OnAssoc { type Key: Ord; }
        trait Plain { type Key; fn get(&self) -> Self::Key; }
    ";
    let declared = Index.read_source("defs.rs", source).unwrap();
    let names: Vec<_> = declared.bounded_trait_defs().iter().map(|declared| declared.name()).collect();
    assert_eq!(names, ["OnParam", "OnMethod", "OnAssoc"]);
}

/// A declaration in inline modules is placed in them, inside the file it is read from.
#[test]
fn a_declaration_is_placed_in_its_modules_and_file() {
    let declared = Index.read_source("lib.rs", "mod outer { mod inner { impl Show for Item {} } }").unwrap();
    let [only] = declared.as_slice() else { panic!("{declared:?}") };
    assert_eq!(
        (only.file.as_str(), only.modules.as_slice()),
        ("lib.rs", &["outer".to_owned(), "inner".to_owned()][..])
    );
    assert_eq!(only.name(), "Item");
}

/// An `#[ext]` block is an extension, `pub impl` or not, with its carrier, bounds and backend; a plain
/// inherent `impl` declares nothing.
#[test]
fn an_extension_block_keeps_its_name_carrier_bounds_and_backend() {
    let source = r"
        #[ext(name = RouteExt, defunc(via = http))]
        pub impl<This, K> This where This: Router<K> + ?Sized, K: Key { fn route(&self) {} }
        #[ext(name = PlainExt)]
        impl<This: Show> This {}
        impl Item {}
    ";
    let declared = Index.read_source("ext.rs", source).unwrap();
    let extensions = declared.extensions();
    assert_eq!(extensions.len(), 2);
    let Declaration::Extension { name, carrier, bounds, via, methods } = &extensions[0].declaration else {
        panic!("{extensions:?}")
    };
    assert_eq!((name.as_str(), carrier.as_str(), via.as_deref()), ("RouteExt", "This", Some("http")));
    assert_eq!(bounds[0].traits, ["Router<K>"]);
    assert_eq!(methods, &["route"]);
}

/// The reader finds this workspace's own law suites and the impls they are checked on: the counter
/// walkthrough of `alux-laws-check`, read as source.
#[test]
fn reads_the_law_suites_of_this_workspace() {
    let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("../alux-laws-check/tests");
    let declared = Index.read_tree(&tests).unwrap();
    let suites: Vec<_> = declared.laws_of("CounterAlg").iter().map(|declared| declared.name()).collect();
    assert_eq!(suites, ["CounterLaws"]);
    let impls: Vec<_> = declared.impls_of("CounterAlg").iter().map(|declared| declared.name()).collect();
    assert_eq!(impls, ["Good", "BumpsTwice"]);
}

/// The reader parses every source file of this workspace, extension blocks written `pub impl`
/// included, and finds the source algebra's own law suite among them.
#[test]
fn reads_the_whole_workspace() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let declared = Index.read_tree(&root).unwrap();
    assert!(declared.extensions().len() > 100, "{}", declared.extensions().len());
    assert_eq!(declared.laws_of("SourceAlg")[0].name(), "SourceLaws");
}
