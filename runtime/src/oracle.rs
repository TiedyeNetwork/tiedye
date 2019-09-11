//! Oracle Module
//! 
//! The Oracle module allows for the querying of offchain price feeds and recording thereof.
//! 
//! ## Interface
//! 
//! ### Public Functions
//! 
//! - `register_new_oracle` - Registers a new endpoint for which the system will begin to query. Requires a bond.

use rstd::convert::{TryInto};
use support::{decl_module, decl_event, decl_storage, StorageMap, dispatch::Result};

pub trait Trait: system::Trait {
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as OracleStorage {
        Oracles get(oracles): map u32 => &str;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        pub fn do_something(origin) -> Result {
            Ok(())
        }

        // Runs after every block.
        fn offchain_worker(now: T::BlockNumber) {
            runtime_io::print("Hello from the offchain worker!");
            let request_id = runtime_io::http_request_start("GET", "http://localhost:7666/mock", &[]).unwrap();
            runtime_io::http_request_write_body(request_id, &[], None).unwrap_or(());
            runtime_io::http_response_wait(&[request_id], None);
            let buffer: &mut [u8] = &mut [0; 8];
            let size = runtime_io::http_response_read_body(request_id, buffer, None).unwrap_or(42);
            let result = &buffer[..];
            runtime_io::print(result);
        }
    }
}
decl_event!(
    pub enum Event {
        Hi(),
    }
);
