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

mod parse;

use snarkvm_console_account::Address as NativeAddress;
use snarkvm_console_network::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Address<N: Network> {
    /// The underlying address.
    address: NativeAddress<N>,
}

impl<N: Network> AddressTrait for Address<N> {}

impl<N: Network> Address<N> {
    /// Initializes a new address.
    pub const fn new(address: NativeAddress<N>) -> Self {
        Self { address }
    }
}

impl<N: Network> TypeName for Address<N> {
    /// Returns the type name as a string.
    #[inline]
    fn type_name() -> &'static str {
        "address"
    }
}

impl<N: Network> Deref for Address<N> {
    type Target = NativeAddress<N>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.address
    }
}