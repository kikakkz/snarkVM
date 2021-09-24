// Copyright (C) 2019-2021 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use crate::prelude::*;
use snarkvm_algorithms::{merkle_tree::MerklePath, prelude::*};
use snarkvm_utilities::has_duplicates;

use anyhow::Result;
use rand::{CryptoRng, Rng};
use rayon::prelude::*;
use std::marker::PhantomData;

pub struct DPC<C: Parameters>(PhantomData<C>);

impl<C: Parameters> DPCScheme<C> for DPC<C> {
    type Account = Account<C>;
    type Authorization = TransactionAuthorization<C>;
    type Execution = Execution<C>;
    type StateTransition = StateTransition<C>;
    type Transaction = Transaction<C>;

    /// Returns an authorization to execute a state transition.
    fn authorize<R: Rng + CryptoRng>(
        private_keys: &Vec<<Self::Account as AccountScheme>::PrivateKey>,
        state: &Self::StateTransition,
        rng: &mut R,
    ) -> Result<Self::Authorization> {
        // Keep a cursor for the private keys.
        let mut index = 0;

        // Construct the signature message.
        let signature_message = state.kernel().to_signature_message()?;

        // Sign the transaction kernel to authorize the transaction.
        let mut signatures = Vec::with_capacity(C::NUM_INPUT_RECORDS);
        for noop_private_key in state.noop_private_keys().iter().take(C::NUM_INPUT_RECORDS) {
            // Fetch the correct private key.
            let private_key = match noop_private_key {
                Some(noop_private_key) => noop_private_key,
                None => {
                    let private_key = &private_keys[index];
                    index += 1;
                    private_key
                }
            };

            // Sign the signature message.
            signatures.push(private_key.sign(&signature_message, rng)?);
        }

        // Return the transaction authorization.
        Ok(TransactionAuthorization::from(state, signatures))
    }

    /// Returns a transaction by executing an authorized state transition.
    fn execute<L: RecordCommitmentTree<C>, R: Rng + CryptoRng>(
        authorization: Self::Authorization,
        executables: &Vec<Executable<C>>,
        ledger: &L,
        rng: &mut R,
    ) -> Result<Self::Transaction> {
        assert_eq!(C::NUM_TOTAL_RECORDS, executables.len());

        let execution_timer = start_timer!(|| "DPC::execute");

        // Generate the local data.
        let local_data = authorization.to_local_data(rng)?;

        // Execute the programs.
        let mut executions = Vec::with_capacity(C::NUM_TOTAL_RECORDS);
        for (i, executable) in executables.iter().enumerate() {
            executions.push(executable.execute(i as u8, &local_data).unwrap());
        }

        // Compute the program commitment.
        let (program_commitment, program_randomness) = authorization.to_program_commitment(rng)?;

        // Compute the encrypted records.
        let (encrypted_records, encrypted_record_hashes, encrypted_record_randomizers) =
            authorization.to_encrypted_records(rng)?;

        let TransactionAuthorization {
            kernel,
            input_records,
            output_records,
            signatures,
        } = authorization;

        // Construct the ledger witnesses.
        let ledger_digest = ledger.latest_digest()?;

        // Compute the ledger membership witnesses.
        let mut input_witnesses = Vec::with_capacity(C::NUM_INPUT_RECORDS);
        for record in input_records.iter().take(C::NUM_INPUT_RECORDS) {
            input_witnesses.push(match record.is_dummy() {
                true => MerklePath::default(),
                false => ledger.prove_cm(&record.commitment())?,
            });
        }

        // Construct the inner circuit public and private variables.
        let inner_public_variables = InnerPublicVariables::new(
            &kernel,
            &ledger_digest,
            &encrypted_record_hashes,
            Some(program_commitment.clone()),
            Some(local_data.root().clone()),
        )?;
        let inner_private_variables = InnerPrivateVariables::new(
            input_records,
            input_witnesses,
            signatures,
            output_records.clone(),
            encrypted_record_randomizers,
            program_randomness.clone(),
            local_data.leaf_randomizers().clone(),
        );

        // Compute the inner circuit proof.
        let inner_proof = {
            let inner_proving_key = C::inner_circuit_proving_key(true)
                .as_ref()
                .ok_or(DPCError::MissingInnerProvingKey)?;

            C::InnerSNARK::prove(
                &inner_proving_key,
                &InnerCircuit::<C>::new(inner_public_variables.clone(), inner_private_variables),
                rng,
            )?
        };

        // Verify that the inner circuit proof passes.
        assert!(C::InnerSNARK::verify(
            C::inner_circuit_verifying_key(),
            &inner_public_variables,
            &inner_proof
        )?);

        let transaction_proof = {
            // Construct the outer circuit public and private variables.
            let outer_public_variables = OuterPublicVariables::new(&inner_public_variables, C::inner_circuit_id());
            let outer_private_variables = OuterPrivateVariables::new(
                C::inner_circuit_verifying_key().clone(),
                inner_proof,
                executions.to_vec(),
                program_commitment.clone(),
                program_randomness,
                local_data.root().clone(),
            );

            let outer_proving_key = C::outer_circuit_proving_key(true)
                .as_ref()
                .ok_or(DPCError::MissingOuterProvingKey)?;

            let outer_proof = C::OuterSNARK::prove(
                &outer_proving_key,
                &OuterCircuit::<C>::new(outer_public_variables.clone(), outer_private_variables),
                rng,
            )?;

            // Verify the outer circuit proof passes.
            assert!(C::OuterSNARK::verify(
                C::outer_circuit_verifying_key(),
                &outer_public_variables,
                &outer_proof
            )?);

            outer_proof
        };
        end_timer!(execution_timer);

        Ok(Self::Transaction::from(
            kernel,
            ledger_digest,
            C::inner_circuit_id().clone(),
            encrypted_records,
            transaction_proof,
        ))
    }

