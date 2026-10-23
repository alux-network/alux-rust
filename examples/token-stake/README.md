# token-stake

Composes [`alux-token`](../../alux-token) and [`alux-stake`](../../alux-stake) into chains whose stake is their native token. Neither specification names the other; a chain is the only place they meet.

`Chain<Variant>` keeps the carriers. The token ledger, epochs, snapshots, exclusion, and observed performance are written once for every variant; bonds, unbonding, and custody are written once for every `Escrowed` variant. Each variant marker answers only the primitives where it differs:

| | `Cosmos` | `Ethereum` | `Cardano` |
| --- | --- | --- | --- |
| Weight source | escrow | escrow, capped at the maximum effective balance | delegation: balances point at a pool, nothing moves |
| Snapshot lag | none | one epoch | two epochs |
| Emission | one percent of supply, minted | four times the square root of the total weight, minted | three tenths of a percent of the reserve, transferred |
| Reward share | by weight | by weight, scaled by duties performed out of assigned | by weight, scaled by blocks produced out of expected |
| Misbehavior | forfeit (burned) and jailing | forfeit for equivocation and for inactivity (quadratic leak), and ejection | withheld rewards only |

TODO: rate-limit how much weight enters and leaves an `Ethereum` chain per epoch (the activation and exit churn limit), once the stake specification has a queue between the weight source and the snapshot.

TODO: grow an `Ethereum` forfeit with how much other stake was forfeited in the same window (the correlation penalty), once the stake specification observes recent forfeits.

The tests state what each composition keeps: escrow keeps the supply and the pool holds exactly what is escrowed; forfeiting burns exactly what it takes; inflation raises the supply by exactly what it pays; Cardano rewards move tokens out of the reserve without changing the supply, and withholding never touches principal.
