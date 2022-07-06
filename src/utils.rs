use std::io::Error;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use borsh::BorshDeserialize;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, program::{invoke, invoke_signed}, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, system_instruction, sysvar::{clock::Clock, rent::Rent, Sysvar}};

use crate::error::AppError;

pub fn now_timestamp() -> u64 {
    Clock::get().unwrap().unix_timestamp as u64
}

pub fn assert_eq_pubkey(account_info: &AccountInfo, account: &Pubkey) -> ProgramResult {
    if account_info.key != account {
        Err(AppError::InvalidEqPubkey.into())
    } else {
        Ok(())
    }
}

pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account.owner != owner {
        Err(AppError::InvalidOwner.into())
    } else {
        Ok(())
    }
}

pub fn assert_derivation(
    program_id: &Pubkey,
    account: &AccountInfo,
    path: &[&[u8]],
) -> Result<u8, ProgramError> {
    let (key, bump) = Pubkey::find_program_address(&path, program_id);
    if key != *account.key {
        return Err(AppError::InvalidDerivedKey.into());
    }
    Ok(bump)
}

pub fn assert_signer(account_info: &AccountInfo) -> ProgramResult {
    if !account_info.is_signer {
        Err(ProgramError::MissingRequiredSignature)
    } else {
        Ok(())
    }
}

pub fn get_random(seed: u8) -> Result<u64, ProgramError> {
    let clock = Clock::get()?;
    let mut hasher = DefaultHasher::new();
    hasher.write_u8(seed);
    hasher.write_u64(clock.slot);
    hasher.write_i64(clock.unix_timestamp);
    let mut random_value: [u8; 8] = [0u8; 8];
    random_value.copy_from_slice(&hasher.finish().to_le_bytes()[..8]);
    Ok(u64::from_le_bytes(random_value))
}

pub fn get_random_u8(seed: u8, divisor: u64) -> Result<u8, ProgramError> {
    let random = get_random(seed)?;
    Ok((random % divisor) as u8)
}

pub struct TokenTransferParams<'a: 'b, 'b> {
    /// source
    pub source: AccountInfo<'a>,
    /// destination
    pub destination: AccountInfo<'a>,
    /// amount
    pub amount: u64,
    /// authority
    pub authority: AccountInfo<'a>,
    /// authority_signer_seeds
    pub authority_signer_seeds: &'b [&'b [u8]],
    /// token_program
    pub token_program: AccountInfo<'a>,
}

#[inline(always)]
pub fn spl_token_transfer<'a>(
    token_program: AccountInfo<'a>,
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    amount: u64,
    signer_seeds: &[&[u8]],
) -> Result<(), ProgramError> {
    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, destination, authority, token_program],
        &[&signer_seeds],
    )
}

#[inline(always)]
pub fn create_or_allocate_account_raw<'a>(
    program_id: Pubkey,
    new_account_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    size: usize,
    signer_seeds: &[&[u8]],
) -> Result<(), ProgramError> {
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let required_lamports = rent
        .minimum_balance(size)
        .max(1)
        .saturating_sub(new_account_info.lamports());

    if required_lamports > 0 {
        msg!("Transfer {} lamports to the new account", required_lamports);
        invoke(
            &system_instruction::transfer(&payer_info.key, new_account_info.key, required_lamports),
            &[
                payer_info.clone(),
                new_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }

    msg!("Allocate space for the account");
    invoke_signed(
        &system_instruction::allocate(new_account_info.key, size.try_into().unwrap()),
        &[new_account_info.clone(), system_program_info.clone()],
        &[&signer_seeds],
    )?;

    msg!("Assign the account to the owning program");
    invoke_signed(
        &system_instruction::assign(new_account_info.key, &program_id),
        &[new_account_info.clone(), system_program_info.clone()],
        &[&signer_seeds],
    )?;
    msg!("Completed assignation!");

    Ok(())
}


#[inline(always)]
pub fn spl_token_create_account<'a>(
    token_program: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    mint_info: &AccountInfo<'a>,
    new_account: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    create_account_seeds: &[&[u8]],     // when account is not a pda, is null
    initialize_account_seeds: &[&[u8]], // when account is not a pda, is null
    rent_info: &AccountInfo<'a>,
) -> ProgramResult {
    let size = spl_token::state::Account::LEN;
    let rent = &Rent::from_account_info(&rent_info)?;
    let required_lamports = rent.minimum_balance(size);

    msg!("spl_token_create_account create");
    invoke_signed(
        &system_instruction::create_account(
            payer_info.key,
            new_account.key,
            required_lamports,
            size as u64,
            token_program.key,
        ),
        &[payer_info.clone(), new_account.clone()],
        &[create_account_seeds],
    )?;

    msg!("spl_token_create_account initialize");
    invoke_signed(
        &spl_token::instruction::initialize_account(token_program.key, new_account.key, mint_info.key, authority.key)?,
        &[
            token_program.clone(),
            new_account.clone(),
            mint_info.clone(),
            authority.clone(),
            rent_info.clone(),
        ],
        &[initialize_account_seeds],
    )?;
    msg!("spl_token_create_account success");

    Ok(())
}

pub fn try_from_slice_unchecked<T: BorshDeserialize>(data: &[u8]) -> Result<T, Error> {
    let mut data_mut = data;
    let result = T::deserialize(&mut data_mut)?;
    Ok(result)
}
