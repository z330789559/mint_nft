use mpl_token_metadata::instruction::{create_master_edition_v3, create_metadata_accounts_v2};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::instruction::{initialize_mint, mint_to};

use crate::{utils::*};

pub fn process_mint(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority_info = next_account_info(account_info_iter)?;
    let signer_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let ata_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let ass_token_program_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;

    let metadata_program_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let edition_info = next_account_info(account_info_iter)?;

    assert_signer(&signer_info)?;
    let size = 82;
    let rent = &Rent::from_account_info(&rent_info)?;
    let required_lamports = rent.minimum_balance(size);

    msg!("Create Account");
    invoke(
        &system_instruction::create_account(
            signer_info.key,
            mint_info.key,
            required_lamports,
            size as u64,
            token_program_info.key,
        ),
        &[signer_info.clone(), mint_info.clone()],
    )?;

    msg!("Initialize Mint");
    invoke(
        &initialize_mint(
            token_program_info.key,
            mint_info.key,
            authority_info.key,
            Some(authority_info.key),
            0,
        )?,
        &[authority_info.clone(), mint_info.clone(), rent_info.clone(), token_program_info.clone(), ],
    )?;

    msg!("Create Associated Token Account");
    invoke(
        &create_associated_token_account(
            signer_info.key,
            signer_info.key,
            mint_info.key,
        ),
        &[
            signer_info.clone(),
            ata_info.clone(),
            ass_token_program_info.clone(),
            mint_info.clone(),
            token_program_info.clone(),
            system_info.clone()
        ],
    )?;

    msg!("Mint To");
    invoke(
        &mint_to(
            token_program_info.key,
            mint_info.key,
            ata_info.key,
            signer_info.key,
            &[signer_info.key],
            1,
        )?,
        &[
            signer_info.clone(),
            ata_info.clone(),
            mint_info.clone(),
            token_program_info.clone(),
            system_info.clone()
        ],
    )?;

    msg!("Create Metadata Account");
    let creator = vec![
        mpl_token_metadata::state::Creator {
            address: *signer_info.key,
            verified: false,
            share: 100,
        },
    ];
    let title = String::from("my_title");
    let symbol = String::from("my_symbol");
    let uri = String::from("https://arweave.net/y5e5DJsiwH0s_ayfMwYk-SnrZtVZzHLQDSTZ5dNRUHA");
    invoke(
        &create_metadata_accounts_v2(
            *metadata_program_info.key,
            *metadata_info.key,
            *mint_info.key,
            *signer_info.key,
            *signer_info.key,
            *signer_info.key,
            title,
            symbol,
            uri,
            Some(creator),
            1,
            true,
            false,
            None,
            None,
        ),
        &[
            metadata_info.clone(),
            mint_info.clone(),
            signer_info.clone(),
            metadata_program_info.clone(),
            token_program_info.clone(),
            system_info.clone(),
            rent_info.clone(),
        ],
    )?;

    msg!("Create Master Edition");
    invoke(
        &create_master_edition_v3(
            *metadata_program_info.key,
            *edition_info.key,
            *mint_info.key,
            *signer_info.key,
            *signer_info.key,
            *metadata_info.key,
            *signer_info.key,
            Some(0),
        ),
        &[
            edition_info.clone(),
            mint_info.clone(),
            signer_info.clone(),
            metadata_info.clone(),
            metadata_program_info.clone(),
            token_program_info.clone(),
            system_info.clone(),
            rent_info.clone(),
        ],
    )?;
    Ok(())
}