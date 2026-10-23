# alux-stake-memory

Interprets [`alux-stake`](https://docs.rs/alux-stake) over ordered maps. `MemoryStake` keeps wallets as its own custody and holdings, and takes every policy the specification leaves open as a parameter: the weight source, snapshot lag, unbonding delay, quorum fraction, and emission per epoch. Offenses forfeit their value in basis points; performance is blocks produced out of blocks expected, and scales each validator's share of the pot by weight. It is the reference interpretation the specification's laws run against.
