#[cfg(test)]


mod tests {
    use super::*;
    use crate::mock::{new_test_ext, RuntimeOrigin, RuntimeEvent, System, Treasury};
    use crate::mock::Test;
    use crate::{Event as TreasuryEvent, Error as TreasuryError};
    use frame_support::{assert_noop, assert_ok};
    use sp_runtime::AccountId32;

    use log::debug;

    #[test]
    fn deposit_funds_works() {
        new_test_ext().execute_with(|| {
           
            // Verify initial treasury balance
            assert_eq!(Treasury::treasury_balance(), 0);

            // Deposit funds
            let sender = AccountId32::from([1; 32]);
            let sent_amount = 200;
            assert_ok!(Treasury::deposit_funds(RuntimeOrigin::signed(sender.clone()), sent_amount));

            // Verify updated treasury balance
            assert_eq!(Treasury::treasury_balance(), sent_amount);

            // Assert the event
            System::assert_last_event(RuntimeEvent::Treasury(TreasuryEvent::FundsDeposited {
                who: sender,
                amount: sent_amount,
            }));
        });
    }

    #[test]
    fn transfer_funds_works() {
        new_test_ext().execute_with(|| {
            let sender = AccountId32::from([1; 32]);
            let recipient = AccountId32::from([2; 32]);
            let deposit_amount = 200;
            let transfer_amount = 150;

            // Deposit funds into the treasury first
            assert_ok!(Treasury::deposit_funds(RuntimeOrigin::signed(sender.clone()), deposit_amount));

            // Transfer funds from the treasury to the recipient
            assert_ok!(Treasury::transfer_funds(
                RuntimeOrigin::root(),
                recipient.clone(),
                transfer_amount
            ));

            // Verify that the treasury balance is updated
            assert_eq!(Treasury::treasury_balance(), deposit_amount - transfer_amount);

            // Check that the transfer event was emitted
            System::assert_last_event(RuntimeEvent::Treasury(TreasuryEvent::FundsTransferred {
                recipient,
                amount: transfer_amount,
            }));
        });
    }

    #[test]
    fn transfer_funds_fails_when_insufficient() {
        new_test_ext().execute_with(|| {
            let recipient = AccountId32::from([2; 32]);
            let transfer_amount = 100;

            // Attempt to transfer funds without depositing first
            assert_noop!(
                Treasury::transfer_funds(RuntimeOrigin::root(), recipient, transfer_amount),
                crate::Error::<Test>::InsufficientFunds
            );
        });
    }

    #[test]
    fn distribute_rewards_works() {
        env_logger::init(); // Initialize the logger
        new_test_ext().execute_with(|| {
            let miner = AccountId32::from([1; 32]);
            let validator = AccountId32::from([2; 32]);
            let deposit_amount = 500;
            let miner_reward = 300;
            let validator_reward = 200;

            // Deposit funds into the treasury
            assert_ok!(Treasury::deposit_funds(RuntimeOrigin::signed(miner.clone()), deposit_amount));
            assert_eq!(Treasury::treasury_balance(), deposit_amount); // Verify treasury balance
    
            debug!("Treasury balance before transfer: {:?}", Treasury::treasury_balance());
            debug!("Miner account: {:?}, Validator account: {:?}", miner, validator);

            // Check Math
            assert!(Treasury::treasury_balance() >= (miner_reward + validator_reward));

            // Distribute rewards to miner and validator
            assert_ok!(Treasury::distribute_rewards(
                RuntimeOrigin::root(),
                miner.clone(),
                validator.clone(),
                miner_reward,
                validator_reward
            ));
    
            debug!("Treasury balance after transfer: {:?}", Treasury::treasury_balance());
            // Verify that the treasury balance is updated
            assert_eq!(Treasury::treasury_balance(), 0);
    
            // Check that the reward distribution event was emitted
            System::assert_last_event(RuntimeEvent::Treasury(TreasuryEvent::RewardsDistributed {
                miner,
                validator,
                miner_reward,
                validator_reward,
            }));
        });
    }
    

    #[test]
    fn distribute_rewards_fails_when_insufficient() {
        new_test_ext().execute_with(|| {
            let miner = AccountId32::from([1; 32]);
            let validator = AccountId32::from([2; 32]);
            let miner_reward = 300;
            let validator_reward = 200;

            // Attempt to distribute rewards without enough balance in the treasury
            assert_noop!(
                Treasury::distribute_rewards(
                    RuntimeOrigin::root(),
                    miner,
                    validator,
                    miner_reward,
                    validator_reward
                ),
                crate::Error::<Test>::InsufficientFunds
            );
        });
    }
}
