use litesvm::{types::TransactionResult, LiteSVM};
use litesvm_token::{spl_token::state::Account, CreateAssociatedTokenAccount, MintTo};
use solana_sdk::{
    clock::Clock,
    message::{AccountMeta, Instruction},
    msg,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

use crate::{
    state::{contributor::ContributorAccount, fundraiser::FundRaiser},
    tests::init::{initialize, program_id, setup, InitializeData},
};

pub(super) struct ContributeData {
    pub contributor: Keypair,
    pub contributor_ata: Pubkey,
    pub contributing_amount: u64,
    pub contribution_pda: (Pubkey, u8),
}

impl ContributeData {
    pub fn generate_data(svm: &mut LiteSVM, maker: &Keypair, init_data: &InitializeData) -> Self {
        let contributor = Keypair::new();
        svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL)
            .unwrap();
        let contributor_ata = CreateAssociatedTokenAccount::new(svm, &contributor, &init_data.mint)
            .token_program_id(&init_data.token_program)
            .send()
            .unwrap();

        let contributing_amount = 2000u64;
        let contribution_pda = Pubkey::find_program_address(
            &[
                b"contribution",
                init_data.fundraiser_pda.0.as_ref(),
                contributor.pubkey().as_ref(),
            ],
            &program_id(),
        );

        MintTo::new(svm, maker, &init_data.mint, &contributor_ata, 1_000_000)
            .send()
            .unwrap();

        Self {
            contributor,
            contributor_ata,
            contributing_amount,
            contribution_pda,
        }
    }
}

pub(super) fn contribute(
    svm: &mut LiteSVM,
    maker: &Keypair,
    init_data: &InitializeData,
    contribute_data: &ContributeData,
    amount: Option<u64>,
) -> TransactionResult {
    let contributing_amount = if amount.is_some() {
        amount.unwrap()
    } else {
        contribute_data.contributing_amount
    };
    let ix_data = [
        vec![1u8], //insn discriminator
        contributing_amount.to_le_bytes().to_vec(),
    ]
    .concat();
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
pub fn test_contribute() {
    let (mut svm, maker) = setup();
    let init_data = InitializeData::initialize_data(&mut svm, &maker);
    initialize(&mut svm, &maker, &init_data).unwrap();

    msg!("contribute before start time");
    let contribute_data = ContributeData::generate_data(&mut svm, &maker, &init_data);
    let result = contribute(&mut svm, &maker, &init_data, &contribute_data, None);
    assert!(result.is_err(), "Contribute before start time should fail");

    msg!("Contribute after start time, but during the duration");
    svm.expire_blockhash();
    let mut clock = svm.get_sysvar::<Clock>();
    let current_time = clock.unix_timestamp;
    clock.unix_timestamp = current_time + 15i64;
    svm.set_sysvar::<Clock>(&clock);

    contribute(&mut svm, &maker, &init_data, &contribute_data, None).unwrap();
    let fundraiser_account = svm.get_account(&init_data.fundraiser_pda.0).unwrap();
    let fundraiser_account = unsafe { &*(fundraiser_account.data.as_ptr() as *const FundRaiser) };
    let vault_account = svm.get_account(&init_data.vault).unwrap();
    let vault_account = unsafe { &*(vault_account.data.as_ptr() as *const Account) };
    msg!(
        "contributed amount: {}",
        contribute_data.contributing_amount
    );
    msg!(
        "reflected amount in fundraiser: {}",
        fundraiser_account.current_amount()
    );
    msg!("reflected amount in vault: {}", vault_account.amount);
    assert!(
        fundraiser_account.current_amount() >= contribute_data.contributing_amount,
        "Current contributed amount should be atleast contributing amount"
    );
    let contribution_account = svm
        .get_account(&contribute_data.contribution_pda.0)
        .unwrap();
    let contribution_account =
        unsafe { &*(contribution_account.data.as_ptr() as *const ContributorAccount) };
    assert!(
        contribution_account.contribution == contribute_data.contributing_amount.to_le_bytes(),
        "contribution account should have contributing amount"
    );
    assert!(
        contribution_account.contributor == contribute_data.contributor.pubkey().to_bytes(),
        "wrong contributor in contribution account"
    );

    msg!("Contribute after end time");
    svm.expire_blockhash();
    let mut clock = svm.get_sysvar::<Clock>();
    msg!("before update time: {}", clock.unix_timestamp);
    clock.unix_timestamp = init_data.start_time + init_data.duration as i64 + 30i64;
    svm.set_sysvar::<Clock>(&clock);
    msg!(
        "after update time: {}",
        svm.get_sysvar::<Clock>().unix_timestamp
    );
    msg!(
        "start time: {}, duration: {}",
        init_data.start_time,
        init_data.duration
    );

    let result = contribute(&mut svm, &maker, &init_data, &contribute_data, None);
    assert!(
        result.is_err(),
        "Contribute after start time + duration, should fail"
    );
}
