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
use sr_primitives::traits::Member;
use sr_primitives::transaction_validity::{TransactionValidity, TransactionLongevity, ValidTransaction, InvalidTransaction};
use support::{decl_module, decl_event, decl_storage, Parameter, StorageMap, StorageValue};
use system::ensure_none;
use system::offchain::SubmitUnsignedTransaction;

pub mod sr25519 {
    mod app_sr25519 {
        use app_crypto::{app_crypto, sr25519};
        app_crypto!(sr25519, tiedye_primitives::ORACLE);

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

/// Error which may occur while executing the off-chain code.
#[cfg_attr(feature = "std", derive(Debug))]
enum OffchainErr {
	DecodeWorkerStatus,
	FailedSigning,
	NetworkState,
	SubmitTransaction,
}

impl support::Printable for OffchainErr {
	fn print(&self) {
		match self {
			OffchainErr::DecodeWorkerStatus => support::print("Offchain error: decoding WorkerStatus failed!"),
			OffchainErr::FailedSigning => support::print("Offchain error: signing failed!"),
			OffchainErr::NetworkState => support::print("Offchain error: fetching network state failed!"),
			OffchainErr::SubmitTransaction => support::print("Offchain error: submitting transaction failed!"),
		}
	}
}

pub type AuthIndex = u32;

#[derive(Default, Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct OracleMessage {
    value: u32,
    authority_index: AuthIndex,
}

pub trait Trait: system::Trait + timestamp::Trait + session::Trait {
    /// The identifier type for an authority.
    type AuthorityId: Member + Parameter + RuntimeAppPublic + Default + Ord;

    /// The overarching event type.
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;

    /// A dispatchable call type.
    type Call: From<Call<Self>>;

    /// A transaction submitter.
    type SubmitTransaction: SubmitUnsignedTransaction<Self, <Self as Trait>::Call>;
}

decl_event!(
    pub enum Event {
        Hi(),
    }
);

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

        fn update_feed(origin, message: OracleMessage, signature: <T::AuthorityId as RuntimeAppPublic>::Signature) {
            ensure_none(origin)?;
            support::print("update feed");
            
            if !<Results<T>>::exists(1) {
                support::print("doesn't exist");
                let mut o = OracleResult::default();
                o.values = vec![message.value];
                o.last_update = <timestamp::Module<T>>::now();
                <Results<T>>::insert(1, o);
                // <Results<T>>::insert(0, OracleResult { values: vec![value], last_update: <timestamp::Module<T>>::now() });
            } else {
                support::print("exists");
                <Results<T>>::mutate(1, |o| {
                    if o.values.len() < 100 {
                        o.values.push(message.value);
                    } else {
                        o.values.drain(0..1);
                        o.values.push(message.value);
                    }
                    o.last_update = <timestamp::Module<T>>::now();
                });
            }
        }

        // Runs after every block.
        fn offchain_worker(now: T::BlockNumber) {
            support::print("Hello from the offchain worker!");
            let value = Self::fetch("http://localhost:7666/mock");
            match Self::do_update(value) {
                Ok(_) => {},
                Err(err) => support::print(err),
            }
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

    fn do_update(value: u32) -> Result<(), OffchainErr> {
        support::print("Here is the value!");
        support::print(value as u64);
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
            let oracle_message = OracleMessage {
                value: value,
                authority_index: authority_index,
            };

            let signature = key.sign(&oracle_message.encode()).ok_or(OffchainErr::FailedSigning)?;
            let call = Call::update_feed(oracle_message, signature);
            T::SubmitTransaction::submit_unsigned(call)
                .map_err(|_| OffchainErr::SubmitTransaction)?;
        }
        Ok(())
    }

    fn initialize_keys(keys: &[T::AuthorityId]) {
        if !keys.is_empty() {
            assert!(Keys::<T>::get().is_empty(), "Keys are already initialized!");
            Keys::<T>::put_ref(keys);
        }
    }
}

impl<T: Trait> session::OneSessionHandler<T::AccountId> for Module<T> {
    type Key = T::AuthorityId;

    fn on_genesis_session<'a, I: 'a>(validators: I)
        where I: Iterator<Item=(&'a T::AccountId, T::AuthorityId)>
    {
        let keys = validators.map(|x| x.1).collect::<Vec<_>>();
        Self::initialize_keys(&keys);
    }

    fn on_new_session<'a, I: 'a>(_changed: bool, validators: I, _queued_validators: I)
		where I: Iterator<Item=(&'a T::AccountId, T::AuthorityId)>
	{
		// Remember who the authorities are for the new session.
		Keys::<T>::put(validators.map(|x| x.1).collect::<Vec<_>>());
	}

    fn on_before_session_ending() {
        // ignore
    }

    fn on_disabled(_i: usize) {
        // ignore
    }
}

impl<T: Trait> support::unsigned::ValidateUnsigned for Module<T> {
    type Call = Call<T>;

    fn validate_unsigned(call: &Self::Call) -> TransactionValidity {
        if let Call::update_feed(oracle_message, signature) = call {

            support::print("HERE");
            let current_session = <session::Module<T>>::current_index();

            let keys = Keys::<T>::get();
            let authority_id = match keys.get(oracle_message.authority_index as usize) {
                Some(id) => id,
                None => return InvalidTransaction::BadProof.into(),
            };

            let signature_valid = oracle_message.using_encoded(|encoded_message| {
                authority_id.verify(&encoded_message, &signature)
            });

            if !signature_valid {
                return InvalidTransaction::BadProof.into();
            }

            Ok(ValidTransaction {
                priority: 0,
                requires: vec![],
                provides: vec![(current_session, authority_id).encode()],
                longevity: TransactionLongevity::max_value(),
                propagate: true,
            })
        } else {
            support::print("ELSE");
            InvalidTransaction::Call.into()
        }
    }
}