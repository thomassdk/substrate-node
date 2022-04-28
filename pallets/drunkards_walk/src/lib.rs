#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use codec::{Decode, Encode, MaxEncodedLen};
	use frame_support::{pallet_prelude::*, traits::Randomness};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		#[pallet::constant] // put the constant in metadata
		/// For constraining the limits of evolution
		type EvolutionaryCeiling: Get<u32>;

		// type MyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[derive(
		Clone, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd, TypeInfo, MaxEncodedLen,
	)]
	pub enum Actions {
		Increment,
		Decrement,
		Idle,
	}

	impl Actions {
		fn from_u8(value: u8) -> Actions {
			match value {
				0 => Actions::Increment,
				1 => Actions::Decrement,
				2 => Actions::Idle,
				_ => panic!("Unknown value: {}", value),
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// parameters. [action]
		Changed(Actions),

		/// parameters. [action, count]
		Executed(Actions, u32),

		/// parameters. [action, count]
		ChanceMutation(Actions, u32),
	}

	#[pallet::error]
	pub enum Error<T> {
		Overflow,
		Underflow,
		NotAValidAction,
	}

	#[pallet::storage]
	#[pallet::getter(fn counter)]
	pub type Counter<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn action)]
	pub type Action<T> = StorageValue<_, u8, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn chance)]
	pub type Chance<T> = StorageValue<_, bool, ValueQuery>;

	// NOTE: For randomness
	// #[pallet::storage]
	// pub type Nonce<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn change(origin: OriginFor<T>, action: u8) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let _ = ensure_signed(origin)?;

			Chance::<T>::put(false);

			if action > 2 {
				return Err(Error::<T>::NotAValidAction)?
			}

			Action::<T>::put(action);

			let action = Actions::from_u8(action);

			// Emit an event.
			Self::deposit_event(Event::Changed(action));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn execute(origin: OriginFor<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let _ = ensure_signed(origin)?;

			Chance::<T>::put(false);

			let action = Action::<T>::get();
			let action = Actions::from_u8(action);

			let mut count = Counter::<T>::get();

			log::info!("{:?} {:?}", action, count);

			match action {
				Actions::Increment => {
					let new_count = count.checked_add(1).ok_or(Error::<T>::Overflow)?;
					ensure!(
						new_count <= T::EvolutionaryCeiling::get(),
						"value must be <= evolutionary ceiling"
					);
					Counter::<T>::put(count);
				},
				Actions::Decrement => {
					count = count.checked_sub(1).ok_or(Error::<T>::Underflow)?;
					Counter::<T>::put(count);
				},
				Actions::Idle => (),
			}

			// Emit an event.
			Self::deposit_event(Event::Executed(action, count));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_n: T::BlockNumber) {
			return if Chance::<T>::get() {
				// TODO: Get random working, for now always increment
				let random_num = 0;
				Action::<T>::put(random_num);
				let action = Actions::from_u8(random_num);
				let mut count = Counter::<T>::get();

				match action {
					Actions::Increment =>
						if let Some(v) = count.checked_add(1) {
							if v < T::EvolutionaryCeiling::get() {
								count = v
							}
						},
					Actions::Decrement => {
						if let Some(v) = count.checked_sub(1) {
							count = v;
						}
					},
					Actions::Idle => (),
				}

				Counter::<T>::put(count);

				Self::deposit_event(Event::ChanceMutation(action, count));
			}
		}

		fn on_initialize(_n: T::BlockNumber) -> u64 {
			Chance::<T>::put(true);
			0
		}
	}
}
