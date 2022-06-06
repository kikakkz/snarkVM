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

#![forbid(unsafe_code)]
#![allow(clippy::too_many_arguments)]

pub mod random;
pub use random::*;

mod sanitizer;
pub use sanitizer::Sanitizer;

pub mod traits;
pub use traits::*;

pub mod variable_length;
pub use variable_length::{read_variable_length_integer, variable_length_integer};

pub mod prelude {
    pub use super::*;

    pub use snarkvm_curves::{AffineCurve, ProjectiveCurve};
    pub use snarkvm_fields::{Field as _, PrimeField as _, SquareRootField as _, Zero as _};

    pub use core::{
        fmt::{self, Debug, Display, Formatter},
        str::{self, FromStr},
    };

    pub use anyhow::{anyhow, bail, ensure, Error, Result};

    pub use core::ops::{
        Add,
        AddAssign,
        BitAnd,
        BitAndAssign,
        BitOr,
        BitOrAssign,
        BitXor,
        BitXorAssign,
        Deref,
        Div,
        DivAssign,
        Mul,
        MulAssign,
        Neg,
        Not,
        Shl,
        ShlAssign,
        Shr,
        ShrAssign,
        Sub,
        SubAssign,
    };

    pub use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{alpha1, alphanumeric1, char, one_of},
        combinator::{map, map_res, opt, recognize},
        multi::{many0, many1, separated_list1},
        sequence::{pair, terminated},
    };

    pub use num_traits::{One, Pow, Zero};

    pub use rand::{distributions::Alphanumeric, CryptoRng, Rng};
}