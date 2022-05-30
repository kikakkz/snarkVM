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

use super::*;

impl<N: Network> State<N> {
    /// Returns the serial number of the record.
    pub fn to_serial_number(
        &self,
        private_key: &PrivateKey<N>,
        program: N::Field,
        process: N::Field,
        data: N::Field,
        randomizer: N::Scalar,
    ) -> Result<SerialNumber<N>> {
        // Compute the commitment for the program state.
        let commitment = self.to_commitment(program, process, data)?;
        // Compute the serial number.
        SerialNumber::<N>::prove(&private_key.sk_vrf(), commitment, randomizer)
    }
}