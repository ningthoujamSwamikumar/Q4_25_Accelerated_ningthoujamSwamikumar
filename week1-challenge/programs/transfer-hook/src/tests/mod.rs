use anchor_lang::{system_program, AccountDeserialize, InstructionData};
use anchor_spl::{
    associated_token,
    token_2022::{
        self,
        spl_token_2022::{
            extension::{
                transfer_hook::TransferHook, BaseStateWithExtensions, StateWithExtensions,
            },
            instruction,
            state::Mint,
        },
    },
};
use litesvm::{types::TransactionResult, LiteSVM};
use litesvm_token::{
    CreateAssociatedTokenAccount, MintTo, MintToChecked, Transfer, TransferChecked,
};
use solana_sdk::{
    account::Account,
    message::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

use crate::WhitelistConfig;

struct Setup {
    pub svm: LiteSVM,
    pub program_id: Pubkey,
    pub token2022_program_id: Pubkey,
    pub payer: Keypair,
    pub mint_keypair: Keypair,
    pub system_program_id: Pubkey,
    pub associated_token_program_id: Pubkey,
}

impl Setup {
    fn new() -> Self {
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

        //deploy associated token program to the test environment
        let associated_token_program_id = Pubkey::from(associated_token::ID.to_bytes());
        let associated_token_program_bytes = include_bytes!("../../associated-token.so");
        svm.add_program(associated_token_program_id, associated_token_program_bytes)
            .expect("Failed deploying associated token program into svm");

        let system_program_id = Pubkey::from(system_program::ID.to_bytes());

        // create and fund test accounts
        let payer = Keypair::new();
        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop payer");

        let mint_keypair = Keypair::new();

        Self {
            mint_keypair,
            payer,
            program_id,
            svm,
            token2022_program_id,
            system_program_id,
            associated_token_program_id,
        }
    }
}

#[test]
fn test_init_mint() {
    let mut setup_values = Setup::new();
    let _result = init_mint(&mut setup_values).unwrap();

    //println!("Transaction logs: {:?}", result.logs);

    let mint_account = setup_values
        .svm
        .get_account(&setup_values.mint_keypair.pubkey())
        .unwrap();

    assert!(
        mint_account.lamports > 0,
        "Mint account should be rent exempt"
    );

    let mint_state = StateWithExtensions::<Mint>::unpack(&mint_account.data[..]).unwrap();
    let mint_authority = mint_state.base.mint_authority.unwrap();
    assert!(
        mint_authority.to_bytes() == setup_values.payer.pubkey().to_bytes(),
        "Unexpected mint authority"
    );
    assert_transfer_hook(&mint_account, &setup_values.program_id);

    println!("✅ init_mint successfully passed.");
}

fn init_mint(setup_values: &mut Setup) -> TransactionResult {
    let init_mint_ix = Instruction {
        program_id: setup_values.program_id,
        accounts: vec![
            AccountMeta::new(setup_values.payer.pubkey(), true),
            AccountMeta::new(setup_values.mint_keypair.pubkey(), true),
            AccountMeta::new_readonly(setup_values.system_program_id, false),
            AccountMeta::new_readonly(setup_values.token2022_program_id, false),
        ],
        data: crate::instruction::InitMint {}.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[init_mint_ix],
        Some(&setup_values.payer.pubkey()),
        &[&setup_values.payer, &setup_values.mint_keypair],
        setup_values.svm.latest_blockhash(),
    );

    //send transaction
    setup_values.svm.send_transaction(tx)
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

#[test]
fn test_init_extra_accounts() {
    let mut setup_values = Setup::new();
    init_mint(&mut setup_values).unwrap();

    let (result, extra_account_meta) = init_extra_accounts(&mut setup_values);
    result.unwrap();

    setup_values
        .svm
        .get_account(&extra_account_meta.0)
        .expect("No extra account found");
    // let data = extra_accounts_account.data;
    // let state = TlvStateBorrowed::unpack(&data[..]).expect("Failed to unpack tlv state");
    // let extra_meta_list = ExtraAccountMetaList::unpack_with_tlv_state::<ExecuteInstruction>(&state)
    //     .expect("Failed to unpack ");
}

fn init_extra_accounts(setup_values: &mut Setup) -> (TransactionResult, (Pubkey, u8)) {
    let extra_account_meta = Pubkey::find_program_address(
        &[
            b"extra-account-metas",
            setup_values.mint_keypair.pubkey().as_ref(),
        ],
        &setup_values.program_id,
    );
    let ix = Instruction {
        program_id: setup_values.program_id,
        accounts: vec![
            AccountMeta::new(setup_values.payer.pubkey(), true),
            AccountMeta::new(extra_account_meta.0, false),
            AccountMeta::new_readonly(setup_values.mint_keypair.pubkey(), false),
            AccountMeta::new_readonly(setup_values.system_program_id, false),
        ],
        data: crate::instruction::InitializeTransferHookAccounts {}.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&setup_values.payer.pubkey()),
        &[&setup_values.payer],
        setup_values.svm.latest_blockhash(),
    );

    let result = setup_values.svm.send_transaction(tx);
    (result, extra_account_meta)
}

#[test]
fn test_init_whitelist() {
    let mut setup_values = Setup::new();
    let (result, whitelist_config) = init_whitelist(&mut setup_values);
    result.unwrap();

    let account = setup_values
        .svm
        .get_account(&whitelist_config.0)
        .expect("No whitelist_config account found");
    let account_data = WhitelistConfig::try_deserialize(&mut &account.data[..])
        .expect("Failed to deserialize whitelist config");

    assert!(
        account_data.admin.to_bytes() == setup_values.payer.pubkey().to_bytes(),
        "Whitelist config admin should be the payer."
    );
    assert!(
        account_data.bump == whitelist_config.1,
        "Account bumps should match"
    );
}

fn init_whitelist(setup_values: &mut Setup) -> (TransactionResult, (Pubkey, u8)) {
    let whitelist_config =
        Pubkey::find_program_address(&[b"whitelist_config"], &setup_values.program_id);
    let ix = Instruction {
        program_id: setup_values.program_id,
        accounts: vec![
            AccountMeta::new(setup_values.payer.pubkey(), true),
            AccountMeta::new(whitelist_config.0, false),
            AccountMeta::new_readonly(setup_values.system_program_id, false),
        ],
        data: crate::instruction::InitWhitelist {}.data(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&setup_values.payer.pubkey()),
        &[&setup_values.payer],
        setup_values.svm.latest_blockhash(),
    );
    (setup_values.svm.send_transaction(tx), whitelist_config)
}

#[test]
fn test_add_whitelist() {
    let mut setup_values = Setup::new();

    init_mint(&mut setup_values).unwrap();
    let (result, whitelist_config) = init_whitelist(&mut setup_values);
    result.unwrap();

    let user = Keypair::new();
    let user_ata = CreateAssociatedTokenAccount::new(
        &mut setup_values.svm,
        &setup_values.payer,
        &setup_values.mint_keypair.pubkey(),
    )
    .owner(&user.pubkey())
    .token_program_id(&setup_values.token2022_program_id)
    .send()
    .unwrap();

    let (result, whitelist) = add_whitelist(&mut setup_values, &whitelist_config, &user, &user_ata);
    result.unwrap();

    let whitelist_account = setup_values
        .svm
        .get_account(&whitelist.0)
        .expect("whitelist account doesn't exist");
    let whitelist_data = crate::Whitelist::try_deserialize(&mut &whitelist_account.data[..])
        .expect("Failed to deserialize whitelist account data");
    assert!(
        whitelist_data.token_account.to_bytes() == user_ata.to_bytes(),
        "User token account should match"
    );
}

fn add_whitelist(
    setup_values: &mut Setup,
    whitelist_config: &(Pubkey, u8),
    user: &Keypair,
    token_acc: &Pubkey,
) -> (TransactionResult, (Pubkey, u8)) {
    let whitelist = Pubkey::find_program_address(
        &[
            b"whitelist",
            setup_values.mint_keypair.pubkey().as_ref(),
            user.pubkey().as_ref(),
        ],
        &setup_values.program_id,
    );
    let ix = Instruction {
        program_id: setup_values.program_id,
        accounts: vec![
            AccountMeta::new(setup_values.payer.pubkey(), true),
            AccountMeta::new_readonly(whitelist_config.0, false),
            AccountMeta::new_readonly(user.pubkey(), false),
            AccountMeta::new_readonly(setup_values.mint_keypair.pubkey(), false),
            AccountMeta::new(*token_acc, false),
            AccountMeta::new(whitelist.0, false),
            AccountMeta::new_readonly(setup_values.system_program_id, false),
            AccountMeta::new_readonly(setup_values.token2022_program_id, false),
        ],
        data: crate::instruction::AddToWhitelist {}.data(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&setup_values.payer.pubkey()),
        &[&setup_values.payer],
        setup_values.svm.latest_blockhash(),
    );
    (setup_values.svm.send_transaction(tx), whitelist)
}

#[test]
fn test_remove_whitelist() {
    let mut setup_values = Setup::new();

    init_mint(&mut setup_values).unwrap();
    let (result, whitelist_config) = init_whitelist(&mut setup_values);
    result.unwrap();

    let user = Keypair::new();
    let user_ata = CreateAssociatedTokenAccount::new(
        &mut setup_values.svm,
        &setup_values.payer,
        &setup_values.mint_keypair.pubkey(),
    )
    .owner(&user.pubkey())
    .token_program_id(&setup_values.token2022_program_id)
    .send()
    .unwrap();

    let (result, whitelist) = add_whitelist(&mut setup_values, &whitelist_config, &user, &user_ata);
    result.unwrap();

    remove_whitelist(&mut setup_values, &whitelist_config, &user, &user_ata).unwrap();

    let whitelist_account = setup_values.svm.get_account(&whitelist.0);
    assert!(
        whitelist_account.is_none(),
        "Whitelist account shouldn't exist"
    );
}

fn remove_whitelist(
    setup_values: &mut Setup,
    whitelist_config: &(Pubkey, u8),
    user: &Keypair,
    token_acc: &Pubkey,
) -> TransactionResult {
    let whitelist = Pubkey::find_program_address(
        &[
            b"whitelist",
            setup_values.mint_keypair.pubkey().as_ref(),
            user.pubkey().as_ref(),
        ],
        &setup_values.program_id,
    );
    let ix = Instruction {
        program_id: setup_values.program_id,
        accounts: vec![
            AccountMeta::new(setup_values.payer.pubkey(), true),
            AccountMeta::new_readonly(whitelist_config.0, false),
            AccountMeta::new_readonly(user.pubkey(), false),
            AccountMeta::new_readonly(setup_values.mint_keypair.pubkey(), false),
            AccountMeta::new(*token_acc, false),
            AccountMeta::new(whitelist.0, false),
            AccountMeta::new_readonly(setup_values.system_program_id, false),
            AccountMeta::new_readonly(setup_values.token2022_program_id, false),
        ],
        data: crate::instruction::RemoveFromWhitelist {}.data(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&setup_values.payer.pubkey()),
        &[&setup_values.payer],
        setup_values.svm.latest_blockhash(),
    );
    setup_values.svm.send_transaction(tx)
}

#[test]
fn test_transfer_hook() {
    let mut setup_values = Setup::new();
    init_mint(&mut setup_values).unwrap();
    init_extra_accounts(&mut setup_values).0.unwrap();

    let (result, whitelist_config) = init_whitelist(&mut setup_values);
    result.unwrap();

    let payer_ata = CreateAssociatedTokenAccount::new(
        &mut setup_values.svm,
        &setup_values.payer,
        &setup_values.mint_keypair.pubkey(),
    )
    .token_program_id(&setup_values.token2022_program_id)
    .send()
    .unwrap();

    let payer_ata_account = setup_values.svm.get_account(&payer_ata).unwrap();
    assert!(
        payer_ata_account.lamports > 0,
        "Payer ata should be rent exempted"
    );

    let payer_copy = setup_values.payer.insecure_clone();
    add_whitelist(
        &mut setup_values,
        &whitelist_config,
        &payer_copy,
        &payer_ata,
    )
    .0
    .unwrap();

    let dest_user = Keypair::new();
    setup_values
        .svm
        .airdrop(&dest_user.pubkey(), 10 * LAMPORTS_PER_SOL)
        .unwrap();
    let dest_ata = CreateAssociatedTokenAccount::new(
        &mut setup_values.svm,
        &dest_user,
        &setup_values.mint_keypair.pubkey(),
    )
    .token_program_id(&setup_values.token2022_program_id)
    .send()
    .unwrap();

    MintToChecked::new(
        &mut setup_values.svm,
        &setup_values.payer,
        &setup_values.mint_keypair.pubkey(),
        &payer_ata,
        50,
    )
    .token_program_id(&setup_values.token2022_program_id)
    .send()
    .unwrap();

    TransferChecked::new(
        &mut setup_values.svm,
        &setup_values.payer,
        &setup_values.mint_keypair.pubkey(),
        &dest_ata,
        30,
    )
    .token_program_id(&setup_values.token2022_program_id)
    .send()
    .unwrap();

    let dest_account = setup_values
        .svm
        .get_account(&dest_ata)
        .expect("Destination ata doesn't exist");
    let dest_ata_amount = u64::from_le_bytes(dest_account.data[64..72].try_into().unwrap());
    assert!(
        dest_ata_amount == 30,
        "Destination ata should have transfered amount."
    );
}
