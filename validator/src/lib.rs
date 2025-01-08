#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
		BoundedVec,
		PalletId,
        traits::{Currency, ExistenceRequirement, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
	use sp_runtime::traits::AccountIdConversion;

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);


	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Currency: ReservableCurrency<Self::AccountId>;
		type SubmissionFee: Get<BalanceOf<Self>>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
    	type PalletId: Get<PalletId>;
		#[pallet::constant]
    	type MaxUrlLength: Get<u32>; // Maximum length for URLs
	}

    /// Tracks registered validators.
    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

	/// Tracks assigned submissions to validators.
	#[pallet::storage]
	#[pallet::getter(fn assigned_submissions)]
	pub type AssignedSubmissions<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId, // Validator Account ID
		BoundedVec<T::Hash, T::MaxUrlLength>, // List of assigned submission hashes with max length
		ValueQuery
	>;

    /// Events emitted by the pallet.
    #[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ValidatorRegistered { validator: T::AccountId, deposit: BalanceOf<T> },
		/// Submission assigned to a validator.
		SubmissionAssigned { validator: T::AccountId, hash: T::Hash },
		/// Validation completed by a validator.
		ValidationCompleted { validator: T::AccountId, hash: T::Hash, valid: bool },
	}

    /// Errors that can occur in the pallet.
    #[pallet::error]
    pub enum Error<T> {
        ValidatorAlreadyRegistered,
        ValidatorNotRegistered,
        NotWhitelisted,
		SubmissionNotAssigned,
        InsufficientFunds,
		UrlTooLong, // Too Long error

    }

	impl<T: Config> Pallet<T> {
		/// Returns the account ID for the pallet
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}

    /// Pallet calls.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new validator with a deposit.
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn register_validator(origin: OriginFor<T>, deposit: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(!Validators::<T>::contains_key(&who), Error::<T>::ValidatorAlreadyRegistered);

            T::Currency::reserve(&who, deposit)?;
            Validators::<T>::insert(&who, deposit);
			log::info!("About to deposit event for miner registration");
            Self::deposit_event(Event::ValidatorRegistered { validator: who.clone(), deposit });
			log::info!("Event deposited");
            Ok(())
        }

		/// Validators validate a submission.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn validate_submission(
			origin: OriginFor<T>,
			hash: T::Hash,
			is_valid: bool, // True if the submission matches the scrape
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure validator is registered
			ensure!(Validators::<T>::contains_key(&who), Error::<T>::ValidatorNotRegistered);

			// Ensure the submission was assigned to this validator
			let assignments = AssignedSubmissions::<T>::get(&who);
			ensure!(assignments.contains(&hash), Error::<T>::SubmissionNotAssigned);

			// Emit event and notify treasury manager
			Self::deposit_event(Event::ValidationCompleted { validator: who.clone(), hash, valid: is_valid });
			Ok(())
		}
    }
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;