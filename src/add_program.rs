use borsh::BorshSerialize;

use crate::state::constants::{team, MAX_NAME_LENGTH};
use crate::state::{constants, Config, NameStorage, Storage};
use crate::utils::{AccountInfoHelpers, ResultExt};
use solana_program::program::invoke;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use solana_program::{bpf_loader_upgradeable, msg, system_program};

pub fn add_permissionless_validator_program(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer_account_info = next_account_info(account_info_iter)?;
    let config_account = next_account_info(account_info_iter)?;
    let registered_program_account = next_account_info(account_info_iter)?;
    let team_account_info = next_account_info(account_info_iter)?;
    let storage_account = next_account_info(account_info_iter)?;
    let name_storage_account = next_account_info(account_info_iter)?;

    registered_program_account
        .assert_owner(&bpf_loader_upgradeable::id())
        .error_log("Error @ registered_program_account assertion")?;
    assert!(registered_program_account.executable);
    team_account_info
        .assert_key_match(&team::id())
        .error_log("Error @ team_account_info assertion")?;

    let (_storage_key, storage_bump) = storage_account
        .assert_seed(program_id, &[b"storage"])
        .error_log("Error @ storage_account assertion")?;
    let (_name_storage_key, name_storage_bump) =
        name_storage_account.assert_seed(program_id, &[b"name_storage"])?;

    let (_config_key, _config_bump) = config_account.assert_seed(program_id, &[b"config"])?;
    config_account.assert_owner(program_id)?;
    let mut config = Config::decode(config_account);

    if name_storage_account.owner == &system_program::id() {
        let name_storage_data = NameStorage::default();
        msg!("Creating new name storage account");
        invoke_signed(
            &system_instruction::create_account(
                payer_account_info.key,
                name_storage_account.key,
                Rent::get()?.minimum_balance(name_storage_data.get_space()),
                name_storage_data.get_space() as u64,
                program_id,
            ),
            &[payer_account_info.clone(), name_storage_account.clone()],
            &[&[b"name_storage", &[name_storage_bump]]],
        )?;
        name_storage_data.serialize(&mut &mut name_storage_account.data.borrow_mut()[..])?;
    }

    let mut name_storage = NameStorage::decode(name_storage_account);
    msg!("transferring Reallocing name Storage account");
    let transfer_lamports: i128 =
        Rent::get()?.minimum_balance(name_storage.get_space() + MAX_NAME_LENGTH) as i128
            - name_storage_account.lamports() as i128;
    if transfer_lamports > 0 {
        invoke(
            &system_instruction::transfer(
                payer_account_info.key,
                name_storage_account.key,
                transfer_lamports as u64,
            ),
            &[payer_account_info.clone(), name_storage_account.clone()],
        )?;
        msg!("Reallocated Storage account");
        name_storage_account
            .realloc(name_storage.get_space() + MAX_NAME_LENGTH, false)
            .error_log("Error @ Reallocation of storage data")?;
    }

    name_storage
        .add_name(&name, name_storage_account)
        .error_log("Error @ name_storage.add_name")?;

    if storage_account.owner == &system_program::id() {
        let storage_data = Storage::default();
        msg!("Creating new storage account");
        invoke_signed(
            &system_instruction::create_account(
                payer_account_info.key,
                storage_account.key,
                Rent::get()?.minimum_balance(storage_data.get_space()),
                storage_data.get_space() as u64,
                program_id,
            ),
            &[payer_account_info.clone(), storage_account.clone()],
            &[&[b"storage", &[storage_bump]]],
        )?;
        msg!("Created new storage account");
        storage_data
            .serialize(&mut &mut storage_account.data.borrow_mut()[..])
            .error_log("Error @ first storage serialization")?;
    }
    msg!("adding program to storage");
    let mut storage_data = Storage::decode(storage_account);
    msg!("transferring Reallocing Storage account");
    let transfer_lamports: i128 = Rent::get()?.minimum_balance(storage_data.get_space() + 32)
        as i128
        - storage_account.lamports() as i128;
    if transfer_lamports > 0 {
        invoke(
            &system_instruction::transfer(
                payer_account_info.key,
                storage_account.key,
                transfer_lamports as u64,
            ),
            &[payer_account_info.clone(), storage_account.clone()],
        )?;
        msg!("Reallocated Storage account");
        storage_account
            .realloc(storage_data.get_space() + 32, false)
            .error_log("Error @ Reallocation of storage data")?;
    }
    storage_data
        .add_program(*registered_program_account.key, storage_account)
        .error_log("Error while adding program to storage")?;

    //Transferring Spam prevention Sol
    msg!("Transferring Spam prevention Sol");
    invoke(
        &system_instruction::transfer(
            payer_account_info.key,
            team_account_info.key,
            constants::PROGRAM_DEPLOYMENT_PAYBACK,
        ),
        &[payer_account_info.clone(), team_account_info.clone()],
    )?;
    msg!("Transferred Spam prevention Sol");

    name_storage
        .serialize(&mut &mut name_storage_account.data.borrow_mut()[..])
        .error_log("Error @ name_storage.serialize")?;
    storage_data
        .serialize(&mut &mut storage_account.data.borrow_mut()[..])
        .error_log("Error @ storage data serialization")?;
    config.validator_numeration += 1;
    config
        .serialize(&mut &mut config_account.data.borrow_mut()[..])
        .error_log("Error @ config account data serialization")?;
    Ok(())
}

