use litesvm::{types::TransactionResult, LiteSVM};
use litesvm_token::{CreateAssociatedTokenAccount, MintTo};
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

use crate::tests::make::{make_insn, program_id, setup, MakerAssociatedValues};

#[test]
pub fn test_take_instruction() {
    let program_id = program_id();
    let (mut svm, payer) = setup();
    let maker_data = MakerAssociatedValues::generate_associated_values(&mut svm, &payer);

    make_insn(&mut svm, &program_id, &payer, &maker_data).unwrap();

    let taker_data = TakerAssociatedValues::generate_associated_values(
        &mut svm,
        &payer,
        &payer.pubkey(),
        &maker_data,
    );

    let tx = take_insn(
        &mut svm,
        &program_id,
        &payer.pubkey(),
        &maker_data,
        &taker_data,
    )
    .unwrap();
    println!("Take insn successful with tx: {}", tx.signature);
}

pub(super) struct TakerAssociatedValues {
    pub taker: Keypair,
    pub taker_ata_a: Pubkey,
    pub taker_ata_b: Pubkey,
    pub maker_ata_b: Pubkey,
}

impl TakerAssociatedValues {
    pub(super) fn generate_associated_values(
        svm: &mut LiteSVM,
        mint_authority: &Keypair,
        maker: &Pubkey,
        maker_data: &MakerAssociatedValues,
    ) -> Self {
        let taker = Keypair::new();
        svm.airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

        let taker_ata_a = CreateAssociatedTokenAccount::new(svm, &taker, &maker_data.mint_a)
            .token_program_id(&maker_data.token_program)
            .send()
            .unwrap();
        let taker_ata_b = CreateAssociatedTokenAccount::new(svm, &taker, &maker_data.mint_b)
            .token_program_id(&maker_data.token_program)
            .send()
            .unwrap();
        let maker_ata_b = CreateAssociatedTokenAccount::new(svm, &taker, &maker_data.mint_b)
            .token_program_id(&maker_data.token_program)
            .owner(maker)
            .send()
            .unwrap();

        MintTo::new(
            svm,
            mint_authority,
            &maker_data.mint_b,
            &taker_ata_b,
            1000000000,
        )
        .send()
        .unwrap();

        Self {
            taker,
            taker_ata_a,
            taker_ata_b,
            maker_ata_b,
        }
    }
}

fn take_insn(
    svm: &mut LiteSVM,
    program_id: &Pubkey,
    maker: &Pubkey,
    maker_data: &MakerAssociatedValues,
    taker_data: &TakerAssociatedValues,
) -> TransactionResult {
    let bump = maker_data.escrow.1;
    let take_data = [vec![1u8], bump.to_le_bytes().to_vec()].concat();
    let take_accounts = [
        AccountMeta::new(taker_data.taker.pubkey(), true),
        AccountMeta::new_readonly(*maker, false), //escrow account creator
        AccountMeta::new_readonly(maker_data.mint_a, false), //receiving from escrow, had been deposited by maker
        AccountMeta::new_readonly(maker_data.mint_b, false), //sending to maker
        AccountMeta::new_readonly(maker_data.escrow.0, false), //escrow pda account
        AccountMeta::new(taker_data.taker_ata_a, false), //receiving token account of taker for mint a
        AccountMeta::new(taker_data.taker_ata_b, false), //sending token account of taker for mint b
        AccountMeta::new(taker_data.maker_ata_b, false), //transfer destination for token mint b
        AccountMeta::new(maker_data.vault, false),       //vault, stores token for mint_a from maker
        AccountMeta::new_readonly(maker_data.system_program, false),
        AccountMeta::new_readonly(maker_data.token_program, false),
        AccountMeta::new_readonly(maker_data.associated_token_program, false),
    ];
    let take_ix = Instruction {
        program_id: *program_id,
        accounts: take_accounts.to_vec(),
        data: take_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[take_ix],
        Some(&taker_data.taker.pubkey()),
        &[&taker_data.taker],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx)
}
