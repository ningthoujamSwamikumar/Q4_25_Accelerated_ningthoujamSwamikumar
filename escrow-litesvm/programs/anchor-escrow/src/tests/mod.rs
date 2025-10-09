#[cfg(test)]
mod tests {

    use {
        crate::{
            accounts::{Take, TakeAfterTime},
            instruction,
        },
        anchor_lang::{
            prelude::{msg, Clock},
            solana_program::program_pack::Pack,
            AccountDeserialize, InstructionData, Key, ToAccountMetas,
        },
        anchor_spl::{
            associated_token::{self, spl_associated_token_account},
            token::spl_token,
        },
        litesvm::LiteSVM,
        litesvm_token::{
            spl_token::ID as TOKEN_PROGRAM_ID, CreateAssociatedTokenAccount, CreateMint, MintTo,
        },
        solana_account::Account,
        solana_address::Address,
        solana_instruction::Instruction,
        solana_keypair::Keypair,
        solana_message::Message,
        solana_native_token::LAMPORTS_PER_SOL,
        solana_pubkey::Pubkey,
        solana_rpc_client::rpc_client::RpcClient,
        solana_sdk_ids::system_program::{self, ID as SYSTEM_PROGRAM_ID},
        solana_signer::Signer,
        solana_transaction::Transaction,
        std::{path::PathBuf, str::FromStr},
    };

    static PROGRAM_ID: Pubkey = crate::ID;
    const TOKEN_DECIMALS: u8 = 6;
    const DECIMALS_PER_TOKEN: u64 = 1000_000;

    fn setup() -> (LiteSVM, Keypair) {
        // Initialize LiteSVM and payer
        let mut program = LiteSVM::new();
        let payer = Keypair::new();

        // Airdrop some SOL to the payer keypair
        program
            .airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to payer");

        // Load program SO file
        let so_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/anchor_escrow.so");

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        program.add_program(PROGRAM_ID, &program_data);

        // Example on how to Load an account from devnet
        let rpc_client = RpcClient::new("https://api.devnet.solana.com");
        let account_address =
            Address::from_str("DRYvf71cbF2s5wgaJQvAGkghMkRcp5arvsK2w97vXhi2").unwrap();
        let fetched_account = rpc_client
            .get_account(&account_address)
            .expect("Failed to fetch account from devnet");

        program
            .set_account(
                payer.pubkey(),
                Account {
                    lamports: fetched_account.lamports,
                    data: fetched_account.data,
                    owner: Pubkey::from(fetched_account.owner.to_bytes()),
                    executable: fetched_account.executable,
                    rent_epoch: fetched_account.rent_epoch,
                },
            )
            .unwrap();

        msg!("Lamports of fetched account: {}", fetched_account.lamports);

        // Return the LiteSVM instance and payer keypair
        (program, payer)
    }

