#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::assert_ok;
    use crate::mock::new_test_ext;
    use crate::mock::TreasuryManager;
    use crate::mock::TreasuryManagerPalletId;
    use crate::mock::MinerRewardPercentage;
    use crate::mock::VerifierRewardPercentage;
    use crate::mock::FeeSplitTreasury;
    use crate::mock::DefaultDevAccount;

    use crate::mock::Balances;

    use sp_runtime::AccountId32;
    use sp_runtime::traits::AccountIdConversion;

    #[test]
    fn test_direct_reward_distribution() {
        new_test_ext().execute_with(|| {
            let _ = env_logger::builder().is_test(true).try_init();
            // Initialize accounts
            let treasury_account = TreasuryManagerPalletId::get().into_account_truncating();
            let dev_account = DefaultDevAccount::get();
            let miner_account = AccountId32::new([5; 32]);
            let verifier_1 = AccountId32::new([3; 32]);
            let verifier_2 = AccountId32::new([4; 32]);

            log::info!("Initial accounts setup:");
            log::info!("Treasury Account: {:?}", treasury_account);
            log::info!("Developer Account: {:?}", dev_account);
            log::info!("Miner Account: {:?}", miner_account);
            log::info!("Verifier 1: {:?}", verifier_1);
            log::info!("Verifier 2: {:?}", verifier_2);

            // Define initial balances
            let initial_treasury_balance = Balances::free_balance(&treasury_account);
            let initial_dev_balance = Balances::free_balance(&dev_account);
            let initial_miner_balance = Balances::free_balance(&miner_account);
            let initial_verifier_1_balance = Balances::free_balance(&verifier_1);
            let initial_verifier_2_balance = Balances::free_balance(&verifier_2);

            log::info!(
                "Initial Balances: Treasury: {}, Dev: {}, Miner: {}, Verifier 1: {}, Verifier 2: {}",
                initial_treasury_balance,
                initial_dev_balance,
                initial_miner_balance,
                initial_verifier_1_balance,
                initial_verifier_2_balance
            );

            // Reward distribution parameters
            let total_reward = 1_000u128;
            let miner_reward_percentage = MinerRewardPercentage::get();
            let verifier_reward_percentage = VerifierRewardPercentage::get();
            let fee_split_treasury = FeeSplitTreasury::get();

            log::info!(
                "Reward Parameters: Total Reward: {}, Miner Percentage: {}, Verifier Percentage: {}, Treasury Split: {}",
                total_reward,
                miner_reward_percentage,
                verifier_reward_percentage,
                fee_split_treasury
            );

            // Execute direct_reward_distribution
            assert_ok!(TreasuryManager::direct_reward_distribution(
                frame_system::RawOrigin::Root.into(),
                miner_account.clone(),
                vec![verifier_1.clone(), verifier_2.clone()],
                total_reward
            ));

            // Calculate expected amounts
            let dev_fee = total_reward * (100 - fee_split_treasury as u128) / 100;
            let remaining_reward = total_reward - dev_fee;
            let miner_reward = remaining_reward * miner_reward_percentage as u128 / 100;
            let total_verifier_reward = remaining_reward - miner_reward;
            let per_verifier_reward = total_verifier_reward / 2;

            log::info!(
                "Calculated Rewards: Dev Fee: {}, Remaining Reward: {}, Miner Reward: {}, Total Verifier Reward: {}, Per Verifier Reward: {}",
                dev_fee,
                remaining_reward,
                miner_reward,
                total_verifier_reward,
                per_verifier_reward
            );

            // Assert balances
            assert_eq!(
                Balances::free_balance(&dev_account),
                initial_dev_balance + dev_fee
            );
            assert_eq!(
                Balances::free_balance(&miner_account),
                initial_miner_balance + miner_reward
            );
            assert_eq!(
                Balances::free_balance(&verifier_1),
                initial_verifier_1_balance + per_verifier_reward
            );
            assert_eq!(
                Balances::free_balance(&verifier_2),
                initial_verifier_2_balance + per_verifier_reward
            );

            // Assert treasury balance reduced by total_reward
            assert_eq!(
                Balances::free_balance(&treasury_account),
                initial_treasury_balance - total_reward
            );
        });
    }
}
