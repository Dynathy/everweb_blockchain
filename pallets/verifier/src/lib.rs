#![cfg_attr(not(feature = "std"), no_std)]

pub use crate::pallet::*;

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

    /// Tracks registered verifiers.
    #[pallet::storage]
    #[pallet::getter(fn verifiers)]
    pub type Verifiers<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

	/// Tracks assigned submissions to verifiers.
	#[pallet::storage]
	#[pallet::getter(fn assigned_submissions)]
	pub type AssignedSubmissions<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId, // Verifier Account ID
		BoundedVec<T::Hash, T::MaxUrlLength>, // List of assigned submission hashes with max length
		ValueQuery
	>;

    /// Events emitted by the pallet.
    #[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		VerifierRegistered { verifier: T::AccountId, deposit: BalanceOf<T> },
		/// Submission assigned to a verifier.
		SubmissionAssigned { verifier: T::AccountId, hash: T::Hash },
		/// Validation completed by a verifier.
		ValidationCompleted { verifier: T::AccountId, hash: T::Hash, valid: bool },
	}

    /// Errors that can occur in the pallet.
    #[pallet::error]
    pub enum Error<T> {
        VerifierAlreadyRegistered,
        VerifierNotRegistered,
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
        /// Register a new verifier with a deposit.
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn register_verifier(
			origin: OriginFor<T>, 
			deposit: BalanceOf<T>
		) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(!Verifiers::<T>::contains_key(&who), Error::<T>::VerifierAlreadyRegistered);

            T::Currency::reserve(&who, deposit)?;
            Verifiers::<T>::insert(&who, deposit);
			log::info!("About to deposit event for miner registration");
            Self::deposit_event(Event::VerifierRegistered { verifier: who.clone(), deposit });
			log::info!("Event deposited");
            Ok(())
        }

		/// Verifiers validate a submission.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn validate_submission(
			origin: OriginFor<T>,
			hash: T::Hash,
			is_valid: bool, // True if the submission matches the scrape
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			log::info!("Verifier {:?} is attempting to validate submission {:?}", who, hash);

			// Ensure verifier is registered
			ensure!(Verifiers::<T>::contains_key(&who), Error::<T>::VerifierNotRegistered);

			log::info!("Verifier {:?} is registered.", who);

			// Ensure the submission was assigned to this verifier
			let assignments = AssignedSubmissions::<T>::get(&who);
			ensure!(assignments.contains(&hash), Error::<T>::SubmissionNotAssigned);

			log::info!("Submission {:?} was assigned to verifier {:?}.", hash, who);
			
			// Emit event and notify treasury manager
			Self::deposit_event(Event::ValidationCompleted { verifier: who.clone(), hash, valid: is_valid });

			log::info!(
				"Validation completed for submission {:?} by verifier {:?}. Valid: {}",
				hash,
				who,
				is_valid
			);

			Ok(())
		}
	}
	impl<T: Config> Pallet<T> {
		pub fn assign_submission(verifier: T::AccountId, hash: T::Hash) -> DispatchResult {

			log::info!(
				"Assigning submission {:?} to verifier {:?}.",
				hash,
				verifier
			);

			let mut assignments = AssignedSubmissions::<T>::get(&verifier);

			log::info!(
				"Current assignments for verifier {:?}: {:?}",
				verifier,
				assignments
			);

			assignments.try_push(hash).map_err(|_| Error::<T>::UrlTooLong)?;
			log::info!(
				"Submission {:?} added to assignments for verifier {:?}.",
				hash,
				verifier
			);

			AssignedSubmissions::<T>::insert(&verifier, assignments);
			log::info!(
				"Assignments for verifier {:?} updated in storage.",
				verifier
			);
			
			// Emit event for submission assignment
			Self::deposit_event(Event::SubmissionAssigned { verifier: verifier.clone(), hash });
			log::info!(
				"Event emitted: Submission {:?} assigned to verifier {:?}.",
				hash,
				verifier
			);

			Ok(())
		}
	
		pub fn verifiers_iter() -> impl Iterator<Item = (T::AccountId, BalanceOf<T>)> {
			log::info!("Iterating over verifiers.");
			Verifiers::<T>::iter()
		}
	}
}
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;