use alux_token::{BalancesAlg, IssuancePolicyAlg, SupplyAlg};
use std::collections::BTreeMap;

/// Keeps a token ledger in memory, with at most one holder allowed to issue.
#[derive(Clone, Debug)]
pub struct MemoryLedger<Holder, Amount> {
    issuer: Option<Holder>,
    balances: BTreeMap<Holder, Amount>,
    supply: Amount,
}

impl<Holder, Amount> MemoryLedger<Holder, Amount>
where
    Amount: Default,
{
    /// Creates an empty ledger in which only the issuer, if any, may issue.
    pub fn new(issuer: Option<Holder>) -> Self {
        Self { issuer, balances: BTreeMap::new(), supply: Amount::default() }
    }
}

impl<Holder, Amount> BalancesAlg for MemoryLedger<Holder, Amount>
where
    Holder: Clone + Ord,
    Amount: Copy + Default + PartialEq,
{
    type Holder = Holder;
    type Amount = Amount;

    fn balance(&self, holder: &Holder) -> Amount {
        self.balances.get(holder).copied().unwrap_or_default()
    }

    fn balance_set(&mut self, holder: &Holder, amount: Amount) {
        if amount == Amount::default() {
            self.balances.remove(holder);
        } else {
            self.balances.insert(holder.clone(), amount);
        }
    }

    fn balance_holders(&self) -> Vec<Holder> {
        self.balances.keys().cloned().collect()
    }
}

impl<Holder, Amount> SupplyAlg for MemoryLedger<Holder, Amount>
where
    Amount: Copy,
{
    type Amount = Amount;

    fn supply(&self) -> Amount {
        self.supply
    }

    fn supply_set(&mut self, amount: Amount) {
        self.supply = amount;
    }
}

/// Lets only the ledger's issuer issue, any amount.
impl<Holder, Amount> IssuancePolicyAlg for MemoryLedger<Holder, Amount>
where
    Holder: PartialEq,
{
    type Holder = Holder;
    type Amount = Amount;
    type Denial = ();

    fn issuance_require(&self, issuer: &Holder, _: &Amount) -> Result<(), ()> {
        (self.issuer.as_ref() == Some(issuer)).then_some(()).ok_or(())
    }
}
