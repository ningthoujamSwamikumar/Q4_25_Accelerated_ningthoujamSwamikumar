use std::path::PathBuf; //using this only in test

use litesvm::{types::TransactionResult, LiteSVM};
use litesvm_token::{CreateMint, TOKEN_ID};
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

pub(super) fn program_id() -> Pubkey {
    Pubkey::from(crate::ID)
}

pub(super) fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let so_path = PathBuf::from(format!(
        "{}/target/deploy/pinocchio_fundraiser.so",
        env!("CARGO_MANIFEST_DIR")
    ));
    msg!("The path is!! {:?}", so_path);
    let program_data = std::fs::read(so_path).expect("Failed to read SO file");
    svm.add_program(program_id(), &program_data).unwrap();

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

    (svm, payer)
}

pub(super) struct InitializeData {
    pub mint: Pubkey,
    pub system_program: Pubkey,
    pub token_program: Pubkey,
    pub associated_token_program: Pubkey,
    pub fundraiser_pda: (Pubkey, u8),
    pub vault: Pubkey,
    pub amount_to_raise: u64,
    pub start_time: i64,
    pub duration: u8,
}

impl InitializeData {
    pub(super) fn initialize_data(svm: &mut LiteSVM, payer: &Keypair) -> Self {
        let system_program = Pubkey::from(pinocchio_system::id());
        let token_program = TOKEN_ID;
        let associated_token_program = Pubkey::from(pinocchio_associated_token_account::id());
        let mint = CreateMint::new(svm, payer)
            .authority(&payer.pubkey())
            .decimals(6)
            .send()
            .unwrap();
        let fundraiser_pda =
            Pubkey::find_program_address(&[b"fundraiser", payer.pubkey().as_ref()], &program_id());
        let vault = Pubkey::find_program_address(
            &[
                fundraiser_pda.0.as_ref(),
                token_program.as_ref(),
                mint.as_ref(),
            ],
            &associated_token_program,
        );

        let amount_to_raise: u64 = 100_000_000u64;
        let start_time: i64 = svm.get_sysvar::<Clock>().unix_timestamp + 10i64;
        let duration: u8 = 30;

        Self {
            mint,
            system_program,
            token_program,
            associated_token_program,
            fundraiser_pda,
            vault: vault.0,
            amount_to_raise,
            start_time,
            duration,
        }
    }
}

pub(super) fn initialize(
    svm: &mut LiteSVM,
    payer: &Keypair,
    data: &InitializeData,
) -> TransactionResult {
    let ix_data = [
        vec![0u8],
        data.amount_to_raise.to_le_bytes().to_vec(),
        data.start_time.to_le_bytes().to_vec(),
        data.duration.to_le_bytes().to_vec(),
    ]
    .concat();
    let ix_accounts = [
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new_readonly(data.mint, false),
        AccountMeta::new(data.fundraiser_pda.0, false),
        AccountMeta::new(data.vault, false),
        AccountMeta::new_readonly(data.system_program, false),
        AccountMeta::new_readonly(data.token_program, false),
        AccountMeta::new_readonly(data.associated_token_program, false),
    ];
    let ix = Instruction {
        program_id: program_id(),
        accounts: ix_accounts.to_vec(),
        data: ix_data,
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx)
}

#[test]
pub fn test_initialize() {
    let (mut svm, payer) = setup();
    let init_data = InitializeData::initialize_data(&mut svm, &payer);

    initialize(&mut svm, &payer, &init_data).unwrap();
}
