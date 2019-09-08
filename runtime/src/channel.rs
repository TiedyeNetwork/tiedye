use codec::{Encode, Decode};
use rstd::prelude::*;
use rstd::convert::{TryInto};
#[cfg(not(feature = "std"))]
use rstd::alloc::borrow::ToOwned;
use support::{decl_module, decl_storage, decl_event, ensure, StorageMap, StorageValue, dispatch::Result};
use support::traits::{Currency, ReservableCurrency};
use system::ensure_signed;

use primitives::sr25519;
use primitives::crypto::Public;
use runtime_io::sr25519_verify;

fn to_u128(slice: &[u8]) -> u128 {
    slice.iter().rev().fold(0, |acc, &b| acc*2 + b as u128)
}

#[derive(Default, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Channel<AccountId, Balance, Moment> {
	sender: AccountId,
	signing_key: Vec<u8>,
	recipient: AccountId,
	start: Moment,
	collateral: Balance,
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait + timestamp::Trait {
	type Currency: ReservableCurrency<Self::AccountId>;
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as ChannelStorage {
		Channels get(channels): map u32 => Channel<T::AccountId, BalanceOf<T>, T::Moment>;

		KeyRegistry get(key_registry): map T::AccountId => Vec<u8>;

		NextFreeId: u32;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		// pub fn verify_signature(_origin, public_key: Vec<u8>, message: Vec<u8>, signature: Vec<u8>) -> Result {
		// 	// runtime_io::print( public_key.as_slice() );
		// 	// runtime_io::print( message.as_slice() );
		// 	// runtime_io::print( signature.as_slice() );

		// 	let sig = sr25519::Signature::from_slice(signature.as_slice());
		// 	let pub_key = sr25519::Public::from_slice(public_key.as_slice());

		// 	let ver: u8 = sr25519_verify(&sig, message.as_slice(), &pub_key).into();
		// 	// runtime_io::print( ver.into() );
		// 	ensure!(ver == 1, "Signature did not verify");

		// 	Ok(())
		// }

		// We insert the public key here, this way we make a distinction between the key being used for signing
		// and the key for the account. This is so that the account can remain secure while the signing key may be
		// delegated to a possible third party.
		pub fn one_way_channel(origin, public_key: Vec<u8>, recipient: T::AccountId, collateral: BalanceOf<T>) -> Result {
			let sender = ensure_signed(origin)?;

			T::Currency::reserve(&sender, collateral)?;

			let new_channel = Channel {
				sender: sender.clone(),
				signing_key: public_key.to_owned(),
				recipient: recipient,
				collateral: collateral,
				start: <timestamp::Module<T>>::now(),
			};

			let channel_id = Self::new_id();

			<Channels<T>>::insert(channel_id, new_channel);
			<KeyRegistry<T>>::insert(sender.clone(), public_key);

			Self::deposit_event(RawEvent::NewChannel(channel_id, sender.clone()));

			Ok(())
		}

		// Ideally we would put the channel_id into the message instead of two variables.
		pub fn close_one_way_channel(origin, public_key: Vec<u8>, channel_id: u32, amount: Vec<u8>, signature: Vec<u8>) -> Result {
			// There are two conditions for which a one-way channel could close:
			//  - Sender is closing. Sender could be submitting an expired state so we must keep the channel in a
			//    a dispute period for some length of time. During the dispute period, the recipient can submit a newer
			//	  and valid state signed by both parties. 
			//  - Recipient is closing. No matter who closes, we need a dispute period. The only case we will not need a 
			//    dispute period is if it was a strict rule in the second-layer protocol that value could increment
			//	  up and never decrease.
			//
			// For sake of simplicity we do not implement a dispute period here. We're still trying to get MVP shipped.
			// We only allow recipient to submit, and the message must be signed by sender.
			let recipient = ensure_signed(origin)?;

			let val: BalanceOf<T> = to_u128(amount.as_slice()).try_into().ok().unwrap();

			let channel = Self::channels(channel_id);

			// The off-chain logic should never allow this to possible, so the adjudication layer will throw it.
			ensure!(channel.collateral >= val, "Submitted an impossible state");

			// We may eventually want to change this to pay out the highest possible balance, in the case of something
			// like a slashing occurring on this user. The recipient of this channel would then need to bring it up
			// in the governance mechanism.
			ensure!(T::Currency::reserved_balance(&channel.sender) >= val, "Submitted an impossible state");

			// Now we need to make sure that the signature matches that of the sender. We can use the `is_signed`
			// helper function defined below.
			ensure!(Self::is_signed(public_key, amount, signature), "Invalid signature");

			Ok(())

		}
	}
}

impl<T: Trait> Module<T> {
	fn new_id() -> u32 {
		<NextFreeId>::mutate(|n| { let r = *n; *n += 1; r })
	}

	fn is_signed(pub_key: Vec<u8>, msg: Vec<u8>, sig: Vec<u8>) -> bool {
		let s = sr25519::Signature::from_slice(sig.as_slice());
		let p = sr25519::Public::from_slice(pub_key.as_slice());
		sr25519_verify(&s, msg.as_slice(), &p)
	}

	fn register_public(owner: T::AccountId, pub_key: Vec<u8>) -> bool {
		// Perform a check that this key is not already registered.
		true
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		NewChannel(u32, AccountId),
	}
);

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok, parameter_types};
	use sr_primitives::{traits::{BlakeTwo256, IdentityLookup}, testing::Header};
	use sr_primitives::weights::Weight;
	use sr_primitives::Perbill;

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for Test {
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type WeightMultiplierUpdate = ();
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
	}
	impl Trait for Test {
		type Event = ();
	}
	type TemplateModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			assert_eq!(TemplateModule::something(), Some(42));
		});
	}
}
