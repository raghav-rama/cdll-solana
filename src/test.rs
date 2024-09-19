#![allow(unused_imports)]

#[cfg(test)]
mod tests {
    use crate::{process_instruction, InstructionData, Node};

    use borsh::{BorshDeserialize, BorshSerialize};
    use solana_program::{
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
        system_instruction,
    };
    use solana_program_test::*;
    use solana_sdk::{
        account::Account,
        signature::{Keypair, Signer},
        system_program,
        transaction::Transaction,
        transport::TransportError,
    };

    #[tokio::test]
    async fn test_initialize_list() {
        let program_id = Pubkey::new_unique();
        let mut program_test = ProgramTest::new(
            "circular_doubly_ll_solana", // Replace with your program's name
            program_id,
            processor!(process_instruction),
        );

        // Create a new head account
        let head_account = Keypair::new();
        let node_size = std::mem::size_of::<Node>();
        let rent = Rent::default();
        let required_lamports = rent.minimum_balance(node_size);

        program_test.add_account(
            head_account.pubkey(),
            Account {
                lamports: required_lamports,
                data: vec![0_u8; node_size],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        );

        // let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let ProgramTestContext {
            mut banks_client,
            last_blockhash,
            payer,
            ..
        } = program_test.start_with_context().await;

        // Create InitializeList instruction
        let instruction_data = InstructionData::InitializeList.try_to_vec().unwrap();
        let instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(head_account.pubkey(), false),
            ],
            data: instruction_data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer],
            last_blockhash,
        );

        banks_client.process_transaction(transaction).await.unwrap();

        // Verify the head account data
        let head_account_data = banks_client
            .get_account(head_account.pubkey())
            .await
            .unwrap()
            .unwrap()
            .data;

        let head_node = Node::try_from_slice(&head_account_data).unwrap();

        assert_eq!(head_node.data, 0);
        assert_eq!(head_node.prev, head_account.pubkey());
        assert_eq!(head_node.next, head_account.pubkey());
    }

    #[tokio::test]
    async fn test_add_node() {
        let program_id = Pubkey::new_unique();
        let mut program_test = ProgramTest::new(
            "circular_doubly_ll_solana",
            program_id,
            processor!(process_instruction),
        );

        // Create head account
        let head_account = Keypair::new();
        let node_size = std::mem::size_of::<Node>();
        let rent = Rent::default();
        let required_lamports = rent.minimum_balance(node_size);

        program_test.add_account(
            head_account.pubkey(),
            Account {
                lamports: required_lamports,
                data: vec![0_u8; node_size],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        );

        // let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let ProgramTestContext {
            mut banks_client,
            last_blockhash,
            payer,
            ..
        } = program_test.start_with_context().await;

        // Initialize the list
        let instruction_data = InstructionData::InitializeList.try_to_vec().unwrap();
        let initialize_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(head_account.pubkey(), false),
            ],
            data: instruction_data,
        };

        let initialize_transaction = Transaction::new_signed_with_payer(
            &[initialize_instruction],
            Some(&payer.pubkey()),
            &[&payer],
            last_blockhash,
        );

        banks_client
            .process_transaction(initialize_transaction)
            .await
            .unwrap();

        // Add a new node
        let new_node_account = Keypair::new();

        // Fund the new node account
        let _create_new_node_account_ix = system_instruction::create_account(
            &payer.pubkey(),
            &new_node_account.pubkey(),
            required_lamports,
            node_size as u64,
            &program_id,
        );

        let add_node_instruction_data = InstructionData::AddNode { data: 42 }.try_to_vec().unwrap();
        println!("add_node_instruction_data: {:?}", add_node_instruction_data);
        let add_node_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(head_account.pubkey(), false),
                AccountMeta::new(new_node_account.pubkey(), false),
                AccountMeta::new(system_program::id(), false),
            ],
            data: add_node_instruction_data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[_create_new_node_account_ix, add_node_instruction],
            Some(&payer.pubkey()),
            &[&payer, &new_node_account],
            last_blockhash,
        );

        banks_client.process_transaction(transaction).await.unwrap();

        // Verify the head and new node
        let head_account_data = banks_client
            .get_account(head_account.pubkey())
            .await
            .unwrap()
            .unwrap()
            .data;

        let new_node_data = banks_client
            .get_account(new_node_account.pubkey())
            .await
            .unwrap()
            .unwrap()
            .data;

        let head_node = Node::try_from_slice(&head_account_data).unwrap();
        let new_node = Node::try_from_slice(&new_node_data).unwrap();
        {
            println!("new_node.prev: {:?}", new_node.prev);
            println!("head_node.prev: {:?}", head_node.prev);
            println!("new_node.next: {:?}", new_node.next);
            println!("head_node.next: {:?}", head_node.next);
        }

        assert_eq!(head_node.prev, new_node_account.pubkey());
        assert_eq!(new_node.next, head_account.pubkey());
        assert_eq!(new_node.data, 42);
        assert_eq!(new_node_account.pubkey(), head_node.prev);
        assert_eq!(head_account.pubkey(), new_node.next);
    }

    #[tokio::test]
    async fn test_remove_node() {
        let program_id = Pubkey::new_unique();
        let mut program_test = ProgramTest::new(
            "circular_doubly_ll_solana",
            program_id,
            processor!(process_instruction),
        );

        // Create accounts
        let head_account = Keypair::new();
        let node1_account = Keypair::new();
        let node2_account = Keypair::new();
        let node_size = std::mem::size_of::<Node>();
        let rent = Rent::default();
        let required_lamports = rent.minimum_balance(node_size);

        // Add accounts to test environment
        let accounts = vec![
            (head_account.pubkey(), head_account.insecure_clone()),
            (node1_account.pubkey(), node1_account.insecure_clone()),
            (node2_account.pubkey(), node2_account.insecure_clone()),
        ];

        for (pubkey, _account) in &accounts {
            program_test.add_account(
                *pubkey,
                Account {
                    lamports: required_lamports,
                    data: vec![0_u8; node_size],
                    owner: program_id,
                    executable: false,
                    rent_epoch: 0,
                },
            );
        }

        // let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let ProgramTestContext {
            mut banks_client,
            last_blockhash,
            payer,
            ..
        } = program_test.start_with_context().await;

        // Initialize the list
        let initialize_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(head_account.pubkey(), false),
            ],
            data: InstructionData::InitializeList.try_to_vec().unwrap(),
        };

        let initialize_transaction = Transaction::new_signed_with_payer(
            &[initialize_instruction],
            Some(&payer.pubkey()),
            &[&payer],
            last_blockhash,
        );

        banks_client
            .process_transaction(initialize_transaction)
            .await
            .unwrap();

        // Add node1
        let add_node1_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(head_account.pubkey(), false),
                AccountMeta::new(node1_account.pubkey(), false),
                AccountMeta::new(system_program::id(), false),
            ],
            data: InstructionData::AddNode { data: 100 }.try_to_vec().unwrap(),
        };

        let transaction1 = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &node1_account.pubkey(),
                    required_lamports,
                    node_size as u64,
                    &program_id,
                ),
                add_node1_instruction,
            ],
            Some(&payer.pubkey()),
            &[&payer, &node1_account],
            last_blockhash,
        );

        banks_client
            .process_transaction(transaction1)
            .await
            .unwrap();

        // Add node2
        let add_node2_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(head_account.pubkey(), false),
                AccountMeta::new(node2_account.pubkey(), false),
            ],
            data: InstructionData::AddNode { data: 200 }.try_to_vec().unwrap(),
        };

        let transaction2 = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &node2_account.pubkey(),
                    required_lamports,
                    node_size as u64,
                    &program_id,
                ),
                add_node2_instruction,
            ],
            Some(&payer.pubkey()),
            &[&payer, &node2_account],
            last_blockhash,
        );

        banks_client
            .process_transaction(transaction2)
            .await
            .unwrap();

        // Remove node1
        let remove_node1_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(head_account.pubkey(), false),
                AccountMeta::new(node1_account.pubkey(), false),
            ],
            data: InstructionData::RemoveNode {
                target_node: node1_account.pubkey(),
            }
            .try_to_vec()
            .unwrap(),
        };

        let transaction3 = Transaction::new_signed_with_payer(
            &[remove_node1_instruction],
            Some(&payer.pubkey()),
            &[&payer],
            last_blockhash,
        );

        banks_client
            .process_transaction(transaction3)
            .await
            .unwrap();

        // Verify node1 is removed
        let node1_account_data = banks_client
            .get_account(node1_account.pubkey())
            .await
            .unwrap();

        assert!(node1_account_data.is_none()); // Account should be deallocated

        // Verify head and node2 pointers
        let head_account_data = banks_client
            .get_account(head_account.pubkey())
            .await
            .unwrap()
            .unwrap()
            .data;

        let node2_account_data = banks_client
            .get_account(node2_account.pubkey())
            .await
            .unwrap()
            .unwrap()
            .data;

        let head_node = Node::try_from_slice(&head_account_data).unwrap();
        let node2_node = Node::try_from_slice(&node2_account_data).unwrap();

        assert_eq!(head_node.prev, node2_account.pubkey());
        assert_eq!(head_node.next, node2_account.pubkey());
        assert_eq!(node2_node.prev, head_account.pubkey());
        assert_eq!(node2_node.next, head_account.pubkey());
    }

    #[tokio::test]
    async fn test_full_list_operations() {
        // Test initializing, adding multiple nodes, and removing nodes in various orders
        let program_id = Pubkey::new_unique();
        let mut program_test = ProgramTest::new(
            "circular_doubly_ll_solana",
            program_id,
            processor!(process_instruction),
        );

        // Create head account
        let head_account = Keypair::new();
        let node_size = std::mem::size_of::<Node>();
        let rent = Rent::default();
        let required_lamports = rent.minimum_balance(node_size);

        program_test.add_account(
            head_account.pubkey(),
            Account {
                lamports: required_lamports,
                data: vec![0_u8; node_size],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        );

        // let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let ProgramTestContext {
            mut banks_client,
            last_blockhash,
            payer,
            ..
        } = program_test.start_with_context().await;

        // Initialize the list
        let initialize_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(head_account.pubkey(), false),
            ],
            data: InstructionData::InitializeList.try_to_vec().unwrap(),
        };

        let initialize_transaction = Transaction::new_signed_with_payer(
            &[initialize_instruction],
            Some(&payer.pubkey()),
            &[&payer],
            last_blockhash,
        );

        banks_client
            .process_transaction(initialize_transaction)
            .await
            .unwrap();

        // Add multiple nodes
        let mut node_accounts = Vec::new();
        for i in 1..=5 {
            let node_account = Keypair::new();
            node_accounts.push(node_account);

            let add_node_instruction = Instruction {
                program_id,
                accounts: vec![
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new(head_account.pubkey(), false),
                    AccountMeta::new(node_accounts[i - 1].pubkey(), false),
                ],
                data: InstructionData::AddNode { data: i as u64 }
                    .try_to_vec()
                    .unwrap(),
            };

            let transaction = Transaction::new_signed_with_payer(
                &[
                    system_instruction::create_account(
                        &payer.pubkey(),
                        &node_accounts[i - 1].pubkey(),
                        required_lamports,
                        node_size as u64,
                        &program_id,
                    ),
                    add_node_instruction,
                ],
                Some(&payer.pubkey()),
                &[&payer, &node_accounts[i - 1]],
                last_blockhash,
            );

            banks_client.process_transaction(transaction).await.unwrap();
        }

        // Remove nodes in reverse order
        for i in (1..=5).rev() {
            let remove_node_instruction = Instruction {
                program_id,
                accounts: vec![
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new(head_account.pubkey(), false),
                    AccountMeta::new(node_accounts[i - 1].pubkey(), false),
                ],
                data: InstructionData::RemoveNode {
                    target_node: node_accounts[i - 1].pubkey(),
                }
                .try_to_vec()
                .unwrap(),
            };

            let transaction = Transaction::new_signed_with_payer(
                &[remove_node_instruction],
                Some(&payer.pubkey()),
                &[&payer],
                last_blockhash,
            );

            banks_client.process_transaction(transaction).await.unwrap();

            // Verify the node is removed
            let node_account_data = banks_client
                .get_account(node_accounts[i - 1].pubkey())
                .await
                .unwrap();

            assert!(node_account_data.is_none());
        }

        // Verify that only the head node remains and points to itself
        let head_account_data = banks_client
            .get_account(head_account.pubkey())
            .await
            .unwrap()
            .unwrap()
            .data;

        let head_node = Node::try_from_slice(&head_account_data).unwrap();

        assert_eq!(head_node.prev, head_account.pubkey());
        assert_eq!(head_node.next, head_account.pubkey());
    }
}
