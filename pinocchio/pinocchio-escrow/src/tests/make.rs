use std::path::PathBuf;

use litesvm::{types::TransactionResult, LiteSVM};
use litesvm_token::{
    spl_token::{
        self,
        solana_program::{msg, rent::Rent, sysvar::SysvarId},
    },
    CreateAssociatedTokenAccount, CreateMint, MintTo,
};

use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

const PROGRAM_ID: &str = "4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT";
const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

pub(super) fn program_id() -> Pubkey {
    Pubkey::from(crate::ID)
}

pub(super) fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();

    svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Airdrop failed");

    // Load program SO file
    msg!("The path is!! {}", env!("CARGO_MANIFEST_DIR"));
    let so_path = PathBuf::from(format!(
        "{}/target/deploy/escrow.so",
        env!("CARGO_MANIFEST_DIR")
    ));
    msg!("The path is!! {:?}", so_path);

    let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

    svm.add_program(program_id(), &program_data);

    (svm, payer)
}

pub(super) struct MakerAssociatedValues {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub maker_ata_a: Pubkey,
    pub escrow: (Pubkey, u8),
    pub vault: Pubkey,
    pub amount_to_give: u64,
    pub amount_to_receive: u64,
    pub associated_token_program: Pubkey,
    pub system_program: Pubkey,
    pub token_program: Pubkey,
}

impl MakerAssociatedValues {
    pub(super) fn generate_associated_values(svm: &mut LiteSVM, payer: &Keypair) -> Self {
        let mint_a = CreateMint::new(svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();
        msg!("Mint A: {}", mint_a);

        let mint_b = CreateMint::new(svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();
        msg!("Mint B: {}", mint_b);

        // Create the maker's associated token account for Mint A
        let maker_ata_a = CreateAssociatedTokenAccount::new(svm, &payer, &mint_a)
            .owner(&payer.pubkey())
            .send()
            .unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let escrow = Pubkey::find_program_address(
            &[b"escrow".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );
        msg!("Escrow PDA: {}\n", escrow.0);

        // Derive the PDA for the vault associated token account using the escrow PDA and Mint A
        let vault = spl_associated_token_account::get_associated_token_address(
            &escrow.0, // owner will be the escrow PDA
            &mint_a,   // mint
        );
        msg!("Vault PDA: {}\n", vault);

        // Define program IDs for associated token program, token program, and system program
        let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(svm, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        let amount_to_receive: u64 = 100000000; // 100 tokens with 6 decimal places
        let amount_to_give: u64 = 500000000; // 500 tokens with 6 decimal places

        Self {
            mint_a,
            mint_b,
            maker_ata_a,
            escrow,
            vault,
            amount_to_give,
            amount_to_receive,
            associated_token_program,
            system_program,
            token_program,
        }
    }
}

#[test]
pub fn test_make_instruction() {
    let (mut svm, payer) = setup();

    let program_id = program_id();

    assert_eq!(program_id.to_string(), PROGRAM_ID);

    let values = MakerAssociatedValues::generate_associated_values(&mut svm, &payer);

    let tx = make_insn(&mut svm, &program_id, &payer, &values).unwrap();
    // Log transaction details
    msg!("\n\nMake transaction sucessfull");
    msg!("CUs Consumed: {}", tx.compute_units_consumed);
}

pub(super) fn make_insn(
    svm: &mut LiteSVM,
    program_id: &Pubkey,
    payer: &Keypair,
    values: &MakerAssociatedValues,
) -> TransactionResult {
    let bump: u8 = values.escrow.1;
    msg!("Bump: {}", bump);

    // Create the "Make" instruction to deposit tokens into the escrow
    let make_data = [
        vec![0u8], // Discriminator for "Make" instruction
        bump.to_le_bytes().to_vec(),
        values.amount_to_receive.to_le_bytes().to_vec(),
        values.amount_to_give.to_le_bytes().to_vec(),
    ]
    .concat();
    let make_ix = Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(values.mint_a, false),
            AccountMeta::new(values.mint_b, false),
            AccountMeta::new(values.escrow.0, false),
            AccountMeta::new(values.maker_ata_a, false),
            AccountMeta::new(values.vault, false),
            AccountMeta::new(values.system_program, false),
            AccountMeta::new(values.token_program, false),
            AccountMeta::new(values.associated_token_program, false),
            AccountMeta::new(Rent::id(), false),
        ],
        data: make_data,
    };

    // Create and send the transaction containing the "Make" instruction
    let message = Message::new(&[make_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();

    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    // Send the transaction and capture the result
    svm.send_transaction(transaction)
}
