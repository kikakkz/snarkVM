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

use crate::{instructions::Instruction, BinaryOperation, Memory, Operation};
use snarkvm_circuits::{Equal as CircuitEqual, Literal, Parser, ParserResult};
use snarkvm_utilities::{FromBytes, ToBytes};

use core::fmt;
use nom::combinator::map;
use std::io::{Read, Result as IoResult, Write};

/// Checks that `first` is equal to `second`, storing the outcome in `destination`.
pub struct Equal<M: Memory> {
    operation: BinaryOperation<M::Environment>,
}

impl<M: Memory> Operation for Equal<M> {
    type Memory = M;

    /// Returns the opcode as a string.
    #[inline]
    fn mnemonic() -> &'static str {
        "eq"
    }

    /// Parses a string into an 'eq' operation.
    #[inline]
    fn parse(string: &str, memory: Self::Memory) -> ParserResult<Self> {
        // Parse the operation from the string.
        let (string, operation) = map(BinaryOperation::parse, |operation| Self { operation })(string)?;
        // Initialize the destination register.
        memory.initialize(operation.operation.destination());
        // Return the operation.
        Ok((string, operation))
    }

    /// Evaluates the operation in-place.
    #[inline]
    fn evaluate(&self, memory: &Self::Memory) {
        // Load the values for the first and second operands.
        let first = self.operation.first().load(memory);
        let second = self.operation.second().load(memory);

        // Perform the operation.
        let result = match (first, second) {
            (Literal::Field(a), Literal::Field(b)) => Literal::Boolean(a.is_equal(&b)),
            (Literal::Group(a), Literal::Group(b)) => Literal::Boolean(a.is_equal(&b)),
            (Literal::I8(a), Literal::I8(b)) => Literal::Boolean(a.is_equal(&b)),
            (Literal::I16(a), Literal::I16(b)) => Literal::Boolean(a.is_equal(&b)),
            (Literal::I32(a), Literal::I32(b)) => Literal::Boolean(a.is_equal(&b)),
            (Literal::I64(a), Literal::I64(b)) => Literal::Boolean(a.is_equal(&b)),
            (Literal::I128(a), Literal::I128(b)) => Literal::Boolean(a.is_equal(&b)),
            (Literal::Scalar(a), Literal::Scalar(b)) => Literal::Boolean(a.is_equal(&b)),
            (Literal::U8(a), Literal::U8(b)) => Literal::Boolean(a.is_equal(&b)),
            (Literal::U16(a), Literal::U16(b)) => Literal::Boolean(a.is_equal(&b)),
            (Literal::U32(a), Literal::U32(b)) => Literal::Boolean(a.is_equal(&b)),
            (Literal::U64(a), Literal::U64(b)) => Literal::Boolean(a.is_equal(&b)),
            (Literal::U128(a), Literal::U128(b)) => Literal::Boolean(a.is_equal(&b)),
            _ => Self::Memory::halt(format!("Invalid '{}' instruction", Self::mnemonic())),
        };

        memory.store(self.operation.destination(), result);
    }
}

impl<M: Memory> fmt::Display for Equal<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.operation)
    }
}

impl<M: Memory> FromBytes for Equal<M> {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        Ok(Self { operation: BinaryOperation::read_le(&mut reader)? })
    }
}

impl<M: Memory> ToBytes for Equal<M> {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.operation.write_le(&mut writer)
    }
}

#[allow(clippy::from_over_into)]
impl<M: Memory> Into<Instruction<M>> for Equal<M> {
    /// Converts the operation into an instruction.
    fn into(self) -> Instruction<M> {
        Instruction::Equal(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Input, Register, Stack};
    use snarkvm_circuits::Circuit;

    #[test]
    fn test_equal_field() {
        let first = Literal::<Circuit>::from_str("2field.public");
        let second = Literal::<Circuit>::from_str("2field.private");
        let expected = Literal::<Circuit>::from_str("true.private");

        let memory = Stack::<Circuit>::default();
        Input::from_str("input r0 field.public;", &memory).assign(first).evaluate(&memory);
        Input::from_str("input r1 field.private;", &memory).assign(second).evaluate(&memory);

        Equal::<Stack<Circuit>>::from_str("r2 r0 r1", &memory).evaluate(&memory);
        assert_eq!(expected, memory.load(&Register::new(2)));
    }

    #[test]
    fn test_equal_group() {
        let first = Literal::<Circuit>::from_str("2group.public");
        let second = Literal::<Circuit>::from_str("0group.private");
        let expected = Literal::<Circuit>::from_str("false.private");

        let memory = Stack::<Circuit>::default();
        Input::from_str("input r0 group.public;", &memory).assign(first).evaluate(&memory);
        Input::from_str("input r1 group.private;", &memory).assign(second).evaluate(&memory);

        Equal::<Stack<Circuit>>::from_str("r2 r0 r1", &memory).evaluate(&memory);
        assert_eq!(expected, memory.load(&Register::new(2)));
    }

    #[test]
    fn test_equal_scalar() {
        let first = Literal::<Circuit>::from_str("2scalar.public");
        let second = Literal::<Circuit>::from_str("2scalar.private");
        let expected = Literal::<Circuit>::from_str("true.private");

        let memory = Stack::<Circuit>::default();
        Input::from_str("input r0 scalar.public;", &memory).assign(first).evaluate(&memory);
        Input::from_str("input r1 scalar.private;", &memory).assign(second).evaluate(&memory);

        Equal::<Stack<Circuit>>::from_str("r2 r0 r1", &memory).evaluate(&memory);
        assert_eq!(expected, memory.load(&Register::new(2)));
    }
}