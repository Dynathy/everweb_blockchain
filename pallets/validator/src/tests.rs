#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{new_test_ext, RuntimeOrigin, RuntimeEvent, MaxUrlLength, System, Validator};
    use crate::mock::Test;
    use crate::AssignedSubmissions;
    use crate::Validators;
    use crate::Pallet as ValidatorPallet;
    use crate::Error;
    use frame_support::{assert_noop, assert_ok, BoundedVec};
    use sp_core::H256;
    use sp_runtime::AccountId32;



    #[test]
    fn register_validator_works() {
        new_test_ext().execute_with(|| {

            System::deposit_event(RuntimeEvent::Validator(crate::Event::ValidatorRegistered {
                validator: AccountId32::new([1; 32]),
                deposit: 100,
            }));
            
            println!("System events: {:?}", System::events());

            let validator_id = AccountId32::new([1; 32]);
            let deposit = 100;

            // Register the validator
            assert_ok!(Validator::register_validator(RuntimeOrigin::signed(validator_id.clone()), deposit));

            // Check storage
            let stored_deposit = Validator::validators(&validator_id);
            println!("Stored Deposit: {:?}", stored_deposit);
            assert_eq!(stored_deposit, Some(deposit));

            // Check emitted events
            let events = System::events();
            println!("Events: {:?}", events);

            //Assert that the correct event was emitted
            assert!(events.iter().any(|record| matches!(
               &record.event,
               RuntimeEvent::Validator(crate::Event::ValidatorRegistered { validator, deposit: dep }) if *validator == validator_id && *dep == deposit
            )));
        });
    }

    #[test]
    fn validate_submission_works() {
        new_test_ext().execute_with(|| {
            let validator_id = AccountId32::new([1; 32]);
            let hash = H256::random();
            let is_valid = true;

            // Add the validator to the registry
            let deposit = 100;
            Validators::<Test>::insert(&validator_id, deposit);

            // Assign a submission to the validator
            let bounded_hashes: BoundedVec<_, MaxUrlLength> = vec![hash].try_into().unwrap();
            AssignedSubmissions::<Test>::insert(&validator_id, bounded_hashes);

            // Validate the submission
            assert_ok!(ValidatorPallet::<Test>::validate_submission(
                RuntimeOrigin::signed(validator_id.clone()),
                hash,
                is_valid
            ));

            // Check emitted events
            let events = System::events();
            assert!(events.iter().any(|record| matches!(
                &record.event,
                RuntimeEvent::Validator(crate::Event::ValidationCompleted { validator, hash: submitted_hash, valid })
                    if *validator == validator_id && *submitted_hash == hash && *valid == is_valid
            )));
        });
    }

    #[test]
    fn validate_submission_fails_if_validator_not_registered() {
        new_test_ext().execute_with(|| {
            let validator_id = AccountId32::new([1; 32]);
            let hash = H256::random();
            let is_valid = true;
    
            // Attempt to validate without registering the validator
            assert_noop!(
                ValidatorPallet::<Test>::validate_submission(
                    RuntimeOrigin::signed(validator_id),
                    hash,
                    is_valid
                ),
                Error::<Test>::ValidatorNotRegistered
            );
        });
    }   

    #[test]
    fn validate_submission_fails_if_submission_not_assigned() {
        new_test_ext().execute_with(|| {
            let validator_id = AccountId32::new([1; 32]);
            let hash = H256::random();
            let is_valid = true;
    
            // Add the validator to the registry
            let deposit = 100;
            Validators::<Test>::insert(&validator_id, deposit);
    
            // Attempt to validate a submission that was not assigned
            assert_noop!(
                ValidatorPallet::<Test>::validate_submission(
                    RuntimeOrigin::signed(validator_id),
                    hash,
                    is_valid
                ),
                Error::<Test>::SubmissionNotAssigned
            );
        });
    }

    #[test]
    fn register_validator_fails_if_already_registered() {
        new_test_ext().execute_with(|| {
            let validator_id = AccountId32::new([1; 32]);
            let deposit = 100;
    
            // Register the validator
            assert_ok!(Validator::register_validator(RuntimeOrigin::signed(validator_id.clone()), deposit));
    
            // Attempt to register the same validator again
            assert_noop!(
                Validator::register_validator(RuntimeOrigin::signed(validator_id.clone()), deposit),
                Error::<Test>::ValidatorAlreadyRegistered
            );
        });
    }
}
