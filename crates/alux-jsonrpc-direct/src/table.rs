use crate::error::RpcError;
use core::fmt::{self, Debug, Display};
use core::future::Future;
use core::pin::Pin;
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::sync::Arc;

/// The answer one dispatched method produces.
pub type Answer = Pin<Box<dyn Future<Output = Result<Value, RpcError>> + Send>>;

/// One registered method: it decodes its parameters, applies its operation, and answers.
pub(crate) type Method = Arc<dyn Fn(Option<Value>) -> Answer + Send + Sync>;

/// Names a method that two composed programs both declared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateMethod(pub &'static str);

impl Display for DuplicateMethod {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "method `{}` is declared twice", self.0)
    }
}

impl core::error::Error for DuplicateMethod {}

/// A JSON-RPC surface: every method a program declared, keyed by the name it answers to.
#[derive(Clone, Default)]
pub struct MethodTable {
    methods: BTreeMap<&'static str, Method>,
}

impl MethodTable {
    /// Returns every method name this surface answers to, in lexical order.
    pub fn names(&self) -> Vec<&'static str> {
        self.methods.keys().copied().collect()
    }

    /// Returns how many methods this surface answers to.
    pub fn len(&self) -> usize {
        self.methods.len()
    }

    /// Returns whether this surface answers to nothing.
    pub fn is_empty(&self) -> bool {
        self.methods.is_empty()
    }

    /// Composes two surfaces, which is only defined when they name different methods.
    ///
    /// # Errors
    ///
    /// Answers with the duplicated name when both surfaces declare it.
    pub fn merge(mut self, other: Self) -> Result<Self, DuplicateMethod> {
        for (name, method) in other.methods {
            self.insert(name, method)?;
        }

        Ok(self)
    }

    pub(crate) fn insert(&mut self, name: &'static str, method: Method) -> Result<(), DuplicateMethod> {
        match self.methods.entry(name) {
            Entry::Occupied(_) => Err(DuplicateMethod(name)),
            Entry::Vacant(entry) => {
                entry.insert(method);
                Ok(())
            }
        }
    }

    pub(crate) fn get(&self, name: &str) -> Option<&Method> {
        self.methods.get(name)
    }
}

impl Debug for MethodTable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("MethodTable").field("methods", &self.names()).finish()
    }
}
