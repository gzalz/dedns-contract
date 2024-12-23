use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};

const DISCRIMINATOR_ZONE: u8 = 1u8;
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Zone {
    owner: Pubkey,
    lamports_per_second: i64,
    min_lease_duration_secs: i64,
    domain: String,
    subdivided: bool,
}

const DISCRIMINATOR_LEASE: u8 = 2u8;
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Lease {
    zone_account: Pubkey,
    owner: Pubkey,
    domain: String,
    expiration: i64,
    expired: bool,
}

const DISCRIMINATOR_RECORD: u8 = 3u8;
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Record {
    host: String,
    ttl: i64,
    record_type: String,
    value: String,
}

const DISCRIMINATOR_DECOMISSION_LEASE: u8 = 4u8;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    if instruction_data[0] == DISCRIMINATOR_ZONE {
        let account = next_account_info(accounts_iter)?;
        if account.owner != program_id {
            msg!("Account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }
        let mut zone_account = Zone::try_from_slice(&instruction_data[1..])?;
        let mut account_data = account.data.borrow_mut();
        let mut zone_account_data = Vec::new();
        zone_account.serialize(&mut zone_account_data)?;
        let mut final_data = vec![DISCRIMINATOR_ZONE];
        final_data.extend_from_slice(&zone_account_data);
        account_data[..final_data.len()].copy_from_slice(&final_data);
    }
    if instruction_data[0] == DISCRIMINATOR_LEASE {
        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;
        let account = next_account_info(accounts_iter)?;
        if account.owner != program_id {
            msg!("Account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }
        let mut lease_account = Lease::try_from_slice(&instruction_data[1..])?;
        let duration = lease_account.expiration;
        lease_account.expiration += current_timestamp;
        lease_account.expired = false;
        let mut account_data = account.data.borrow_mut();
        let mut lease_account_data = Vec::new();
        lease_account.serialize(&mut lease_account_data)?;
        let mut final_data = vec![DISCRIMINATOR_LEASE];
        final_data.extend_from_slice(&lease_account_data);
        account_data[..final_data.len()].copy_from_slice(&final_data);
    }
    if instruction_data[0] == DISCRIMINATOR_RECORD {
        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;
        let account = next_account_info(accounts_iter)?;
        if account.owner != program_id {
            msg!("Account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }
        let mut record_account = Record::try_from_slice(&instruction_data[1..])?;
        record_account.ttl = 300;
        let mut account_data = account.data.borrow_mut();
        let mut record_account_data = Vec::new();
        record_account.serialize(&mut record_account_data)?;
        let mut final_data = vec![DISCRIMINATOR_RECORD];
        final_data.extend_from_slice(&record_account_data);
        account_data[..final_data.len()].copy_from_slice(&final_data);
    }
    if instruction_data[0] == DISCRIMINATOR_DECOMISSION_LEASE {
        let account = next_account_info(accounts_iter)?;
        if account.owner != program_id {
            msg!("Account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }
        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;
        let account_data: &mut [u8] = &mut account.data.borrow_mut();
        let mut account_data_slice = &account_data[1..];
        let mut lease_account_data = Vec::new();
        let mut lease_account = Lease::deserialize(&mut account_data_slice)?;
        if current_timestamp >= lease_account.expiration {
            lease_account.expired = true;
            lease_account.serialize(&mut lease_account_data)?;
            let mut final_data = vec![DISCRIMINATOR_LEASE];
            final_data.extend_from_slice(&lease_account_data);
            account_data[..final_data.len()].copy_from_slice(&final_data);
        }
    }
    Ok(())
}