pub fn add_marketplace_program(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer_account_info = next_account_info(account_info_iter)?;
    let config_account = next_account_info(account_info_iter)?;
    let registered_program_account = next_account_info(account_info_iter)?;
    let team_account_info = next_account_info(account_info_iter)?;
    let storage_account = next_account_info(account_info_iter)?;

    registered_program_account
        .assert_owner(&bpf_loader_upgradeable::id())
        .error_log("Error @ registered_program_account assertion")?;
    assert!(registered_program_account.executable);
    team_account_info
        .assert_key_match(&team::id())
        .error_log("Error @ team_account_info assertion")?;

    let (_storage_key, storage_bump) = storage_account
        .assert_seed(program_id, &[b"storage"])
        .error_log("Error @ storage_account assertion")?;

    let (_config_key, _config_bump) = config_account.assert_seed(program_id, &[b"config"])?;
    config_account.assert_owner(program_id)?;
    let mut config = Config::decode(config_account);

    if storage_account.owner == &system_program::id() {
        let storage_data = Storage::default();
        msg!("Creating new storage account");
        invoke_signed(
            &system_instruction::create_account(
                payer_account_info.key,
                storage_account.key,
                Rent::get()?.minimum_balance(storage_data.get_space()),
                storage_data.get_space() as u64,
                program_id,
            ),
            &[payer_account_info.clone(), storage_account.clone()],
            &[&[b"storage", &[storage_bump]]],
        )?;
        msg!("Created new storage account");
        storage_data
            .serialize(&mut &mut storage_account.data.borrow_mut()[..])
            .error_log("Error @ first storage serialization")?;
    }
    msg!("adding program to storage");
    let mut storage_data = Storage::decode(storage_account);
    msg!("transferring Reallocing Storage account");
    let transfer_lamports: i128 = Rent::get()?.minimum_balance(storage_data.get_space() + 32)
        as i128
        - storage_account.lamports() as i128;
    if transfer_lamports > 0 {
        invoke(
            &system_instruction::transfer(
                payer_account_info.key,
                storage_account.key,
                transfer_lamports as u64,
            ),
            &[payer_account_info.clone(), storage_account.clone()],
        )?;
        msg!("Reallocated Storage account");
        storage_account
            .realloc(storage_data.get_space() + 32, false)
            .error_log("Error @ Reallocation of storage data")?;
    }
    storage_data
        .add_program(*registered_program_account.key, storage_account)
        .error_log("Error while adding program to storage")?;

    msg!("Transferring Spam prevention Sol");
    invoke(
        &system_instruction::transfer(
            payer_account_info.key,
            team_account_info.key,
            constants::PROGRAM_DEPLOYMENT_PAYBACK,
        ),
        &[payer_account_info.clone(), team_account_info.clone()],
    )?;
    msg!("Transferred Spam prevention Sol");

    storage_data
        .serialize(&mut &mut storage_account.data.borrow_mut()[..])
        .error_log("Error @ storage data serialization")?;
    config.validator_numeration += 1;
    config
        .serialize(&mut &mut config_account.data.borrow_mut()[..])
        .error_log("Error @ config account data serialization")?;
    Ok(())
}
