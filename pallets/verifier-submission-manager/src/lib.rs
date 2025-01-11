#![cfg_attr(not(feature = "std"), no_std)]

pub use crate::pallet::*;

#[frame_support::pallet]
pub mod pallet {

    use codec::FullCodec;
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{Currency, Get},
    };
    use frame_system::pallet_prelude::*;
    use sp_std::{fmt::Debug, vec::Vec};
    use pallet_treasury_manager;
    use pallet_treasury_manager::{Config as TreasuryConfig, Event as TreasuryEvent};

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_treasury_manager::Config {
        // Add the treasury manager pallet dependency.
        type TreasuryManager: TreasuryConfig;
        type Currency: Currency<Self::AccountId>;
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        // Add TotalReward
        type TotalReward: Get<BalanceOf<Self>>;
        #[pallet::constant]
        type MaxVerifierSubmissions: Get<u32>;

        #[pallet::constant]
        type VerificationTimeout: Get<BlockNumberFor<Self>>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::storage]
    #[pallet::getter(fn verifier_submissions)]
    pub type VerifierSubmissions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        BoundedVec<(T::AccountId, bool), T::MaxVerifierSubmissions>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn verification_deadline)]
    pub type VerificationDeadline<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        BlockNumberFor<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn processed_submissions)]
    pub type ProcessedSubmissions<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn miner_for_hash)]
    pub type MinerForHash<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        T::AccountId,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SubmissionValidated {
            miner: T::AccountId,
            hash: T::Hash,
            valid: bool,
        },
        SubmissionExpired {
            hash: T::Hash,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        SubmissionAlreadyProcessed,
        VerificationExpired,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(n: BlockNumberFor<T>) {
            for (hash, deadline) in VerificationDeadline::<T>::iter() {
                log::debug!(
                    "on_finalize: Current Block: {:?}, Hash: {:?}, Deadline: {:?}",
                    n,
                    hash,
                    deadline
                );

                // Skip submissions that have already been processed
                if ProcessedSubmissions::<T>::contains_key(&hash) {
                    log::info!("Skipping already processed submission: {:?}", hash);
                    continue;
                }

                // Retrieve submissions for the given hash
                let submissions = VerifierSubmissions::<T>::get(&hash);

                // If the submission has sufficient entries and is before the deadline, process it
                if submissions.len() > 0 && n <= deadline {
                    log::info!("Processing valid submissions for hash: {:?}", hash);

                    if let Some(miner) = MinerForHash::<T>::get(&hash) {
                        Self::process_submissions(miner.clone(), hash, submissions.to_vec());
                        ProcessedSubmissions::<T>::insert(&hash, ());
                        log::info!("Submission marked as processed: {:?}", hash);
                    } else {
                        log::warn!("No miner found for hash: {:?}", hash);
                    }
                    continue; // Skip further checks for this submission
                }

                // Handle expiration if the deadline has passed and the submission is unprocessed
                if n > deadline {
                    log::info!("Submission expired: {:?}", hash);
                    Self::handle_expired_submission(hash);
                    ProcessedSubmissions::<T>::insert(&hash, ());
                    log::info!("Submission marked as expired: {:?}", hash);
                } else {
                    log::info!("Submission still valid and awaiting further verifications: {:?}", hash);
                }
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn submit_verification(
            origin: OriginFor<T>,
            miner: T::AccountId,
            hash: T::Hash,
            is_valid: bool,
        ) -> DispatchResult {
            let verifier = ensure_signed(origin)?;
            
            log::info!(
                "submit_verification: Miner: {:?}, Verifier: {:?}, Hash: {:?}, Valid: {:?}",
                miner, verifier, hash, is_valid
            );
            ///Ensure that when a verification is submitted the miner is stored
            if MinerForHash::<T>::get(&hash).is_none() {
                MinerForHash::<T>::insert(&hash, &miner);
                log::info!("Miner for hash {:?} set to {:?}", hash, miner);
            }

            VerifierSubmissions::<T>::try_mutate(&hash, |submissions| -> Result<(), DispatchError> {
                log::info!("Verifier submissions before mutation: {:?}", submissions);
                if let Err(_) = submissions.try_push((verifier.clone(), is_valid)) {
                    // Handle case where the BoundedVec is full
                    log::info!("VerifierSubmissions is full for hash {:?}", hash);
                    return Err(DispatchError::Other("VerifierSubmissions is full"));
                }
                log::info!("Verifier submissions after mutation: {:?}", submissions);
                Ok(())
            })?;

            ensure!(
                !ProcessedSubmissions::<T>::contains_key(&hash),
                Error::<T>::SubmissionAlreadyProcessed
            );
        
            let current_block = <frame_system::Pallet<T>>::block_number();
            if let Some(deadline) = VerificationDeadline::<T>::get(&hash) {
                log::info!("Submission deadline for hash {:?}: {:?}", hash, deadline);
                ensure!(current_block <= deadline, Error::<T>::VerificationExpired);
            } else {
                VerificationDeadline::<T>::insert(&hash, current_block + T::VerificationTimeout::get());
                log::info!(
                    "Set new verification deadline for hash {:?}: {:?}",
                    hash,
                    current_block + T::VerificationTimeout::get()
                );
            }
        
            // Fetch the updated submissions for further processing
            let submissions = VerifierSubmissions::<T>::get(&hash);
            if submissions.len() >= T::MaxVerifierSubmissions::get() as usize {
                Self::process_submissions(miner, hash, submissions.to_vec());
                ProcessedSubmissions::<T>::insert(&hash, ());
            }
        
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn process_submissions(
            miner: T::AccountId,
            hash: T::Hash,
            submissions: Vec<(T::AccountId, bool)>,
        ) {
            log::info!(
                "process_submissions: Miner: {:?}, Hash: {:?}, Submissions: {:?}",
                miner,
                hash,
                submissions
            );
            let valid_count = submissions.iter().filter(|(_, valid)| *valid).count();
            let total_count = submissions.len();
            log::info!(
                "Valid count: {}, Total count: {}, Threshold: {}",
                valid_count,
                total_count,
                (2.0 / 3.0) * total_count as f32
            );

            // Ensure a minimum of 3 verifiers
            if total_count < 3 {
                log::info!(
                    "Insufficient verifiers for hash {:?}: Only {} verifiers present.",
                    hash,
                    total_count
                );
                return; // Do nothing and wait for more submissions
            }

            // Calculate 2/3 threshold
            if valid_count as f32 >= (2.0 / 3.0) * total_count as f32 {
                log::info!(
                    "2/3 threshold met: {} out of {} verifiers signaled valid",
                    valid_count,
                    total_count
                );
                Self::handle_valid_submission(hash, miner, submissions);
            } else {
                log::info!(
                    "2/3 threshold not met: {} out of {} verifiers signaled invalid",
                    total_count - valid_count,
                    total_count
                );
                Self::handle_invalid_submission(hash, miner);
            }
    
            // Mark submission as processed
            log::info!("Marking submission as processed: {:?}", hash);
            ProcessedSubmissions::<T>::insert(hash, ());
        }
    

        fn handle_valid_submission(hash: T::Hash, miner: T::AccountId, submissions: Vec<(T::AccountId, bool)>) {
            // Extract verifier accounts
            let verifiers: Vec<T::AccountId> = submissions.into_iter().map(|(verifier, _)| verifier).collect();

            // Define total reward (this can be parameterized or dynamic)
            let total_reward = <T as pallet_treasury_manager::Config>::TotalReward::get();

            log::info!(
                "handle_valid_submission: Miner: {:?}, Hash: {:?}, Verifiers: {:?}, Total Reward: {:?}",
                miner,
                hash,
                verifiers,
                total_reward
            );

             // Call TreasuryManager's reward distribution function
            let call_result = pallet_treasury_manager::Pallet::<T>::direct_reward_distribution(
                frame_system::RawOrigin::Root.into(),
                miner.clone(),
                verifiers.clone(),
                total_reward,
            );
            if let Err(e) = call_result {
                log::error!("Failed to distribute rewards: {:?}", e);
            } else {
                log::info!("Rewards distributed successfully for hash: {:?}", hash);
            }
        
            // Emit event
            Self::deposit_event(Event::SubmissionValidated {
                miner,
                hash,
                valid: true,
            });          
        }

        fn handle_invalid_submission(hash: T::Hash, miner: T::AccountId) {
            Self::deposit_event(Event::SubmissionValidated {
                miner,
                hash,
                valid: false,
            });
        }

        fn handle_expired_submission(hash: T::Hash) {
            log::info!("Inserting hash into ProcessedSubmissions: {:?}", hash);
            VerificationDeadline::<T>::remove(&hash);
            MinerForHash::<T>::remove(&hash); // Clean up miner entry
            ProcessedSubmissions::<T>::insert(&hash, ());
            log::info!("Hash inserted into ProcessedSubmissions: {:?}", hash);
            Self::deposit_event(Event::SubmissionExpired { hash });
        }
    }
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;