    #[test]
    fn test_make() {
        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let (mut program, payer) = setup();

        // Get the maker's public key from the payer keypair
        let maker = payer.pubkey();

        // Create two mints (Mint A and Mint B) with 6 decimal places and the maker as the authority
        let mint_a = CreateMint::new(&mut program, &payer)
            .decimals(6)
            .authority(&maker)
            .send()
            .unwrap();
        msg!("Mint A: {}\n", mint_a);

        let mint_b = CreateMint::new(&mut program, &payer)
            .decimals(6)
            .authority(&maker)
            .send()
            .unwrap();
        msg!("Mint B: {}\n", mint_b);

        // Create the maker's associated token account for Mint A
        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
            .owner(&maker)
            .send()
            .unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let escrow = Pubkey::find_program_address(
            &[b"escrow", maker.as_ref(), &123u64.to_le_bytes()],
            &PROGRAM_ID,
        )
        .0;
        msg!("Escrow PDA: {}\n", escrow);

        // Derive the PDA for the vault associated token account using the escrow PDA and Mint A
        let vault = associated_token::get_associated_token_address(&escrow, &mint_a);
        msg!("Vault PDA: {}\n", vault);

        // Define program IDs for associated token program, token program, and system program
        let asspciated_token_program = spl_associated_token_account::ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Make {
                maker: maker,
                mint_a: mint_a,
                mint_b: mint_b,
                maker_ata_a: maker_ata_a,
                escrow: escrow,
                vault: vault,
                associated_token_program: asspciated_token_program,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Make {
                deposit: 10,
                seed: 123u64,
                receive: 10,
            }
            .data(),
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nMake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        // Verify the vault account and escrow account data after the "Make" instruction
        let vault_account = program.get_account(&vault).unwrap();
        let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
        assert_eq!(vault_data.amount, 10);
        assert_eq!(vault_data.owner, escrow);
        assert_eq!(vault_data.mint, mint_a);

        let escrow_account = program.get_account(&escrow).unwrap();
        let escrow_data =
            crate::state::Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
        assert_eq!(escrow_data.seed, 123u64);
        assert_eq!(escrow_data.maker, maker);
        assert_eq!(escrow_data.mint_a, mint_a);
        assert_eq!(escrow_data.mint_b, mint_b);
        assert_eq!(escrow_data.receive, 10);
    }

    #[derive(Debug)]
    struct TestValues {
        taker: Keypair,
        mint_a: Pubkey,
        mint_b: Pubkey,
        maker_ata_a: Pubkey,
        maker_ata_b: Pubkey,
        taker_ata_a: Pubkey,
        taker_ata_b: Pubkey,
        escrow: Pubkey,
        vault: Pubkey,
        escrow_seed: u64,
    }

    impl TestValues {
        /// initializer: who initializes the escrow aka maker
        pub fn new(svm: &mut LiteSVM, maker: &Keypair) -> Self {
            let taker = Keypair::new();
            let mint_authority = Keypair::new();
            svm.airdrop(&mint_authority.pubkey(), 10 * LAMPORTS_PER_SOL)
                .unwrap();
            svm.airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

            let mint_a = CreateMint::new(svm, &mint_authority)
                .decimals(TOKEN_DECIMALS)
                .authority(&mint_authority.pubkey())
                .send()
                .unwrap();
            let mint_b = CreateMint::new(svm, &mint_authority)
                .decimals(TOKEN_DECIMALS)
                .authority(&mint_authority.pubkey())
                .send()
                .unwrap();
            let taker_ata_a = CreateAssociatedTokenAccount::new(svm, &taker, &mint_a)
                .owner(&taker.pubkey())
                .send()
                .unwrap();
            let taker_ata_b = CreateAssociatedTokenAccount::new(svm, &taker, &mint_b)
                .owner(&taker.pubkey())
                .send()
                .unwrap();
            let maker_ata_a = CreateAssociatedTokenAccount::new(svm, &maker, &mint_a)
                .owner(&maker.pubkey())
                .send()
                .unwrap();
            let maker_ata_b = CreateAssociatedTokenAccount::new(svm, &maker, &mint_b)
                .owner(&maker.pubkey())
                .send()
                .unwrap();
            let escrow_seed = 1234u64;
            let escrow = Pubkey::find_program_address(
                &[
                    b"escrow",
                    maker.pubkey().key().as_ref(),
                    &escrow_seed.to_le_bytes(),
                ],
                &PROGRAM_ID,
            )
            .0;
            let vault = associated_token::get_associated_token_address(&escrow, &mint_a);

            MintTo::new(
                svm,
                &mint_authority,
                &mint_a,
                &maker_ata_a,
                100u64 * DECIMALS_PER_TOKEN,
            )
            .send()
            .unwrap();
            MintTo::new(
                svm,
                &mint_authority,
                &mint_b,
                &taker_ata_b,
                100u64 * DECIMALS_PER_TOKEN,
            )
            .send()
            .unwrap();

            Self {
                taker,
                mint_a,
                mint_b,
                maker_ata_a,
                maker_ata_b,
                taker_ata_a,
                taker_ata_b,
                escrow,
                vault,
                escrow_seed,
            }
        }
    }

    #[test]
    fn test_take() {
        let (mut svm, maker) = setup();
        let test_values = TestValues::new(&mut svm, &maker);
        //make
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Make {
                maker: maker.pubkey(),
                mint_a: test_values.mint_a,
                mint_b: test_values.mint_b,
                maker_ata_a: test_values.maker_ata_a,
                escrow: test_values.escrow,
                vault: test_values.vault,
                associated_token_program: spl_associated_token_account::ID,
                token_program: TOKEN_PROGRAM_ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: instruction::Make {
                deposit: 10,
                seed: test_values.escrow_seed,
                receive: 10,
            }
            .data(),
        };

        let message = Message::new(&[make_ix], Some(&maker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let tx = Transaction::new(&[&maker], message, recent_blockhash);
        let tx_sig = svm.send_transaction(tx).unwrap();
        msg!("Make transaction successful: {:?}", tx_sig.signature);

        //take offer by the taker
        let take_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: Take {
                associated_token_program: spl_associated_token_account::ID,
                escrow: test_values.escrow,
                maker: maker.pubkey(),
                maker_ata_b: test_values.maker_ata_b,
                mint_a: test_values.mint_a,
                mint_b: test_values.mint_b,
                system_program: system_program::ID,
                taker: test_values.taker.pubkey(),
                taker_ata_a: test_values.taker_ata_a,
                taker_ata_b: test_values.taker_ata_b,
                token_program: TOKEN_PROGRAM_ID,
                vault: test_values.vault,
            }
            .to_account_metas(None),
            data: instruction::Take {}.data(),
        };
        let take_message = Message::new(&[take_ix], Some(&test_values.taker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let tx = Transaction::new(&[&test_values.taker], take_message, recent_blockhash);
        let tx_sig = svm.send_transaction(tx).unwrap();
        msg!("Take transaction successful: {:?}", tx_sig.signature);
    }

    #[test]
    fn test_refund() {
        let (mut svm, maker) = setup();
        let test_values = TestValues::new(&mut svm, &maker);
        //make
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Make {
                maker: maker.pubkey(),
                mint_a: test_values.mint_a,
                mint_b: test_values.mint_b,
                maker_ata_a: test_values.maker_ata_a,
                escrow: test_values.escrow,
                vault: test_values.vault,
                associated_token_program: spl_associated_token_account::ID,
                token_program: TOKEN_PROGRAM_ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: instruction::Make {
                deposit: 10,
                seed: test_values.escrow_seed,
                receive: 10,
            }
            .data(),
        };

        let message = Message::new(&[make_ix], Some(&maker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let tx = Transaction::new(&[&maker], message, recent_blockhash);
        let tx_sig = svm.send_transaction(tx).unwrap();
        msg!("Make transaction successful: {:?}", tx_sig.signature);

        //refund
        let refund_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Refund {
                escrow: test_values.escrow,
                maker: maker.pubkey(),
                maker_ata_a: test_values.maker_ata_a,
                mint_a: test_values.mint_a,
                system_program: system_program::ID,
                token_program: spl_token::ID,
                vault: test_values.vault,
            }
            .to_account_metas(None),
            data: instruction::Refund {}.data(),
        };
        let message = Message::new(&[refund_ix], Some(&maker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&maker], message, recent_blockhash);
        let tx_sig = svm.send_transaction(tx).unwrap();
        msg!("Refund successfull with tx sign: {:?}", tx_sig.signature);
    }

    #[test]
    fn take_after_time() {
        let (mut svm, maker) = setup();
        let test_values = TestValues::new(&mut svm, &maker);
        let mut initial_time = svm.get_sysvar::<Clock>();
        println!("initial time: {}", initial_time.unix_timestamp);
        //make
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Make {
                maker: maker.pubkey(),
                mint_a: test_values.mint_a,
                mint_b: test_values.mint_b,
                maker_ata_a: test_values.maker_ata_a,
                escrow: test_values.escrow,
                vault: test_values.vault,
                associated_token_program: spl_associated_token_account::ID,
                token_program: TOKEN_PROGRAM_ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: instruction::Make {
                deposit: 10,
                seed: test_values.escrow_seed,
                receive: 10,
            }
            .data(),
        };

        let message = Message::new(&[make_ix], Some(&maker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let tx = Transaction::new(&[&maker], message, recent_blockhash);
        let tx_sig = svm.send_transaction(tx).unwrap();
        msg!("Make transaction successful: {:?}", tx_sig.signature);

        //take offer by the taker
        let take_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: TakeAfterTime {
                associated_token_program: spl_associated_token_account::ID,
                escrow: test_values.escrow,
                maker: maker.pubkey(),
                maker_ata_b: test_values.maker_ata_b,
                mint_a: test_values.mint_a,
                mint_b: test_values.mint_b,
                system_program: system_program::ID,
                taker: test_values.taker.pubkey(),
                taker_ata_a: test_values.taker_ata_a,
                taker_ata_b: test_values.taker_ata_b,
                token_program: TOKEN_PROGRAM_ID,
                vault: test_values.vault,
            }
            .to_account_metas(None),
            data: instruction::TakeAfterTime {}.data(),
        };
        let take_message = Message::new(&[take_ix], Some(&test_values.taker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let tx = Transaction::new(&[&test_values.taker], take_message, recent_blockhash);
        let tx_sig = svm.send_transaction(tx);
        println!("tx_sig: {:?}", tx_sig);
        assert!(tx_sig.is_err());
        msg!("Take transaction before time failed");

        initial_time.unix_timestamp = initial_time.unix_timestamp.saturating_add(60 * 15 * 10000);
        svm.set_sysvar::<Clock>(&initial_time);
        println!("time set to {}", svm.get_sysvar::<Clock>().unix_timestamp);

        //take offer by the taker
        let take_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: TakeAfterTime {
                associated_token_program: spl_associated_token_account::ID,
                escrow: test_values.escrow,
                maker: maker.pubkey(),
                maker_ata_b: test_values.maker_ata_b,
                mint_a: test_values.mint_a,
                mint_b: test_values.mint_b,
                system_program: system_program::ID,
                taker: test_values.taker.pubkey(),
                taker_ata_a: test_values.taker_ata_a,
                taker_ata_b: test_values.taker_ata_b,
                token_program: TOKEN_PROGRAM_ID,
                vault: test_values.vault,
            }
            .to_account_metas(None),
            data: instruction::TakeAfterTime {}.data(),
        };
        let take_message = Message::new(&[take_ix], Some(&test_values.taker.pubkey()));
        svm.expire_blockhash();
        let recent_blockhash = svm.latest_blockhash();

        let tx = Transaction::new(&[&test_values.taker], take_message, recent_blockhash);
        let tx_sig = svm.send_transaction(tx).unwrap();
        msg!(
            "Take transaction successful after time threshold: {:?}",
            tx_sig.signature
        );
    }

    #[test]
    fn refund_before_time() {
        let (mut svm, maker) = setup();
        let test_values = TestValues::new(&mut svm, &maker);
        let mut initial_time = svm.get_sysvar::<Clock>();
        println!("initial time: {}", initial_time.unix_timestamp);

        //make
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Make {
                maker: maker.pubkey(),
                mint_a: test_values.mint_a,
                mint_b: test_values.mint_b,
                maker_ata_a: test_values.maker_ata_a,
                escrow: test_values.escrow,
                vault: test_values.vault,
                associated_token_program: spl_associated_token_account::ID,
                token_program: TOKEN_PROGRAM_ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: instruction::Make {
                deposit: 10,
                seed: test_values.escrow_seed,
                receive: 10,
            }
            .data(),
        };

        let message = Message::new(&[make_ix], Some(&maker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let tx = Transaction::new(&[&maker], message, recent_blockhash);
        let tx_sig = svm.send_transaction(tx).unwrap();
        msg!("Make transaction successful: {:?}", tx_sig.signature);

        initial_time.unix_timestamp = initial_time.unix_timestamp.saturating_add(60 * 5 * 1000);
        svm.set_sysvar::<Clock>(&initial_time);
        initial_time = svm.get_sysvar::<Clock>();
        println!("updated time: {}", initial_time.unix_timestamp);

        //refund before permissible time
        let refund_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::RefundBeforeTime {
                escrow: test_values.escrow,
                maker: maker.pubkey(),
                maker_ata_a: test_values.maker_ata_a,
                mint_a: test_values.mint_a,
                system_program: system_program::ID,
                token_program: spl_token::ID,
                vault: test_values.vault,
            }
            .to_account_metas(None),
            data: instruction::RefundBeforeTime {}.data(),
        };
        let message = Message::new(&[refund_ix], Some(&maker.pubkey()));
        svm.expire_blockhash();
        let recent_blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&maker], message, recent_blockhash);
        let tx_sig = svm.send_transaction(tx);
        assert!(tx_sig.is_ok());
        msg!(
            "Refund successfull before permissible time with tx sign: {:?}",
            tx_sig.unwrap().signature
        );

        initial_time.unix_timestamp = initial_time.unix_timestamp.saturating_add(60 * 30 * 1000);
        svm.set_sysvar::<Clock>(&initial_time);
        println!("updated time: {}", svm.get_sysvar::<Clock>().unix_timestamp);

        //refund after permissible time
        let refund_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::RefundBeforeTime {
                escrow: test_values.escrow,
                maker: maker.pubkey(),
                maker_ata_a: test_values.maker_ata_a,
                mint_a: test_values.mint_a,
                system_program: system_program::ID,
                token_program: spl_token::ID,
                vault: test_values.vault,
            }
            .to_account_metas(None),
            data: instruction::RefundBeforeTime {}.data(),
        };
        let message = Message::new(&[refund_ix], Some(&maker.pubkey()));
        svm.expire_blockhash();
        let recent_blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&maker], message, recent_blockhash);
        let tx_sig = svm.send_transaction(tx);
        assert!(tx_sig.is_err());
        msg!("Refund failed");
    }
}
