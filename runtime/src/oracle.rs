//! Oracle Module
//! 
//! The Oracle module allows for the querying of offchain price feeds and recording thereof.
//! 
//! ## Interface
//! 
//! ### Public Functions
//! 
//! - `register_new_oracle` - Registers a new endpoint for which the system will begin to query. Requires a bond.

use codec::{Encode, Decode};
use rstd::prelude::*;
use sr_primitives::app_crypto::RuntimeAppPublic;
use primitives::crypto::KeyTypeId;
use sr_primitives::traits::Member;
use support::{decl_module, decl_event, decl_storage, Parameter, StorageMap, StorageValue, dispatch::Result};
use system::ensure_none;
use system::offchain::SubmitUnsignedTransaction;

// TODO: extract this out to its own primitives folder.
pub const ORACLE: KeyTypeId = KeyTypeId(*b"orac");

pub mod sr25519 {
    mod app_sr25519 {
        use app_crypto::{app_crypto, sr25519};
        app_crypto!(sr25519, crate::oracle::ORACLE);

        impl From<Signature> for sr_primitives::AnySignature {
            fn from(sig: Signature) -> Self {
                sr25519::Signature::from(sig).into()
            }
        }
    }

    // An oracle keypair using sr25519 crypto.
    #[cfg(feature = "std")]
    pub type AuthorityPair = app_sr25519::Pair;

    pub type AuthoritySignature = app_sr25519::Signature;

    pub type AuthorityId = app_sr25519::Public;
}

#[derive(Default, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct OracleResult<Moment> {
    values: Vec<u32>,
    // Last update.
    last_update: Moment,
}

pub trait Trait: system::Trait + timestamp::Trait {
    /// The identifier type for an authority.
    type AuthorityId: Member + Parameter + RuntimeAppPublic + Default + Ord;

    /// The overarching event type.
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;

    /// A dispatchable call type.
    type Call: From<Call<Self>>;

    /// A transaction submitter.
    type SubmitTransaction: SubmitUnsignedTransaction<Self, <Self as Trait>::Call>;
}

decl_storage! {
    trait Store for Module<T: Trait> as OracleStorage {
        /// The oracle endpoints to query.
        // Oracles get(oracles): map u32 => ;

        /// The current set of keys that can sign oracle fetching.
        Keys get(keys): Vec<T::AuthorityId>;

        /// The results from querying.
        Results get(results): map u32 => OracleResult<T::Moment>;

        NextFreeId: u32;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        fn update_feed(origin, value: u32, signature: <T::AuthorityId as RuntimeAppPublic>::Signature) {
            ensure_none(origin)?;
            
            if !<Results<T>>::exists(1) {
                support::print("doesn't exist");
                let mut o = OracleResult::default();
                o.values = vec![value];
                o.last_update = <timestamp::Module<T>>::now();
                <Results<T>>::insert(1, o);
                // <Results<T>>::insert(0, OracleResult { values: vec![value], last_update: <timestamp::Module<T>>::now() });
            } else {
                support::print("exists");
                <Results<T>>::mutate(1, |o| {
                    if o.values.len() < 100 {
                        o.values.push(value);
                    } else {
                        o.values.drain(0..1);
                        o.values.push(value);
                    }
                    o.last_update = <timestamp::Module<T>>::now();
                });
            }
        }

        // Runs after every block.
        fn offchain_worker(now: T::BlockNumber) {
            support::print("Hello from the offchain worker!");
            let value = Self::fetch("http://localhost:7666/mock");
            support::print(value as u64);
            Self::do_update(value);
        }
    }
}

impl<T: Trait> Module<T> {
    fn new_id() -> u32 {
        <NextFreeId>::mutate(|n| { let r = *n; *n +=1; r })
    }

    fn fetch(endpoint: &str) -> u32 {
        let request_id = runtime_io::http_request_start("GET", endpoint, &[]).unwrap();
        runtime_io::http_request_write_body(request_id, &[], None).unwrap_or(());
        runtime_io::http_response_wait(&[request_id], None);
        let buffer: &mut [u8] = &mut [0; 4];
        let size = runtime_io::http_response_read_body(request_id, buffer, None).unwrap_or(42);
        let mut result = &buffer[..];
        u32::decode(&mut result).unwrap()
    }

    fn do_update(value: u32) -> Result {
        let authorities = Keys::<T>::get();
        let mut local_keys = T::AuthorityId::all();
        local_keys.sort();

        for (authority_index, key) in authorities.into_iter()
            .enumerate()
            .filter_map(|(index, authority)| {
                local_keys.binary_search(&authority)
                    .ok()
                    .map(|location| (index as u32, &local_keys[location]))
            })
        {
            let oracle_data = value;

            let signature = key.sign(&oracle_data.encode()).ok_or("Incorrect")?;
            let call = Call::update_feed(oracle_data, signature);
            T::SubmitTransaction::submit_unsigned(call)
                .map_err(|_| "Unable to submit_unsigned.")?;
        }
        Ok(())
    }
}

decl_event!(
    pub enum Event {
        Hi(),
    }
);
