//! Validates pure trait-algebra generation independently of any transport.

#![allow(async_fn_in_trait, dead_code)]

use alux_sdk_macros::trait_algebra;

#[trait_algebra(derive(Debug, Clone, PartialEq))]
trait Calculator {
    async fn add(&self, left: i64, right: i64) -> i64;
    async fn note(&self, message: String);
}

#[derive(Default)]
struct Calculation {
    sum: i64,
    notes: Vec<String>,
}

impl CalculatorInterpreter for Calculation {
    async fn add(&mut self, left: i64, right: i64) -> i64 {
        let result = left + right;
        self.sum += result;
        result
    }

    async fn note(&mut self, message: String) {
        self.notes.push(message);
    }
}

#[tokio::test]
async fn operations_are_pure_data_with_typed_construction_and_elimination() {
    let operation = CalculatorOp::add(2, 3);
    assert_eq!(operation, CalculatorOp::Add { left: 2, right: 3 });

    let mut calculation = Calculation::default();
    let reply = operation.interpret(&mut calculation).await;
    assert_eq!(reply.into_add(), 5);

    CalculatorOp::note("meaning".to_owned()).interpret(&mut calculation).await;
    assert_eq!(calculation.sum, 5);
    assert_eq!(calculation.notes, ["meaning"]);
}

#[trait_algebra(derive(Debug, Clone, PartialEq))]
trait Store {
    type Account;
    type Item;

    async fn put(&self, account: Self::Account, item: Self::Item) -> u64;
    async fn identity(&self) -> Self::Account;
    async fn clear(&self, account: Self::Account);
}

struct Memory {
    identity: u32,
    items: Vec<(u32, String)>,
}

impl StoreInterpreter for Memory {
    type Account = u32;
    type Item = String;

    async fn put(&mut self, account: u32, item: String) -> u64 {
        self.items.push((account, item));
        self.items.len() as u64
    }

    async fn identity(&mut self) -> u32 {
        self.identity
    }

    async fn clear(&mut self, _account: u32) {
        self.items.clear();
    }
}

#[tokio::test]
async fn associated_carriers_are_lifted_into_operation_and_reply_syntax() {
    let mut memory = Memory { identity: 7, items: Vec::new() };

    let count = StoreOp::<u32, String>::put(1, "item".to_owned()).interpret(&mut memory).await;
    assert_eq!(count.into_put(), 1);

    let identity = StoreOp::<u32, String>::identity().interpret(&mut memory).await;
    assert_eq!(identity.into_identity(), 7);

    StoreOp::<u32, String>::clear(1).interpret(&mut memory).await;
    assert!(memory.items.is_empty());
}

#[trait_algebra(derive(Debug, PartialEq))]
trait Flag {
    fn set(&self, value: bool) -> bool;
}

#[derive(Default)]
struct FlagState(bool);

impl FlagInterpreter for FlagState {
    fn set(&mut self, value: bool) -> bool {
        self.0 = value;
        self.0
    }
}

#[test]
fn a_synchronous_algebra_has_a_synchronous_fold() {
    let mut state = FlagState::default();

    let reply = FlagOp::set(true).interpret(&mut state);

    assert!(reply.into_set());
    assert!(state.0);
}
