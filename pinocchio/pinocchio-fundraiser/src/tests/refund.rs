use litesvm::{types::TransactionResult, LiteSVM};
use solana_sdk::{
    clock::Clock,
    message::{AccountMeta, Instruction},
    msg,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

use crate::{
    state::{contributor::ContributorAccount, HasLen},
    tests::{
        contribute::{contribute, ContributeData},
        init::{initialize, program_id, setup, InitializeData},
    },
};

pub(super) fn refund(
    svm: &mut LiteSVM,
    maker: &Keypair,
    init_data: &InitializeData,
    contribute_data: &ContributeData,
) -> TransactionResult {
    let ix_data = [vec![2u8]].concat();
    let ix_accounts = [
        AccountMeta::new(contribute_data.contributor.pubkey(), true),
        AccountMeta::new_readonly(init_data.mint, false),
        AccountMeta::new_readonly(maker.pubkey(), false),
        AccountMeta::new(init_data.fundraiser_pda.0, false),
        AccountMeta::new(contribute_data.contributor_ata, false),
        AccountMeta::new(init_data.vault, false),
        AccountMeta::new(contribute_data.contribution_pda.0, false),
        AccountMeta::new_readonly(init_data.system_program, false),
        AccountMeta::new_readonly(init_data.token_program, false),
        AccountMeta::new_readonly(init_data.associated_token_program, false),
    ]
    .to_vec();
    let ix = Instruction {
        program_id: program_id(),
        accounts: ix_accounts,
        data: ix_data,
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&contribute_data.contributor.pubkey()),
        &[&contribute_data.contributor],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx)
}

#[test]
pub fn test_refund() {
    let (mut svm, maker) = setup();
    let init_data = InitializeData::initialize_data(&mut svm, &maker);
    initialize(&mut svm, &maker, &init_data).unwrap();

    msg!("Contribute after start time, but during the duration");
    let mut clock = svm.get_sysvar::<Clock>();
    msg!("clock before update: {}", clock.unix_timestamp);
    clock.unix_timestamp = init_data.start_time + 10i64;
    svm.set_sysvar::<Clock>(&clock);
    let clock = svm.get_sysvar::<Clock>();
    msg!("clock after update: {}", clock.unix_timestamp);

    let contribute_data = ContributeData::generate_data(&mut svm, &maker, &init_data);
    contribute(&mut svm, &maker, &init_data, &contribute_data, None).unwrap();

    let contributor = svm
        .get_account(&contribute_data.contributor.pubkey())
        .unwrap();
    refund(&mut svm, &maker, &init_data, &contribute_data).unwrap();
    let new_contributor = svm
        .get_account(&contribute_data.contributor.pubkey())
        .unwrap();
    assert!(
        contributor.lamports < new_contributor.lamports,
        "lamports should be refunded"
    );

    let result = svm.get_account(&contribute_data.contribution_pda.0);
    assert!(result.is_none(), "contribution account should be closed");
}
