use codec::{Encode, Decode};
use rstd::prelude::*;
use support::{decl_module, decl_storage, decl_event, ensure, StorageMap, StorageValue, Parameter, dispatch::Result};
use support::traits::{Currency, ReservableCurrency};
use system::ensure_signed;

use primitives::sr25519;
use primitives::crypto::Public;
use runtime_io::sr25519_verify;

// #[cfg(feature = "std")]
// use schnorrkel::{Keypair, PublicKey, MiniSecretKey, MINI_SECRET_KEY_LENGTH, ExpansionMode, signing_context, Signature};

#[derive(Default, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Channel<AccountId, Balance, Moment> {
	sender: AccountId,
	recipient: AccountId,
	start: Moment,
	collateral: Balance,
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait + timestamp::Trait {
	// type ChannelId: Member + Parameter + Default + Copy;
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Currency: ReservableCurrency<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as ChannelStorage {
		Channels get(channels): map u32 => Channel<T::AccountId, BalanceOf<T>, T::Moment>;

		NextFreeId: u32;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		pub fn verify_signature(origin, public_key: Vec<u8>, message: Vec<u8>, signature: Vec<u8>) -> Result {
			// runtime_io::print( public_key.as_slice() );
			// runtime_io::print( message.as_slice() );
			// runtime_io::print( signature.as_slice() );

			let sig = sr25519::Signature::from_slice(signature.as_slice());
			let pub_key = sr25519::Public::from_slice(public_key.as_slice());

			let ver: u8 = sr25519_verify(&sig, message.as_slice(), &pub_key).into();
			// runtime_io::print( ver.into() );
			ensure!(ver == 1, "Signature did not verify");

			Ok(())
		}

		pub fn one_way_channel(origin, recipient: T::AccountId, collateral: BalanceOf<T>) -> Result {
			let sender = ensure_signed(origin)?;

			T::Currency::reserve(&sender, collateral)?;

			let new_channel = Channel {
				sender: sender.clone(),
				recipient: recipient,
				collateral: collateral,
				start: <timestamp::Module<T>>::now(),
			};

			let channel_id = Self::new_id();

			<Channels<T>>::insert(channel_id, new_channel);

			Self::deposit_event(RawEvent::NewChannel(channel_id, sender.clone()));

			Ok(())
		}

		// pub fn close_channel(origin, signature: Vec<u8>, msg: Vec<u8>) -> Result {
		// 	let sender = ensure_signed(origin)?;

		// 	let closing_party;
		// 	if sender == Self::channel().sender {
		// 		closing_party = 0;
		// 	} else if sender == Self::channel().recipient {
		// 		closing_party = 1;
		// 	} else { return Ok(()); }

		// 	let mut sig = sr25519::Signature::default();
		// 	sig.as_mut().copy_from_slice(&signature);

		// 	let mut pubkey = sr25519::Public::default();

		// 	pubkey.as_mut().copy_from_slice(&sender.encode());

		// 	// runtime_io::sr25519_verify(&sig, msg.as_ref(), &pubkey);

		// 	Ok(())
		// }
	}
}

impl<T: Trait> Module<T> {
	fn new_id() -> u32 {
		<NextFreeId>::mutate(|n| { let r = *n; *n += 1; r })
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
