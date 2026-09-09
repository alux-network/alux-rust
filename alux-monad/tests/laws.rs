//! Checks monad laws and evaluation behavior through public extensions.

use alux_monad::*;
use core::cell::Cell;
use core::convert::identity;
use core::iter::once;

#[test]
fn option_laws() {
    let f = |x| (x > 0).then_some(x + 1);
    let g = |x| (x % 2 == 0).then_some(x * 2);
    for x in -3..=3 {
        assert_eq!(Some(x).bind(f), f(x));
        for m in [None, Some(x)] {
            assert_eq!(m.bind(Some), m);
            assert_eq!(m.bind(f).bind(g), m.bind(|a| f(a).bind(g)));
            assert_eq!(m.map(f).join(), m.bind(f));
        }
    }
    for m in [None, Some(None), Some(Some(1))] {
        assert_eq!(m.join(), m.bind(identity));
    }
}

#[test]
fn result_laws() {
    let f = |x| if x > 0 { Ok(x + 1) } else { Err("nonpositive") };
    let g = |x| if x % 2 == 0 { Ok(x * 2) } else { Err("odd") };
    for x in -3..=3 {
        assert_eq!(Ok(x).bind(f), f(x));
        for m in [Err("initial"), Ok(x)] {
            assert_eq!(m.bind(Ok), m);
            assert_eq!(m.bind(f).bind(g), m.bind(|a| f(a).bind(g)));
            assert_eq!(m.map(f).join(), m.bind(f));
        }
    }
    for m in [Err("outer"), Ok(Err("inner")), Ok(Ok(1))] {
        assert_eq!(m.join(), m.bind(identity));
    }
}

#[test]
fn iterator_laws() {
    let f = |x| if x > 0 { vec![x, x + 1] } else { vec![] };
    let g = |x| [x, x * 2];
    for x in -3..=3 {
        assert_eq!(once(x).bind(f).collect::<Vec<_>>(), f(x));
        for m in [vec![], vec![x], vec![x, 0, x + 1]] {
            assert_eq!(m.iter().copied().bind(once).collect::<Vec<_>>(), m);
            let left: Vec<_> = m.iter().copied().bind(f).bind(g).collect();
            let right: Vec<_> = m.iter().copied().bind(|a| f(a).into_iter().bind(g)).collect();
            assert_eq!(left, right);
            assert_eq!(m.iter().copied().map(f).join().collect::<Vec<_>>(), m.into_iter().bind(f).collect::<Vec<_>>());
        }
    }
    let nested = [vec![1, 2], vec![], vec![3]];
    assert_eq!(
        nested.clone().into_iter().join().collect::<Vec<_>>(),
        nested.into_iter().bind(identity).collect::<Vec<_>>()
    );
}

#[test]
fn absence_and_errors_skip_the_continuation() {
    assert_eq!(None::<i32>.bind(|_| -> Option<i32> { panic!("absent") }), None);
    let error = String::from("original error");
    let result = Err::<i32, _>(error).bind(|_| -> Result<i32, String> { panic!("failed") });
    assert_eq!(result, Err(String::from("original error")));
}

#[test]
fn single_value_instances_accept_consuming_closures() {
    let captured = String::from("option");
    assert_eq!(Some(()).bind(|()| Some(captured)), Some(String::from("option")));
    let captured = String::from("result");
    assert_eq!(Ok::<_, ()>(()).bind(|()| Ok(captured)), Ok(String::from("result")));
}

#[test]
fn iterator_bind_is_lazy_and_visits_in_order() {
    let calls = Cell::new(0);
    let mut visited = Vec::new();
    let mut values = (1..).bind(|x| {
        calls.set(calls.get() + 1);
        visited.push(x);
        [x, x * 10]
    });
    assert_eq!(calls.get(), 0);
    assert_eq!(values.next(), Some(1));
    assert_eq!(calls.get(), 1);
    assert_eq!(values.next(), Some(10));
    assert_eq!(calls.get(), 1);
    assert_eq!(values.take(3).collect::<Vec<_>>(), [2, 20, 3]);
    assert_eq!(visited, [1, 2, 3]);
}

#[test]
fn iterator_join_is_lazy_and_handles_empty_inner_sequences() {
    let calls = Cell::new(0);
    let values = [vec![], vec![1, 2], vec![3]].into_iter().inspect(|_| calls.set(calls.get() + 1)).join();
    assert_eq!(calls.get(), 0);
    assert_eq!(values.take(1).collect::<Vec<_>>(), [1]);
    assert_eq!(calls.get(), 2);
}
