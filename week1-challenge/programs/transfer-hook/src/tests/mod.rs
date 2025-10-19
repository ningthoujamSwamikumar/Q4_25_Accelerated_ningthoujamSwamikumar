use anchor_lang::{system_program, InstructionData};
use anchor_spl::token_2022::{
    self,
    spl_token_2022::{
        extension::{transfer_hook::TransferHook, BaseStateWithExtensions, StateWithExtensions},
        state::Mint,
    },
};
use litesvm::LiteSVM;
use solana_sdk::{
    account::Account,
    message::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

#[test]
fn test_init_mint() {
    // initialize the test environment
    let mut svm = LiteSVM::new();

    // deploy your program to the test environment
    let program_id = Pubkey::from(crate::ID.to_bytes());
    let program_bytes = include_bytes!("../../../../target/deploy/transfer_hook.so");
    svm.add_program(program_id, program_bytes)
        .expect("Failed to add program into svm");

    //deploy token 2022 to the test environment
    let token2022_program_id = Pubkey::from(token_2022::ID.to_bytes());
    let token2022_program_bytes = include_bytes!("../../token2022.so");
    svm.add_program(token2022_program_id, token2022_program_bytes)
        .expect("Failed deploying token 2022 program into svm");

    // create and fund test accounts
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Failed to airdrop payer");

    let mint_keypair = Keypair::new();

    let system_program_id = Pubkey::from(system_program::ID.to_bytes());
    let init_mint_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(mint_keypair.pubkey(), true),
            AccountMeta::new_readonly(system_program_id, false),
            AccountMeta::new_readonly(token2022_program_id, false),
        ],
        data: crate::instruction::InitMint {}.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[init_mint_ix],
        Some(&payer.pubkey()),
        &[&payer, &mint_keypair],
        svm.latest_blockhash(),
    );

    //send transaction
    let result = svm.send_transaction(tx).unwrap();

    println!("Transaction logs: {:?}", result.logs);

    let mint_account = svm.get_account(&mint_keypair.pubkey()).unwrap();
    assert_transfer_hook(&mint_account, &program_id);
}

fn assert_transfer_hook(mint: &Account, transfer_hook_id: &Pubkey) {
    //parse the mint account into a Mint with extensions
    let state_with_ext =
        StateWithExtensions::<Mint>::unpack(&mint.data).expect("Failed to unpack mint account");

    //extract the transfer hook extension
    let hook_ext = state_with_ext
        .get_extension::<TransferHook>()
        .expect("Failed to extract transfer hook extension");

    // now check the field
    assert!(
        hook_ext.program_id.0.to_bytes() == transfer_hook_id.to_bytes(),
        "Transfer Hook program mismatch"
    );
}
