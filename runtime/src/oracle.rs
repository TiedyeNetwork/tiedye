use support::{decl_module, decl_event, dispatch::Result};

pub trait Trait: system::Trait {
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        pub fn do_something(origin) -> Result {
            Ok(())
        }
    }
}

decl_event!(
    pub enum Event {
        Hi(),
    }
);
