#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{new_test_ext, RuntimeOrigin, RuntimeEvent, System, VerifierSubmissionManager,};
    use crate::mock::Test;
    use crate::mock::Balances;
    use crate::Error;
    use crate::ProcessedSubmissions;
    use crate::mock::TreasuryManagerPalletId;
    use crate::mock::TreasuryPalletId;
    use frame_support::{assert_noop, assert_ok, assert_err, BoundedVec,};
    use frame_support::traits::Hooks;
    use frame_support::traits::OnFinalize;
    use frame_system::pallet_prelude::BlockNumberFor;
    use sp_core::H256;
    use sp_runtime::AccountId32;
    use std::sync::Once;
    use frame_support::traits::fungible::Mutate;
    use sp_runtime::traits::AccountIdConversion;
    use frame_support::traits::Currency;

    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            env_logger::builder()
                .is_test(true) // Ensure logs integrate nicely with test output
                .init();
        });
    }

    #[test]
    fn test_valid_submission() {
        init_logger(); // Initialize the logger
        new_test_ext().execute_with(|| {
            let miner = AccountId32::new([1; 32]);
            let verifier = AccountId32::new([2; 32]);
            let submission_hash = H256::from_low_u64_be(1);

            log::info!("Starting test for valid submission...");
            log::info!("Miner: {:?}, Verifier: {:?}, Submission Hash: {:?}", miner, verifier, submission_hash);

            // Set balances
            let treasury_account = TreasuryManagerPalletId::get().into_account_truncating();
            log::info!(
                "VerifierSubmissionManager Treasury account: {:?}",
                treasury_account
            );
            // Update the balance
            pallet_balances::Pallet::<Test>::set_balance(&treasury_account, 1_000u128);

            // Verifier submits a valid submission
            assert_ok!(VerifierSubmissionManager::submit_verification(
                RuntimeOrigin::signed(verifier.clone()),
                miner.clone(),
                submission_hash,
                true
            ));

            log::info!("Submission by {:?} for {:?} validated successfully", verifier, miner);
            // Ensure the submission is recorded
            let submissions = VerifierSubmissionManager::verifier_submissions(submission_hash);
            assert_eq!(submissions.len(), 1);
            assert_eq!(submissions[0], (verifier.clone(), true));

            // Ensure the deadline is set
            let deadline = VerifierSubmissionManager::verification_deadline(submission_hash);
            assert!(deadline.is_some());
            log::info!("Test for valid submission completed successfully.");
        });
    }

    #[test]
    fn test_exceed_max_verifier_submissions() {
        new_test_ext().execute_with(|| {
            let miner = AccountId32::new([1; 32]);
            let submission_hash = H256::from_low_u64_be(1);

            // Set balances
            let treasury_account = TreasuryManagerPalletId::get().into_account_truncating();
            pallet_balances::Pallet::<Test>::set_balance(&treasury_account, 1_000u128);

            // Submit the maximum number of submissions
            for i in 0..10 {
                let verifier = AccountId32::new([i as u8; 32]);
                assert_ok!(VerifierSubmissionManager::submit_verification(
                    RuntimeOrigin::signed(verifier.clone()),
                    miner.clone(),
                    submission_hash,
                    true
                ));
            }

            // Attempt to submit beyond the limit
            let extra_verifier = AccountId32::new([99; 32]);
            assert_err!(
                VerifierSubmissionManager::submit_verification(
                    RuntimeOrigin::signed(extra_verifier.clone()),
                    miner.clone(),
                    submission_hash,
                    true
                ),
                sp_runtime::DispatchError::Other("VerifierSubmissions is full")
            );

            // Ensure no additional entries are added
            let submissions = VerifierSubmissionManager::verifier_submissions(submission_hash);
            assert_eq!(
                submissions.len(),
                10,
                "VerifierSubmissions should contain exactly the maximum allowed entries"
            );
        });
    }

    #[test]
    fn test_submission_after_deadline() {
        new_test_ext().execute_with(|| {
            let miner = AccountId32::new([1; 32]);
            let verifier = AccountId32::new([2; 32]);
            let submission_hash = H256::from_low_u64_be(1);

            // Set balances
            let treasury_account = TreasuryManagerPalletId::get().into_account_truncating();
            pallet_balances::Pallet::<Test>::set_balance(&treasury_account, 1_000u128);

            // Submit a verification
            assert_ok!(VerifierSubmissionManager::submit_verification(
                RuntimeOrigin::signed(verifier.clone()),
                miner.clone(),
                submission_hash,
                true
            ));

            // Move past the deadline
            System::set_block_number(10);

            // Attempt another submission after the deadline
            let late_verifier = AccountId32::new([3; 32]);
            assert_err!(
                VerifierSubmissionManager::submit_verification(
                    RuntimeOrigin::signed(late_verifier.clone()),
                    miner.clone(),
                    submission_hash,
                    true
                ),
                crate::Error::<Test>::VerificationExpired
            );
        });
    }


    #[test]
    fn test_process_submissions_via_public_api() {
        init_logger(); // Ensure logger is initialized
        new_test_ext().execute_with(|| {
            let miner = AccountId32::new([1; 32]);
            let verifier1 = AccountId32::new([2; 32]);
            let verifier2 = AccountId32::new([3; 32]);
            let verifier3 = AccountId32::new([4; 32]);
            let submission_hash = H256::random();

            // Set balances
            let treasury_account = TreasuryPalletId::get().into_account_truncating();
            log::info!("Treasury account: {:?}", treasury_account);
            //Balances::make_free_balance_be(&treasury_account, 1_000_000u128);

            let balance = Balances::free_balance(&treasury_account);
            log::info!("Treasury balance after setup: {:?}", balance);

            // Set the initial block number
            System::set_block_number(1);

            // Submit verifications
            assert_ok!(VerifierSubmissionManager::submit_verification(
                RuntimeOrigin::signed(verifier1.clone()),
                miner.clone(),
                submission_hash,
                true,
            ));
            assert_ok!(VerifierSubmissionManager::submit_verification(
                RuntimeOrigin::signed(verifier2.clone()),
                miner.clone(),
                submission_hash,
                true,
            ));

            //Advanced the block and add another verifier
            System::set_block_number(2);
            assert_ok!(VerifierSubmissionManager::submit_verification(
                RuntimeOrigin::signed(verifier3.clone()),
                miner.clone(),
                submission_hash,
                true,
            ));

            log::info!(
                "Submissions before finalizing: {:?}",
                VerifierSubmissionManager::verifier_submissions(submission_hash)
            );

            // Call `on_finalize` before the deadline
            System::set_block_number(5);
            <crate::Pallet<Test> as frame_support::traits::Hooks<BlockNumberFor<Test>>>::on_finalize(5);

            // Ensure submission is marked as processed
            assert!(
                ProcessedSubmissions::<Test>::contains_key(submission_hash),
                "Submission should be marked as processed"
            );

            // Ensure the correct event is emitted
            System::assert_last_event(RuntimeEvent::VerifierSubmissionManager(
                crate::Event::SubmissionValidated { miner, hash: submission_hash, valid: true },
            ));

            // Move past the deadline
            System::set_block_number(11);
            <crate::Pallet<Test> as frame_support::traits::Hooks<BlockNumberFor<Test>>>::on_finalize(11);

            // Check that no further events are emitted after expiration
            assert!(
                ProcessedSubmissions::<Test>::contains_key(submission_hash),
                "Submission should remain processed"
            );
        });
    }

    #[test]
    fn test_2_3_threshold_submission() {
        new_test_ext().execute_with(|| {
            let miner = AccountId32::new([1; 32]);
            let verifier1 = AccountId32::new([2; 32]);
            let verifier2 = AccountId32::new([3; 32]);
            let verifier3 = AccountId32::new([4; 32]);
            let verifier4 = AccountId32::new([5; 32]);
            let verifier5 = AccountId32::new([6; 32]);
            let submission_hash = H256::random();

            // Set balances
            let treasury_account = TreasuryManagerPalletId::get().into_account_truncating();
            pallet_balances::Pallet::<Test>::set_balance(&treasury_account, 1_000u128);

            // Set initial block
            System::set_block_number(1);

            // Verifiers submit their verifications
            assert_ok!(VerifierSubmissionManager::submit_verification(
                RuntimeOrigin::signed(verifier1.clone()),
                miner.clone(),
                submission_hash,
                true,
            ));
            assert_ok!(VerifierSubmissionManager::submit_verification(
                RuntimeOrigin::signed(verifier2.clone()),
                miner.clone(),
                submission_hash,
                true,
            ));
            assert_ok!(VerifierSubmissionManager::submit_verification(
                RuntimeOrigin::signed(verifier3.clone()),
                miner.clone(),
                submission_hash,
                false,
            ));
            assert_ok!(VerifierSubmissionManager::submit_verification(
                RuntimeOrigin::signed(verifier4.clone()),
                miner.clone(),
                submission_hash,
                true,
            ));

            assert_ok!(VerifierSubmissionManager::submit_verification(
                RuntimeOrigin::signed(verifier5.clone()),
                miner.clone(),
                submission_hash,
                true,
            ));

            // Process submissions
            System::set_block_number(5);
            <VerifierSubmissionManager as frame_support::traits::Hooks<BlockNumberFor<Test>>>::on_finalize(5);

            // Check processed status
            assert!(ProcessedSubmissions::<Test>::contains_key(submission_hash));

            // Check the last event
            System::assert_last_event(RuntimeEvent::VerifierSubmissionManager(
                crate::Event::SubmissionValidated {
                    miner,
                    hash: submission_hash,
                    valid: true,
                },
            ));
        });
    }

    #[test]
    fn test_2_3_threshold_submission_failure() {
        new_test_ext().execute_with(|| {
            let miner = AccountId32::new([1; 32]);
            let verifier1 = AccountId32::new([2; 32]);
            let verifier2 = AccountId32::new([3; 32]);
            let verifier3 = AccountId32::new([4; 32]);
            let verifier4 = AccountId32::new([5; 32]);
            let verifier5 = AccountId32::new([6; 32]);
            let submission_hash = H256::random();

            // Set balances
            let treasury_account = TreasuryManagerPalletId::get().into_account_truncating();
            pallet_balances::Pallet::<Test>::set_balance(&treasury_account, 1_000u128);

            // Set initial block
            System::set_block_number(1);

            // Verifiers submit their verifications
            assert_ok!(VerifierSubmissionManager::submit_verification(
                RuntimeOrigin::signed(verifier1.clone()),
                miner.clone(),
                submission_hash,
                false,
            ));
            assert_ok!(VerifierSubmissionManager::submit_verification(
                RuntimeOrigin::signed(verifier2.clone()),
                miner.clone(),
                submission_hash,
                false,
            ));
            assert_ok!(VerifierSubmissionManager::submit_verification(
                RuntimeOrigin::signed(verifier3.clone()),
                miner.clone(),
                submission_hash,
                false,
            ));
            assert_ok!(VerifierSubmissionManager::submit_verification(
                RuntimeOrigin::signed(verifier4.clone()),
                miner.clone(),
                submission_hash,
                false,
            ));

            assert_ok!(VerifierSubmissionManager::submit_verification(
                RuntimeOrigin::signed(verifier5.clone()),
                miner.clone(),
                submission_hash,
                false,
            ));

            // Process submissions
            System::set_block_number(5);
            <VerifierSubmissionManager as frame_support::traits::Hooks<BlockNumberFor<Test>>>::on_finalize(5);

            // Check processed status
            assert!(ProcessedSubmissions::<Test>::contains_key(submission_hash));

            // Check the last event
            System::assert_last_event(RuntimeEvent::VerifierSubmissionManager(
                crate::Event::SubmissionValidated {
                    miner,
                    hash: submission_hash,
                    valid: false,
                },
            ));
        });
    }
}