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
	use pallet_miner_submission_manager::Pallet as SubmissionManager;
	use pallet_whitelist::Pallet as Whitelist;
    

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);


	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_miner_submission_manager::Config + pallet_whitelist::Config  {
		type Currency: ReservableCurrency<Self::AccountId>;
		type SubmissionFee: Get<BalanceOf<Self>>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
    	type PalletId: Get<PalletId>;
		#[pallet::constant]
    	type MaxUrlLength: Get<u32>; // Maximum length for URLs
	}

    /// Tracks registered miners and their deposits.
    #[pallet::storage]
    #[pallet::getter(fn miners)]
    pub type Miners<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

    /// Events emitted by the pallet.
    #[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MinerRegistered { miner: T::AccountId, deposit: BalanceOf<T> },
		SubmissionForwarded { miner: T::AccountId, url: Vec<u8>, hash: T::Hash },
	}

    /// Errors that can occur in the pallet.
    #[pallet::error]
    pub enum Error<T> {
        MinerAlreadyRegistered,
        MinerNotRegistered,
        NotWhitelisted,
        InsufficientFunds,
		UrlTooLong, // Too Long error
    }

	impl<T: Config> Pallet<T> {
		/// Returns the account ID for the pallet
		pub fn account_id() -> T::AccountId {
			<T as self::Config>::PalletId::get().into_account_truncating()
		}
	}

    /// Pallet calls.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new miner with a deposit.
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn register_miner(origin: OriginFor<T>, deposit: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(!Miners::<T>::contains_key(&who), Error::<T>::MinerAlreadyRegistered);

            <T as self::Config>::Currency::reserve(&who, deposit)?;
            Miners::<T>::insert(&who, deposit);
			log::info!("About to deposit event for miner registration");
            Self::deposit_event(Event::MinerRegistered { miner: who.clone(), deposit });
			log::info!("Event deposited");
            Ok(())
        }

        /// Submit a hash for validation.
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
		pub fn submit_hash(origin: OriginFor<T>, url: Vec<u8>, hash: T::Hash) -> DispatchResult {
            let miner = ensure_signed(origin)?;

            let is_whitelisted = Whitelist::<T>::is_whitelisted(url.clone())?;
            
            ensure!(is_whitelisted, Error::<T>::NotWhitelisted);
        
            log::info!("URL is whitelisted, proceeding with submission");
        
            SubmissionManager::<T>::submit_hash(
                frame_system::RawOrigin::Signed(miner.clone()).into(),
                url.clone(),
                hash,
            )?;
        
            Self::deposit_event(Event::SubmissionForwarded { miner, url, hash });
            Ok(())
        }        
    }
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;