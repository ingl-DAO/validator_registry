pub mod state;
pub mod utils;
use crate::state::constants::team;
use crate::state::{constants, Config, Storage, NameStorage};
use crate::utils::ResultExt;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program::invoke;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use solana_program::{bpf_loader_upgradeable, entrypoint, msg, system_program};
use utils::AccountInfoHelpers;

#[derive(BorshSerialize, BorshDeserialize)]
pub enum InstructionEnum {
    InitConfig,
    AddProgram { name: String },
    RemovePrograms { program_count: u8 },
    Reset,
    Blank,
}
impl InstructionEnum {
    pub fn decode(input: &[u8]) -> Self {
        let instruction = InstructionEnum::try_from_slice(input).unwrap();
        instruction
    }
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &constants::ID);
    let instruction = InstructionEnum::decode(input);
    match instruction {
        InstructionEnum::InitConfig => {
            let account_info_iter = &mut accounts.iter();
            let payer_account_info = next_account_info(account_info_iter)?;
            let config_account = next_account_info(account_info_iter)?;
            let (_config_key, config_bump) =
                config_account.assert_seed(program_id, &[b"config"])?;
            let config_data = Config::default();
            invoke_signed(
                &system_instruction::create_account(
                    payer_account_info.key,
                    config_account.key,
                    Rent::get()?.minimum_balance(config_data.get_space()),
                    config_data.get_space() as u64,
                    program_id,
                ),
                &[payer_account_info.clone(), config_account.clone()],
                &[&[b"config", &[config_bump]]],
            )?;
            config_data.serialize(&mut &mut config_account.data.borrow_mut()[..])?;
            Ok(())
        }
        InstructionEnum::AddProgram { mut name } => {
            let account_info_iter = &mut accounts.iter();
            let payer_account_info = next_account_info(account_info_iter)?;
            let config_account = next_account_info(account_info_iter)?;
            let registered_program_account = next_account_info(account_info_iter)?;
            let team_account_info = next_account_info(account_info_iter)?;
            registered_program_account
                .assert_owner(&bpf_loader_upgradeable::id())
                .error_log("Error @ registered_program_account assertion")?;
            assert!(registered_program_account.executable);
            team_account_info
                .assert_key_match(&team::id())
                .error_log("Error @ team_account_info assertion")?;
            let (_config_key, _config_bump) =
                config_account.assert_seed(program_id, &[b"config"])?;
            config_account.assert_owner(program_id)?;
            let mut config = Config::decode(config_account);
            let storage_account = next_account_info(account_info_iter)?;
            let storage_numeration = config
                .validator_numeration
                .checked_div(
                    constants::MAX_PROGRAMS_PER_STORAGE_ACCOUNT
                        .try_into()
                        .unwrap(),
                )
                .unwrap();
            msg!("storage_numeration: {}", storage_numeration);

            let name_storage_numeration = config
                .validator_numeration
                .checked_div(constants::MAX_PROGRAMS_PER_NAME_STORAGE_ACCOUNT.into())
                .unwrap();
            for ind in 0..name_storage_numeration + 1 {
                let current_name_storage_account = next_account_info(account_info_iter)?;
                let (_current_name_storage_key, current_name_storage_bump) = current_name_storage_account
                    .assert_seed(program_id, &[b"name_storage", &ind.to_be_bytes()])
                    .error_log("Error @ name_storage assertion")?;

                if config.validator_numeration % constants::MAX_PROGRAMS_PER_NAME_STORAGE_ACCOUNT == 0 && current_name_storage_account.owner == &system_program::id() {
                    let name_storage_data = Storage::default();
                    msg!("Creating new name storage account");
                    invoke_signed(
                        &system_instruction::create_account(
                            payer_account_info.key,
                            current_name_storage_account.key,
                            Rent::get()?.minimum_balance(name_storage_data.get_space()),
                            name_storage_data.get_space() as u64,
                            program_id,
                        ),
                        &[payer_account_info.clone(), current_name_storage_account.clone()],
                        &[&[
                            b"name_storage",
                            &ind.to_be_bytes(),
                            &[current_name_storage_bump],
                        ]],
                    )?;
                    name_storage_data.serialize(&mut &mut current_name_storage_account.data.borrow_mut()[..])?;
                }

                let mut name_storage = NameStorage::decode(current_name_storage_account);
                
                if name_storage.names.contains(&name){
                    msg!("Name already exists");
                    return Err(ProgramError::InvalidArgument);
                }

                if ind == name_storage_numeration {
                    name_storage.add_name(&name).error_log("Error @ name_storage.add_name")?;
                    name_storage.serialize(&mut &mut current_name_storage_account.data.borrow_mut()[..]).error_log("Error @ name_storage.serialize")?;
                }


                
            }

            let (_storage_key, storage_bump) = storage_account
                .assert_seed(program_id, &[b"storage", &storage_numeration.to_be_bytes()])
                .error_log("Error @ storage_account assertion")?;
            if config.validator_numeration % constants::MAX_PROGRAMS_PER_STORAGE_ACCOUNT == 0
                && storage_account.owner == &system_program::id()
            {
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
                    &[&[
                        b"storage",
                        &storage_numeration.to_be_bytes(),
                        &[storage_bump],
                    ]],
                )?;
                msg!("Created new storage account");
                storage_data
                    .serialize(&mut &mut storage_account.data.borrow_mut()[..])
                    .error_log("Error @ first storage serialization")?;
            }
            msg!("adding program to storage");
            let mut storage_data = Storage::decode(storage_account);
            storage_data
                .add_program(*registered_program_account.key)
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

            msg!("transferring Reallocing Storage account");
            let transfer_lamports: i128 = Rent::get()?.minimum_balance(storage_data.get_space())
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
                    .realloc(storage_data.get_space(), true)
                    .error_log("Error @ Reallocation of storage data")?;
            }
            storage_data
                .serialize(&mut &mut storage_account.data.borrow_mut()[..])
                .error_log("Error @ storage data serialization")?;
            config.validator_numeration += 1;
            config
                .serialize(&mut &mut config_account.data.borrow_mut()[..])
                .error_log("Error @ config account data serialization")?;
            Ok(())
        }

        InstructionEnum::Reset => {
            let account_info_iter = &mut accounts.iter();
            let _payer_account_info = next_account_info(account_info_iter)?;
            let config_account = next_account_info(account_info_iter)?;
            let (_config_key, _config_bump) =
                config_account.assert_seed(program_id, &[b"config"])?;
            config_account.assert_owner(program_id)?;
            let mut config = Config::decode(config_account);
            let storage_account = next_account_info(account_info_iter)?;
            let storage_numeration = config
                .validator_numeration
                .checked_div(constants::MAX_PROGRAMS_PER_STORAGE_ACCOUNT)
                .unwrap();
            msg!("storage_numeration: {}", storage_numeration);
            let (_storage_key, _storage_bump) = storage_account
                .assert_seed(program_id, &[b"storage", &storage_numeration.to_be_bytes()])
                .error_log("Error @ storage_account assertion")?;

            let storage_data = Storage::default();
            config.validator_numeration -=
                config.validator_numeration % constants::MAX_PROGRAMS_PER_STORAGE_ACCOUNT;

            storage_data
                .serialize(&mut &mut storage_account.data.borrow_mut()[..])
                .error_log("Error @ first storage serialization")?;
            config
                .serialize(&mut &mut config_account.data.borrow_mut()[..])
                .error_log("Error @ config account data serialization")?;
            Ok(())
        }

        _ => Err(ProgramError::InvalidInstructionData),
    }
}
