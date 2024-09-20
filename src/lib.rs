// Circular Doubly Linked List Solana Program

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    // program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    // system_instruction,
    // sysvar::{rent::Rent, Sysvar},
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Node {
    pub data: u64,
    pub prev: Pubkey,
    pub next: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum InstructionData {
    InitializeList,
    AddNode { data: u64 },
    RemoveNode { target_node: Pubkey },
}

impl InstructionData {
    pub fn try_to_vec(&self) -> Result<Vec<u8>, ProgramError> {
        let mut buf = Vec::with_capacity(std::mem::size_of::<Self>());
        self.serialize(&mut buf)?;
        Ok(buf)
    }
}
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = InstructionData::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    match instruction {
        InstructionData::InitializeList => {
            msg!("Instruction: InitializeList");
            initialize_list(program_id, accounts)
        }
        InstructionData::AddNode { data } => {
            msg!("Instruction: AddNode");
            add_node(program_id, accounts, data)
        }
        InstructionData::RemoveNode { target_node } => {
            msg!("Instruction: RemoveNode");
            remove_node(program_id, accounts, target_node)
        }
    }
}

fn initialize_list(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    msg!("Initializing Circular Doubly Linked List");

    let account_info_iter = &mut accounts.iter();
    let _initializer = next_account_info(account_info_iter)?;
    let head_account = next_account_info(account_info_iter)?;

    if head_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !head_account.data_is_empty() {
        if head_account.data.borrow().to_vec() != vec![0_u8; std::mem::size_of::<Node>()] {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
    }

    let node = Node {
        data: 0,
        prev: *head_account.key,
        next: *head_account.key,
    };

    node.serialize(&mut &mut head_account.data.borrow_mut()[..])?;

    Ok(())
}

fn add_node(program_id: &Pubkey, accounts: &[AccountInfo], data: u64) -> ProgramResult {
    msg!("Adding Node with data: {}", data);

    let account_info_iter = &mut accounts.iter();
    msg!("1");
    let head_account = next_account_info(account_info_iter)?; // Head node account
    msg!("2");
    let _tail_account = next_account_info(account_info_iter)?; // Account paying for the transaction
    msg!("3");
    msg!("head_account_key: {:?}", head_account.key);
    msg!("head_account: {:?}", head_account);
    let new_node_account = next_account_info(account_info_iter)?; // New node account
    msg!("4");

    if head_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Calculate the size needed for the Node struct
    // let node_size = std::mem::size_of::<Node>();

    // Calculate the minimum lamports required for rent exemption
    // let rent = Rent::get()?;
    // let required_lamports = rent.minimum_balance(node_size);
    msg!("5");
    let system_program = next_account_info(account_info_iter)?;
    msg!("system_program: {:?}", system_program);
    // Create the new node account
    // invoke(
    //     &system_instruction::create_account(
    //         payer_account.key,
    //         new_node_account.key,
    //         required_lamports,
    //         node_size as u64,
    //         program_id,
    //     ),
    //     &[
    //         payer_account.clone(),
    //         new_node_account.clone(),
    //         system_program.clone(),
    //     ],
    // )?;
    msg!("6");
    // Deserialize the head node without mutably borrowing
    let head_node = {
        let head_data = head_account.data.borrow();
        Node::try_from_slice(&head_data)?
    };
    msg!("6.5");
    let tail_account_key = head_node.prev;
    let tail_account = accounts
        .iter()
        .find(|a| *a.key == tail_account_key)
        .ok_or(ProgramError::InvalidAccountData)?;
    msg!("7");

    let new_node = Node {
        data,
        prev: tail_account_key,
        next: *head_account.key,
    };

    new_node.serialize(&mut &mut new_node_account.data.borrow_mut()[..])?;

    {
        msg!("8");
        let mut tail_data = tail_account.data.borrow_mut();
        msg!("9");
        let mut tail_node = Node::try_from_slice(&tail_data)?;
        msg!("10");
        tail_node.next = *new_node_account.key;
        msg!("11");
        tail_node.serialize(&mut &mut tail_data[..])?;
    } // Workaround for mutable borrow error
    msg!("12");

    {
        let mut head_data = head_account.data.borrow_mut();
        msg!("13");
        let mut head_node = Node::try_from_slice(&head_data)?;
        msg!("14");
        head_node.prev = *new_node_account.key;
        msg!("15");
        head_node.serialize(&mut &mut head_data[..])?;
        msg!("16");
    } // same here

    Ok(())
}

fn remove_node(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    target_node_key: Pubkey,
) -> ProgramResult {
    msg!("Removing Node: {}", target_node_key);

    let account_info_iter = &mut accounts.iter();
    let payer_account = next_account_info(account_info_iter)?; // Account paying for the transaction
    let head_account = next_account_info(account_info_iter)?; // Head node account
    let target_node_account = next_account_info(account_info_iter)?; // Target node account

    if head_account.owner != program_id || target_node_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize
    let target_data = target_node_account.data.borrow();
    let target_node = Node::try_from_slice(&target_data)?;

    // previous node
    {
        let next_account_key = target_node.next;
        let prev_account_key = target_node.prev;
        let prev_account = accounts
            .iter()
            .find(|a| *a.key == prev_account_key)
            .ok_or(ProgramError::InvalidAccountData)?;
        let mut prev_data = prev_account.data.borrow_mut();
        let mut prev_node = Node::try_from_slice(&prev_data)?;
        prev_node.next = next_account_key;
        prev_node.serialize(&mut &mut prev_data[..])?;
    }

    // next node
    {
        let next_account_key = target_node.next;
        let prev_account_key = target_node.prev;
        let next_account = accounts
            .iter()
            .find(|a| *a.key == next_account_key)
            .ok_or(ProgramError::InvalidAccountData)?;
        let mut next_data = next_account.data.borrow_mut();
        let mut next_node = Node::try_from_slice(&next_data)?;
        next_node.prev = prev_account_key;
        next_node.serialize(&mut &mut next_data[..])?;
        // If the target node is the head, update the head
        if *head_account.key == target_node_key {
            let mut head_data = head_account.data.borrow_mut();
            head_data.copy_from_slice(&next_data);
        }
    }

    // Update the previous and next nodes
    // prev_node.next = next_account_key;
    // let mut prev_data_cursor = Cursor::new(&mut **prev_data);
    // prev_node.serialize(&mut prev_data_cursor)?;

    // next_node.prev = prev_account_key;
    // let mut next_data_cursor = Cursor::new(&mut **next_data);
    // next_node.serialize(&mut next_data_cursor)?;

    // Deallocate the target node account
    **payer_account.lamports.borrow_mut() += **target_node_account.lamports.borrow();
    **target_node_account.lamports.borrow_mut() = 0;
    // target_node_account.data.borrow_mut().fill(0);

    Ok(())
}

// AbTEJTiFgZCMZyHWBMugkPJ4ZayGAxtoap4ChRfUKwv3

// tests
mod test;
