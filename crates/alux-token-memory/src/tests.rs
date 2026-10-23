use crate::MemoryLedger;
use alux_token::{TokenIssuanceLawsExt, TokenLawsExt, TokenLedgerExt};
use anyhow::Result;

const HOLDERS: [u8; 3] = [0, 1, 2];
const AMOUNTS: [u64; 4] = [0, 1, 30, 1_000];

fn funded() -> MemoryLedger<u8, u64> {
    let mut ledger = MemoryLedger::new(Some(0));
    for holder in HOLDERS {
        ledger.token_mint(&holder, 100);
    }
    ledger
}

fn each_case(mut check: impl FnMut(&mut MemoryLedger<u8, u64>, u8, u8, u64) -> Result<()>) -> Result<()> {
    for from in HOLDERS {
        for to in HOLDERS {
            for amount in AMOUNTS {
                check(&mut funded(), from, to, amount)?;
            }
        }
    }
    Ok(())
}

#[test]
fn transfers_satisfy_the_ledger_laws() -> Result<()> {
    each_case(|ledger, from, to, amount| ledger.check_transfer(&from, &to, amount))
}

#[test]
fn minting_and_burning_satisfy_the_ledger_laws() -> Result<()> {
    each_case(|ledger, holder, _, amount| {
        ledger.check_mint_then_burn(&holder, amount)?;
        ledger.check_burn(&holder, amount)
    })
}

#[test]
fn issuance_satisfies_the_policy_laws() -> Result<()> {
    each_case(|ledger, issuer, to, amount| ledger.check_issue(&issuer, &to, amount))
}
