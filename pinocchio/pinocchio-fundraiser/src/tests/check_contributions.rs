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
    state::{fundraiser::FundRaiser, HasLen},
    tests::{
        contribute::{contribute, ContributeData},
        init::{initialize, program_id, setup, InitializeData},
    },
};

#[test]
pub fn test_check_contributions() {
    let (mut svm, maker) = setup();
    let init_data = InitializeData::initialize_data(&mut svm, &maker);
    initialize(&mut svm, &maker, &init_data).unwrap();

    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = init_data.start_time + 10i64;
    svm.set_sysvar::<Clock>(&clock);

    for i in 0..101 {
        svm.expire_blockhash();
        let contributor = Keypair::new();
        svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL)
            .unwrap();
        let ata = CreateAssociatedTokenAccount::new(&mut svm, &contributor, &init_data.mint)
            .send()
            .unwrap();
        MintTo::new(&mut svm, &maker, &init_data.mint, &ata, 1_000_000)
            .send()
            .unwrap();

        let contrib_data = ContributeData::generate_data(&mut svm, &maker, &init_data);
        contribute(&mut svm, &maker, &init_data, &contrib_data, Some(1_000_000)).unwrap();
        msg!("  #{} contribution", i);
    }

    let fundraiser_account = svm.get_account(&init_data.fundraiser_pda.0).unwrap();
    assert!(
        fundraiser_account.data.len() == FundRaiser::LEN,
        "fundraise data len should be matched"
    );
    let fundraiser_account = unsafe { &*(fundraiser_account.data.as_ptr() as *const FundRaiser) };
    msg!(
        "current amount: {}, and amount to raise: {}",
        fundraiser_account.current_amount(),
        fundraiser_account.amount_to_raise()
    );
    assert!(
        fundraiser_account.amount_to_raise() <= fundraiser_account.current_amount(),
        "Not enough amount contributed"
    );
    msg!("enough fund has been contributed to run check contri insn");
    //update time if time check is implemented in check_contri ixn

    let check_contri_data = CheckContriData::generate_check_data(&mut svm, &maker, &init_data);
    check_contributions(&mut svm, &maker, &init_data, &check_contri_data).unwrap();

    let maker_ata = svm.get_account(&check_contri_data.maker_ata).unwrap();
    let maker_ata = unsafe { &*(maker_ata.data.as_ptr() as *const Account) };
    assert!(
        maker_ata.amount >= fundraiser_account.amount_to_raise(),
        "maker ata should recieve all the raised amount"
    );
}

pub(super) struct CheckContriData {
    pub maker_ata: Pubkey,
}

impl CheckContriData {
    pub fn generate_check_data(
        svm: &mut LiteSVM,
        maker: &Keypair,
        init_data: &InitializeData,
    ) -> Self {
        let maker_ata = CreateAssociatedTokenAccount::new(svm, maker, &init_data.mint)
            .token_program_id(&init_data.token_program)
            .send()
            .unwrap();
        Self { maker_ata }
    }
}

pub(super) fn check_contributions(
    svm: &mut LiteSVM,
    maker: &Keypair,
    init_data: &InitializeData,
    check_contri_data: &CheckContriData,
) -> TransactionResult {
    let ix_data = [vec![3u8]].concat();
    let ix_accounts = [
        AccountMeta::new(maker.pubkey(), true),
        AccountMeta::new_readonly(init_data.mint, false),
        AccountMeta::new(init_data.fundraiser_pda.0, false),
        AccountMeta::new(check_contri_data.maker_ata, false),
        AccountMeta::new(init_data.vault, false),
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
        Some(&maker.pubkey()),
        &[maker],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx)
}
