use crate::tests::{ALICE, BOB, CardanoScenariosExt, CosmosScenariosExt, EscrowScenariosExt, EthereumScenariosExt};
use crate::{Cardano, Chain, Cosmos, Ethereum};
use alux_token::TokenLedgerExt;

/// Runs a scenario extension on a fresh chain of a variant, in which Alice and Bob hold 1000 each.
macro_rules! scenario_test {
    ($test:ident: $variant:ty => $scenario:ident) => {
        #[test]
        fn $test() {
            let mut chain = Chain::<$variant>::default();
            chain.token_mint(&ALICE, 1_000);
            chain.token_mint(&BOB, 1_000);
            if let Err(violation) = chain.$scenario() {
                panic!("{violation:#}");
            }
        }
    };
}

scenario_test!(cosmos_escrow_keeps_the_supply: Cosmos => scenario_escrow_keeps_the_supply);
scenario_test!(cosmos_custody_refuses_what_the_ledger_refuses: Cosmos => scenario_custody_refuses_what_the_ledger_refuses);
scenario_test!(cosmos_forfeit_burns_and_jail_removes_weight: Cosmos => scenario_forfeit_burns_and_jail_removes_weight);
scenario_test!(cosmos_inflation_raises_the_supply_by_exactly_what_it_pays: Cosmos => scenario_inflation_raises_the_supply_by_exactly_what_it_pays);
scenario_test!(ethereum_escrow_keeps_the_supply: Ethereum => scenario_escrow_keeps_the_supply);
scenario_test!(ethereum_custody_refuses_what_the_ledger_refuses: Ethereum => scenario_custody_refuses_what_the_ledger_refuses);
scenario_test!(ethereum_weight_is_capped_at_the_effective_balance: Ethereum => scenario_weight_is_capped_at_the_effective_balance);
scenario_test!(ethereum_equivocation_forfeits_and_ejects: Ethereum => scenario_equivocation_forfeits_and_ejects);
scenario_test!(ethereum_inactivity_leak_grows_with_the_square_of_the_epochs: Ethereum => scenario_inactivity_leak_grows_with_the_square_of_the_epochs);
scenario_test!(ethereum_rewards_pay_duties_performed_and_mint_only_what_is_paid: Ethereum => scenario_rewards_pay_duties_performed_and_mint_only_what_is_paid);
scenario_test!(cardano_delegation_moves_nothing_and_weight_lags_two_epochs: Cardano => scenario_delegation_moves_nothing_and_weight_lags_two_epochs);
scenario_test!(cardano_withholds_rewards_from_weak_pools_and_keeps_the_supply: Cardano => scenario_withholds_rewards_from_weak_pools_and_keeps_the_supply);
