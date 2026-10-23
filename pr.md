<!-- PR title (conventional commits style) -->
<!-- # feat(token, stake): add token and proof-of-stake specifications with Cosmos, Ethereum, and Cardano examples -->

## 🗒️ Overview

Adds two independent specifications, each with its laws and a reference in-memory interpretation, and an example that composes them into three proof-of-stake variants.

- `alux-token`: a fungible token ledger. Balances and supply are primitives; `TokenLedgerExt` derives transfer, mint, and burn, and `TokenIssuanceExt` adds minting governed by an issuance policy. `TokenLawsExt` and `TokenIssuanceLawsExt` state the obligations: the supply is the sum of balances, a transfer moves exactly its amount or changes nothing, and minting and burning change balance and supply by exactly their amount.
- `alux-stake`: proof of stake general enough for its variants. The core is a validator's weight in an epoch, taken from a weight source by a snapshot, with a snapshot lag and exclusion (jailing, ejection). Around it sit separate algebras for rewards (emission, share, payout; withholding is a share policy), quorum finality, and two weight sources: escrow (bond, unbond, withdraw, forfeit) and delegation (holdings pointed at a pool, nothing moves). Consensus enters only as primitive observations: performance and proven offenses. Each extension has a law extension.
- `alux-token-memory`, `alux-stake-memory`: reference interpretations over ordered maps whose tests run the specifications' laws over values they choose.
- `examples/token-stake`: `Chain<Variant>` composes both specifications into chains whose stake is their native token. Neither specification names the other; custody is answered by the token ledger. `Cosmos` (escrow, inflation, burned forfeits, jailing), `Ethereum` (escrow with an effective balance cap, square-root issuance, rewards per duty, equivocation and inactivity forfeits, ejection), and `Cardano` (delegation, two-epoch lag, reserve rewards withheld by performance, no forfeits) differ only in the primitives they answer. Scenarios are extensions over `trait_set!` aliases of the composed bounds, run per variant by `scenario_test!`.

## 📋 Checklist

- [x] Meaning before machinery: any semantic change moved the algebra or first-order syntax first
- [x] Laws, expansion checks, or cross-interpreter scenarios added/updated (if needed)
- [ ] Documentation updated: crate docs, `ARCHITECTURE.md`, `AGENTS.md`, migration notes (if needed)
- [ ] Ready for review (reviewers, labels, project/milestone, issues, ...)

## 📝 Additional notes

`alux-token` and `alux-stake` depend on `anyhow` besides `alux-ext`, because their public laws return `anyhow::Result`. `AGENTS.md` still says a specification crate declares only `alux-ext`, and its crate table does not list the new crates.

TODO: a queue between the weight source and the snapshot, for Ethereum's activation and exit churn limit.

TODO: an observation of recent forfeits, for Ethereum's correlation penalty.
