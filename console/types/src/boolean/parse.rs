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

impl<N: Network> Parser for Boolean<N> {
    /// Parses a string into a boolean.
    #[inline]
    fn parse(string: &str) -> ParserResult<Self> {
        // Parse the boolean from the string.
        let (string, value) = alt((map(tag("true"), |_| true), map(tag("false"), |_| false)))(string)?;

        Ok((string, Boolean::new(value)))
    }
}

impl<N: Network> FromStr for Boolean<N> {
    type Err = Error;

    /// Parses a string into a boolean.
    #[inline]
    fn from_str(string: &str) -> Result<Self> {
        match Self::parse(string) {
            Ok((remainder, object)) => {
                // Ensure the remainder is empty.
                ensure!(remainder.is_empty(), "Failed to parse string. Found invalid character in: \"{remainder}\"");
                // Return the object.
                Ok(object)
            }
            Err(error) => bail!("Failed to parse string. {error}"),
        }
    }
}

impl<N: Network> Debug for Boolean<N> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<N: Network> Display for Boolean<N> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.boolean)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_console_network::Testnet3;

    type CurrentNetwork = Testnet3;

    #[test]
    fn test_parse() -> Result<()> {
        // Ensure type and empty value fails.
        assert!(Boolean::<CurrentNetwork>::parse(&Boolean::<CurrentNetwork>::type_name()).is_err());
        assert!(Boolean::<CurrentNetwork>::parse("").is_err());

        for boolean in &[true, false] {
            let expected = format!("{}", boolean);
            let (remainder, candidate) = Boolean::<CurrentNetwork>::parse(&expected).unwrap();
            assert_eq!(format!("{expected}"), candidate.to_string());
            assert_eq!("", remainder);
        }
        Ok(())
    }

    #[test]
    fn test_display() {
        /// Attempts to construct a boolean from the given element,
        /// format it in display mode, and recover a boolean from it.
        fn check_display<N: Network>(element: bool) {
            let candidate = Boolean::<N>::new(element);
            assert_eq!(format!("{element}"), format!("{candidate}"));

            let candidate_recovered = Boolean::<N>::from_str(&format!("{candidate}")).unwrap();
            assert_eq!(candidate, candidate_recovered);
        }

        check_display::<CurrentNetwork>(false);
        check_display::<CurrentNetwork>(true);
    }

    #[test]
    fn test_display_false() {
        // Constant
        let candidate = Boolean::<CurrentNetwork>::new(false);
        assert_eq!("false", &format!("{}", candidate));
    }

    #[test]
    fn test_display_true() {
        // Constant
        let candidate = Boolean::<CurrentNetwork>::new(true);
        assert_eq!("true", &format!("{}", candidate));
    }
}