#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{new_test_ext, RuntimeOrigin, RuntimeEvent, MaxUrlLength, System, Verifier};
    use crate::mock::Test;
    use crate::AssignedSubmissions;
    use crate::Verifiers;
    use crate::Pallet as VerifierPallet;
    use crate::Error;
    use frame_support::{assert_noop, assert_ok, BoundedVec};
    use sp_core::H256;
    use sp_runtime::AccountId32;



    #[test]
    fn register_verifier_works() {
        new_test_ext().execute_with(|| {

            System::deposit_event(RuntimeEvent::Verifier(crate::Event::VerifierRegistered {
                verifier: AccountId32::new([1; 32]),
                deposit: 100,
            }));
            
            println!("System events: {:?}", System::events());

            let verifier_id = AccountId32::new([1; 32]);
            let deposit = 100;

            // Register the verifier
            assert_ok!(Verifier::register_verifier(RuntimeOrigin::signed(verifier_id.clone()), deposit));

            // Check storage
            let stored_deposit = Verifier::verifiers(&verifier_id);
            println!("Stored Deposit: {:?}", stored_deposit);
            assert_eq!(stored_deposit, Some(deposit));

            // Check emitted events
            let events = System::events();
            println!("Events: {:?}", events);

            //Assert that the correct event was emitted
            assert!(events.iter().any(|record| matches!(
               &record.event,
               RuntimeEvent::Verifier(crate::Event::VerifierRegistered { verifier, deposit: dep }) if *verifier == verifier_id && *dep == deposit
            )));
        });
    }

    #[test]
    fn validate_submission_works() {
        new_test_ext().execute_with(|| {
            let verifier_id = AccountId32::new([1; 32]);
            let hash = H256::random();
            let is_valid = true;

            // Add the verifier to the registry
            let deposit = 100;
            Verifiers::<Test>::insert(&verifier_id, deposit);

            // Assign a submission to the verifier
            let bounded_hashes: BoundedVec<_, MaxUrlLength> = vec![hash].try_into().unwrap();
            AssignedSubmissions::<Test>::insert(&verifier_id, bounded_hashes);

            // Validate the submission
            assert_ok!(VerifierPallet::<Test>::validate_submission(
                RuntimeOrigin::signed(verifier_id.clone()),
                hash,
                is_valid
            ));

            // Check emitted events
            let events = System::events();
            assert!(events.iter().any(|record| matches!(
                &record.event,
                RuntimeEvent::Verifier(crate::Event::ValidationCompleted { verifier, hash: submitted_hash, valid })
                    if *verifier == verifier_id && *submitted_hash == hash && *valid == is_valid
            )));
        });
    }

    #[test]
    fn validate_submission_fails_if_verifier_not_registered() {
        new_test_ext().execute_with(|| {
            let verifier_id = AccountId32::new([1; 32]);
            let hash = H256::random();
            let is_valid = true;
    
            // Attempt to validate without registering the verifier
            assert_noop!(
                VerifierPallet::<Test>::validate_submission(
                    RuntimeOrigin::signed(verifier_id),
                    hash,
                    is_valid
                ),
                Error::<Test>::VerifierNotRegistered
            );
        });
    }   

    #[test]
    fn validate_submission_fails_if_submission_not_assigned() {
        new_test_ext().execute_with(|| {
            let verifier_id = AccountId32::new([1; 32]);
            let hash = H256::random();
            let is_valid = true;
    
            // Add the verifier to the registry
            let deposit = 100;
            Verifiers::<Test>::insert(&verifier_id, deposit);
    
            // Attempt to validate a submission that was not assigned
            assert_noop!(
                VerifierPallet::<Test>::validate_submission(
                    RuntimeOrigin::signed(verifier_id),
                    hash,
                    is_valid
                ),
                Error::<Test>::SubmissionNotAssigned
            );
        });
    }

    #[test]
    fn register_verifier_fails_if_already_registered() {
        new_test_ext().execute_with(|| {
            let verifier_id = AccountId32::new([1; 32]);
            let deposit = 100;
    
            // Register the verifier
            assert_ok!(Verifier::register_verifier(RuntimeOrigin::signed(verifier_id.clone()), deposit));
    
            // Attempt to register the same verifier again
            assert_noop!(
                Verifier::register_verifier(RuntimeOrigin::signed(verifier_id.clone()), deposit),
                Error::<Test>::VerifierAlreadyRegistered
            );
        });
    }
}
