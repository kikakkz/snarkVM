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

impl<N: Network> Parser for Address<N> {
    /// Parses a string into an address.
    #[inline]
    fn parse(string: &str) -> ParserResult<Self> {
        // Prepare a parser for the Aleo address.
        let parse_address = recognize(pair(
            tag("aleo1"),
            many1(terminated(one_of("qpzry9x8gf2tvdw0s3jn54khce6mua7l"), many0(char('_')))),
        ));

        // Parse the address from the string.
        let (string, address) = map_res(parse_address, |address: &str| -> Result<_, Error> {
            Ok(Address::new(NativeAddress::from_str(&address.replace('_', ""))?))
        })(string)?;

        Ok((string, address))
    }
}

impl<N: Network> FromStr for Address<N> {
    type Err = Error;

    /// Parses a string into an address.
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

impl<N: Network> Debug for Address<N> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<N: Network> Display for Address<N> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_console_network::Testnet3;

    type CurrentNetwork = Testnet3;

    const ITERATIONS: u64 = 1_000;

    #[test]
    fn test_parse() -> Result<()> {
        // Ensure type and empty value fails.
        assert!(Address::<CurrentNetwork>::parse(&Address::<CurrentNetwork>::type_name()).is_err());
        assert!(Address::<CurrentNetwork>::parse("").is_err());

        for _ in 0..ITERATIONS {
            // Sample a new address.
            let private_key = snarkvm_console_account::PrivateKey::<CurrentNetwork>::new(&mut test_crypto_rng())?;
            let address = NativeAddress::try_from(private_key)?;

            let expected = format!("{}", address);
            let (remainder, candidate) = Address::<CurrentNetwork>::parse(&expected).unwrap();
            assert_eq!(format!("{expected}"), candidate.to_string());
            assert_eq!("", remainder);
        }
        Ok(())
    }

    #[test]
    fn test_display() -> Result<()> {
        for _ in 0..ITERATIONS {
            // Sample a new address.
            let private_key = snarkvm_console_account::PrivateKey::<CurrentNetwork>::new(&mut test_crypto_rng())?;
            let address = NativeAddress::try_from(private_key)?;

            let candidate = Address::<CurrentNetwork>::new(address);
            assert_eq!(format!("{address}"), format!("{candidate}"));

            let candidate_recovered = Address::<CurrentNetwork>::from_str(&format!("{candidate}")).unwrap();
            assert_eq!(candidate, candidate_recovered);
        }
        Ok(())
    }
}