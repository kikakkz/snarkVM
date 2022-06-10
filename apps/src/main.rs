// Copyright (C) 2019-2022 Aleo Systems Inc.
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

//! Generic PoSW Miner and Verifier, compatible with any implementer of the SNARK trait.

use snarkvm_dpc::{testnet2::Testnet2, BlockTemplate, Network, PoSWScheme};

use core::sync::atomic::AtomicBool;
use rand::thread_rng;

fn main() {
    // Construct the block template.
    let block = Testnet2::genesis_block();
    let block_template = BlockTemplate::new(
        block.previous_block_hash(),
        block.height(),
        block.timestamp(),
        // block.difficulty_target(),
        512,
        block.cumulative_weight(),
        block.previous_ledger_root(),
        block.transactions().clone(),
        block.to_coinbase_transaction().unwrap().to_records().next().unwrap(),
    );

    for n in 1..3 {
        println!("mine the {} block: {}", n, block_template.difficulty_target());
        // Construct a block header.
        let block_header = Testnet2::posw().mine(&block_template, &AtomicBool::new(false), &mut thread_rng()).unwrap();
        println!("mined a block: {}", block_header);

        assert!(Testnet2::posw().verify_from_block_header(&block_header));
    }
}
