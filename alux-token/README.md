# alux-token

States what a fungible token means before a chain chooses how balances are stored or who may mint. Holders own amounts of one token; the supply is the sum of every balance; moving tokens keeps the supply, and only minting and burning change it, by exactly the amount minted or burned.

The crate is a specification. It depends only on [`alux-ext`](https://docs.rs/alux-ext) and names no storage, chain, or runtime.

## Primitives

| Algebra | Meaning |
| --- | --- |
| [`BalancesAlg`] | how much each holder has |
| [`SupplyAlg`] | how much of the token exists |
| [`IssuancePolicyAlg`] | who may create tokens, and how many |

## Derived meaning

[`TokenLedgerExt`] derives transfer, mint, and burn from balances and supply. [`TokenIssuanceExt`] adds governed minting. Each extension's bounds are exactly the meanings it consumes, so a new operation is a new extension and a new interpretation is a new set of primitive impls; neither edits the other.

A downstream operation composes the same way. Paying a fee to a collector before a transfer is an extension over the ledger alone:

```rust
use alux_ext::ext;
use alux_token::{BalancesAlg, Shortfall, SupplyAlg, TokenLedgerExt};
use std::ops::{Add, Sub};

/// Derives fee-charging transfers from the token ledger.
#[ext(name = FeeTransferExt)]
pub impl<This, Holder, Amount> This
where
    This: BalancesAlg<Holder = Holder, Amount = Amount> + SupplyAlg<Amount = Amount>,
    Amount: Copy + Ord + Add<Output = Amount> + Sub<Output = Amount>,
{
    /// Moves an amount and pays a fee to the collector, or changes nothing.
    fn transfer_with_fee(
        &mut self,
        from: &Holder,
        to: &Holder,
        collector: &Holder,
        amount: Amount,
        fee: Amount,
    ) -> Result<(), Shortfall<Amount>> {
        let available = self.balance(from);
        if available < amount + fee {
            return Err(Shortfall { available, requested: amount + fee });
        }
        self.token_transfer(from, collector, fee)?;
        self.token_transfer(from, to, amount)
    }
}
```

## Laws

[`TokenLawsExt`] and [`TokenIssuanceLawsExt`] state the obligations every interpretation keeps, over the holders and amounts a caller chooses: the supply is the sum of balances, a transfer moves exactly its amount or refuses an overdraft and changes nothing, minting and burning change balance and supply by exactly their amount, and issuance does exactly what the policy permits. [`alux-token-memory`](https://docs.rs/alux-token-memory) is the reference interpretation that runs them.

Proof of stake in [`alux-stake`](https://docs.rs/alux-stake) does not name this crate. A chain lends stake its custody by delegating to [`TokenLedgerExt`], and neither specification changes.
