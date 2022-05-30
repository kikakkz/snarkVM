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

impl<N: Network> SerialNumber<N> {
    /// Returns a new NSEC5 proof, given a VRF secret key, an input, and a randomizer.
    pub fn prove(sk_vrf: &N::Scalar, commitment: N::Field, randomizer: N::Scalar) -> Result<Self> {
        // Compute the generator `H` as `HashToCurve(commitment)`.
        let generator_h = N::hash_to_group_psd2(&[commitment])?;

        // Compute `pk_vrf` as `sk_vrf * G`.
        let pk_vrf = N::g_scalar_multiply(sk_vrf);
        // Compute `gamma` as `sk_vrf * H`.
        let gamma = generator_h * *sk_vrf;
        // Compute `u` as `randomizer * G`.
        let u = N::g_scalar_multiply(&randomizer);
        // Compute `v` as `randomizer * H`.
        let v = generator_h * randomizer;

        // Prepare the preimage as `(pk_vrf, gamma, u, v)`, and use the x-coordinate of each affine point.
        let mut preimage = [pk_vrf, gamma, u, v];
        N::Projective::batch_normalization(&mut preimage);
        let [pk_vrf, gamma, u, v] = preimage.map(|c| c.to_affine());

        // Compute `challenge` as `HashToScalar(sk_vrf * G, gamma, randomizer * G, randomizer * H)`.
        let challenge = N::hash_to_scalar_psd4(&[pk_vrf, gamma, u, v].map(|c| c.to_x_coordinate()))?;
        // Compute `response` as `randomizer - challenge * sk_vrf`.
        let response = randomizer - challenge * sk_vrf;

        // Compute `output` as `HashToScalar(COFACTOR * gamma)`.
        let output = N::hash_to_scalar_psd4(&[gamma.mul_by_cofactor().to_x_coordinate()])?;

        // Return the proof.
        Ok(Self { output, proof: (gamma, challenge, response) })
    }
}