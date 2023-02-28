pub mod add_program;
pub mod state;
pub mod utils;
use crate::add_program::add_program;
use crate::state::{constants, Config, NameStorage, Storage};
use crate::utils::ResultExt;
use borsh::{BorshDeserialize, BorshSerialize};

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
use solana_program::{entrypoint, msg};
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
    //TODO: use slicing for storage account storing and fetching.
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
        InstructionEnum::AddProgram { name } => {
            add_program(program_id, accounts, name)?;
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
            let name_storage_account = next_account_info(account_info_iter)?;
            let storage_numeration = config
                .validator_numeration
                .checked_div(constants::MAX_PROGRAMS_PER_STORAGE_ACCOUNT)
                .unwrap();

            let name_storage_numeration = config
                .validator_numeration
                .checked_div(constants::MAX_PROGRAMS_PER_NAME_STORAGE_ACCOUNT)
                .unwrap();

            msg!("storage_numeration: {}", storage_numeration);
            let (_storage_key, _storage_bump) = storage_account
                .assert_seed(program_id, &[b"storage", &storage_numeration.to_be_bytes()])
                .error_log("Error @ storage_account assertion")?;

            let (_name_storage_key, _name_storage_bump) = name_storage_account
                .assert_seed(
                    program_id,
                    &[b"name_storage", &name_storage_numeration.to_be_bytes()],
                )
                .error_log("Error @ name_storage_account assertion")?;

            let name_storage_data = NameStorage::default();

            let storage_data = Storage::default();
            config.validator_numeration -=
                config.validator_numeration % constants::MAX_PROGRAMS_PER_STORAGE_ACCOUNT;

            name_storage_data
                .serialize(&mut &mut name_storage_account.data.borrow_mut()[..])
                .error_log("Error @ first name storage serialization")?;

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
