use bip39::{Mnemonic, Language};
use rand::Rng;
use bip32::{XPrv, ChildNumber, ExtendedPrivateKey};
use bitcoin::address::Address;
use bitcoin::Network;
use bitcoin::secp256k1::{Secp256k1, PublicKey as SecpPublicKey};
use bitcoin::CompressedPublicKey;

pub fn generate_seed_phrase(word_count: u8) -> Mnemonic {
    let mut rng = rand::thread_rng();
    let entropy = match word_count {
        12 => {
            let mut entropy = [0u8; 16];
            rng.fill(&mut entropy);
            entropy.to_vec()
        }
        24 => {
            let mut entropy = [0u8; 32];
            rng.fill(&mut entropy);
            entropy.to_vec()
        }
        _ => panic!("Invalid word count. Use 12 or 24."),
    };

    Mnemonic::from_entropy_in(Language::English, &entropy)
        .expect("Failed to generate mnemonic")
}

#[derive(Copy, Clone, Debug)]
pub enum AddressType {
    P2PKH,
    P2SH_P2WPKH,
    Bech32,
}

impl AddressType {
    /// Get the derivation path based on the address type.
    fn path_components(&self) -> Vec<ChildNumber> {
        let first_child = match self {
            AddressType::P2PKH => 44,
            AddressType::P2SH_P2WPKH => 49,
            AddressType::Bech32 => 84,
        };

        vec![
            ChildNumber::new(first_child, true).expect("Invalid child number"),
            ChildNumber::new(0, true).expect("Invalid child number"),
            ChildNumber::new(0, true).expect("Invalid child number"),
            ChildNumber::new(0, false).expect("Invalid child number"),
            ChildNumber::new(0, false).expect("Invalid child number"),
        ]
    }
}

pub fn derive_address(mnemonic: &Mnemonic, address_type: AddressType) -> Result<String, String> {
    // Convert mnemonic to seed
    let seed = mnemonic.to_seed("");
    let secp = Secp256k1::new();
    
    // Create extended private key from the seed
    let mut xprv = XPrv::new(&seed).map_err(|e| e.to_string())?;

    // Derive each component of the path manually based on address type
    for child in address_type.path_components() {
        xprv = xprv.derive_child(child).map_err(|e| e.to_string())?;
    }

    // Get the derived public key and convert to `CompressedPublicKey`
    let secp_pubkey = SecpPublicKey::from_slice(&xprv.public_key().to_bytes())
        .map_err(|e| e.to_string())?;
    let compressed_pubkey = CompressedPublicKey(secp_pubkey);

    // Generate the appropriate address based on the specified type
    let address = match address_type {
        AddressType::P2PKH => Address::p2pkh(&compressed_pubkey, Network::Bitcoin),
        AddressType::P2SH_P2WPKH => Address::p2shwpkh(&compressed_pubkey, Network::Bitcoin),
        AddressType::Bech32 => Address::p2wpkh(&compressed_pubkey, Network::Bitcoin),
    };

    Ok(address.to_string())
}
