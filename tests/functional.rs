#![cfg(feature = "test-sbf")]

use {
    escrow::{
        instruction::escrow_tokens,
        state::{get_escrow_pda, Escrow},
    },
    mollusk_svm::{result::Check, Mollusk},
    solana_sdk::{account::Account, program_pack::Pack, pubkey::Pubkey, rent::Rent},
};

const DECIMALS: u8 = 0;
const SUPPLY: u64 = 500_000_000;
const AMOUNT: u64 = 100_000;

fn setup_mint(rent: &Rent) -> Account {
    let state = spl_token::state::Mint {
        decimals: DECIMALS,
        is_initialized: true,
        supply: SUPPLY,
        ..Default::default()
    };
    let mut data = vec![0u8; spl_token::state::Mint::LEN];
    state.pack_into_slice(&mut data);

    let lamports = rent.minimum_balance(data.len());

    Account {
        lamports,
        data,
        owner: spl_token::id(),
        ..Default::default()
    }
}

fn setup_token_account(rent: &Rent, mint: &Pubkey, owner: &Pubkey, amount: u64) -> Account {
    let state = spl_token::state::Account {
        mint: *mint,
        owner: *owner,
        amount,
        state: spl_token::state::AccountState::Initialized,
        ..Default::default()
    };
    let mut data = vec![0u8; spl_token::state::Account::LEN];
    state.pack_into_slice(&mut data);

    let lamports = rent.minimum_balance(data.len());

    Account {
        lamports,
        data,
        owner: spl_token::id(),
        ..Default::default()
    }
}

#[test]
fn test_escrow_tokens() {
    let program_id = Pubkey::new_unique();

    let sender = Pubkey::new_unique();
    let sender_token_address = Pubkey::new_unique();
    let escrow_token_address = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let escrow = get_escrow_pda(&program_id, &mint, &sender).0;

    let mut mollusk = Mollusk::new(&program_id, "escrow");
    mollusk_svm_programs_token::token::add_program(&mut mollusk);

    let expiration_slot = mollusk.sysvars.clock.slot + 10;
    let rent = &mollusk.sysvars.rent;

    let escrow_account = {
        let space = std::mem::size_of::<Escrow>();
        let lamports = mollusk.sysvars.rent.minimum_balance(space);
        Account::new(lamports, space, &program_id)
    };

    let mint_account = setup_mint(rent);
    let sender_token_account = setup_token_account(rent, &mint, &sender, AMOUNT);
    let escrow_token_account = setup_token_account(rent, &mint, &escrow, 0);

    let check_escrow_data = Escrow {
        amount: AMOUNT,
        expiration_slot,
        creator: sender,
    }
    .pack();

    let result = mollusk.process_and_validate_instruction(
        &escrow_tokens(
            &program_id,
            &sender,
            &sender_token_address,
            &escrow,
            &escrow_token_address,
            &mint,
            &spl_token::id(),
            AMOUNT,
            expiration_slot,
        ),
        &[
            (sender, Account::default()),
            (sender_token_address, sender_token_account),
            (escrow, escrow_account),
            (escrow_token_address, escrow_token_account),
            (mint, mint_account),
            mollusk_svm_programs_token::token::keyed_account(),
        ],
        &[
            Check::success(),
            Check::account(&escrow)
                .owner(&program_id)
                .data(&check_escrow_data)
                .build(),
        ],
    );

    let resulting_escrow_token_account = result.get_account(&escrow_token_address).unwrap();
    let unpacked_account_state =
        spl_token::state::Account::unpack(&resulting_escrow_token_account.data).unwrap();
    assert_eq!(unpacked_account_state.amount, AMOUNT);
}