    fn verify<L: RecordCommitmentTree<C> + RecordSerialNumberTree<C>>(
        transaction: &Self::Transaction,
        ledger: &L,
    ) -> bool {
        let verify_time = start_timer!(|| "DPC::verify");

        // Returns false if the number of serial numbers in the transaction is incorrect.
        if transaction.serial_numbers().len() != C::NUM_INPUT_RECORDS {
            eprintln!("Transaction contains incorrect number of serial numbers");
            return false;
        }

        // Returns false if there are duplicate serial numbers in the transaction.
        if has_duplicates(transaction.serial_numbers().iter()) {
            eprintln!("Transaction contains duplicate serial numbers");
            return false;
        }

        // Returns false if the number of commitments in the transaction is incorrect.
        if transaction.commitments().len() != C::NUM_OUTPUT_RECORDS {
            eprintln!("Transaction contains incorrect number of commitments");
            return false;
        }

        // Returns false if there are duplicate commitments numbers in the transaction.
        if has_duplicates(transaction.commitments().iter()) {
            eprintln!("Transaction contains duplicate commitments");
            return false;
        }

        let ledger_time = start_timer!(|| "Ledger checks");

        // Returns false if any transaction serial number previously existed in the ledger.
        for sn in transaction.serial_numbers() {
            if ledger.contains_serial_number(sn) {
                eprintln!("Ledger already contains this transaction serial number.");
                return false;
            }
        }

        // Returns false if any transaction commitment previously existed in the ledger.
        for cm in transaction.commitments() {
            if ledger.contains_commitment(cm) {
                eprintln!("Ledger already contains this transaction commitment.");
                return false;
            }
        }

        // Returns false if the ledger digest in the transaction is invalid.
        if !ledger.is_valid_digest(&transaction.ledger_digest) {
            eprintln!("Ledger digest is invalid.");
            return false;
        }

        end_timer!(ledger_time);

        // Construct the ciphertext hashes

        // Returns false if the number of encrypted records in the transaction is incorrect.
        if transaction.encrypted_records().len() != C::NUM_OUTPUT_RECORDS {
            eprintln!("Transaction contains incorrect number of encrypted records");
            return false;
        }

        let outer_public_variables = match OuterPublicVariables::from(transaction) {
            Ok(outer_public_variables) => outer_public_variables,
            Err(error) => {
                eprintln!("Unable to construct outer public variables - {}", error);
                return false;
            }
        };

        match C::OuterSNARK::verify(
            C::outer_circuit_verifying_key(),
            &outer_public_variables,
            &transaction.proof,
        ) {
            Ok(is_valid) => {
                if !is_valid {
                    eprintln!("Transaction proof failed to verify.");
                    return false;
                }
            }
            Err(error) => {
                eprintln!(
                    "Outer circuit verifier failed to validate transaction proof: {:?}",
                    error
                );
                return false;
            }
        }

        end_timer!(verify_time);

        true
    }

    /// Returns true iff all the transactions in the block are valid according to the ledger.
    fn verify_transactions<L: RecordCommitmentTree<C> + RecordSerialNumberTree<C> + Sync>(
        transactions: &[Self::Transaction],
        ledger: &L,
    ) -> bool {
        transactions
            .as_parallel_slice()
            .par_iter()
            .all(|tx| Self::verify(tx, ledger))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_utilities::{FromBytes, ToBytes};

    use rand::SeedableRng;
    use rand_chacha::ChaChaRng;

    fn transaction_authorization_serialization_test<C: Parameters>() {
        let mut rng = ChaChaRng::seed_from_u64(1231275789u64);

        let recipient = Account::new(&mut rng).unwrap();
        let amount = AleoAmount::from_bytes(10);
        let state = StateTransition::new_coinbase(recipient.address, amount, &mut rng).unwrap();
        let authorization = DPC::<C>::authorize(&vec![], &state, &mut rng).unwrap();

        // Serialize and deserialize the transaction authorization.
        let deserialized_authorization = FromBytes::read_le(&authorization.to_bytes_le().unwrap()[..]).unwrap();
        assert_eq!(authorization, deserialized_authorization);
    }

    #[test]
    fn test_transaction_authorization_serialization() {
        transaction_authorization_serialization_test::<crate::testnet1::Testnet1Parameters>();
        transaction_authorization_serialization_test::<crate::testnet2::Testnet2Parameters>();
    }
}