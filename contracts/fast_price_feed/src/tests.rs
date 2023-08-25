use crate::helpers::{is_gov, only_signer, only_token_manager, only_updater};
use crate::execute::{enable_fast_price, disable_fast_price};
use crate::msg::ExecuteMsg;
use crate::execute::initialize;
use crate::state::*;

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Addr, Response, Timestamp, Uint256, Uint64};
use crate::errors::ContractError;

#[test]
fn test_set_compacted_prices() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let mock_info = mock_info("admin", &[]);
    env.block.time = Timestamp::from_seconds(1);

    crate::state::VAULT_ADDRESS
        .save(&mut deps.storage, &Addr::unchecked("mock_vault_address"))
        .unwrap();
    crate::state::CONFIG
        .save(
            &mut deps.storage,
            &Config {
                fast_price_events: Addr::unchecked("fast_price_events"),
                token_manager: Addr::unchecked("toke_manager"),
                max_deviation_basis_points: Uint256::zero(),
                max_price_update_delay: Uint64::zero(),
                price_duration: Uint64::zero(),
                min_block_interval: Uint64::zero(),
            },
        )
        .unwrap();

    crate::state::TOKEN_DATA
        .save(
            &mut deps.storage,
            &vec![
                TokenData {
                    token: Addr::unchecked("token0"),
                    token_precision: Uint256::one(),
                },
                TokenData {
                    token: Addr::unchecked("token1"),
                    token_precision: Uint256::one(),
                },
                TokenData {
                    token: Addr::unchecked("token2"),
                    token_precision: Uint256::one(),
                },
                TokenData {
                    token: Addr::unchecked("token3"),
                    token_precision: Uint256::one(),
                },
                TokenData {
                    token: Addr::unchecked("token4"),
                    token_precision: Uint256::one(),
                },
            ],
        )
        .unwrap();

    crate::state::PRICE_DATA_INTERVAL
        .save(&mut deps.storage, &Uint64::one())
        .unwrap();

    // generate price for functions and pack then in Uint256
    let prices: [u64; 4] = [1u64, 2u64, 3u64, 4u64];
    let mut combined_bytes = [0u8; 32];
    for (i, price) in prices.iter().enumerate() {
        combined_bytes[i * 8..(i + 1) * 8].copy_from_slice(&price.to_le_bytes());
    }

    let result = crate::contract::execute(
        deps.as_mut(),
        env,
        mock_info,
        ExecuteMsg::SetCompactedPrices {
            price_bit_array: vec![Uint256::from_le_bytes(combined_bytes)],
            timestamp: Uint64::from(1u32),
        },
    );
    assert!(result.is_ok());

    for i in 0..prices.len() - 1 {
        assert_eq!(
            PRICES
                .load(&deps.storage, &Addr::unchecked(format!("token{}", i)))
                .unwrap(),
            Uint256::from(prices[i]).checked_mul(crate::execute::PRICE_PRECISION).unwrap()
        );
    }

    // there is no price for token4
    assert_eq!(
        PRICES
            .load(&deps.storage, &Addr::unchecked("token5"))
            .unwrap_or_default(),
        Uint256::zero()
    );
}

#[test]
fn test_initialize() {
    let mut deps = mock_dependencies();
    let sender = Addr::unchecked("sender");

    let min_auth = Uint256::from(12345u64);
    let signers = vec![Addr::unchecked("signer1"), Addr::unchecked("signer2")];
    let updaters = vec![Addr::unchecked("updater1"), Addr::unchecked("updater2")];

    // Test initialization
    GOV.save(deps.as_mut().storage, &sender).ok();
    let result = initialize(
        deps.as_mut(),
        sender.clone(),
        min_auth.clone(),
        signers.clone(),
        updaters.clone(),
    );
    assert_eq!(
        result,
        Ok(Response::new().add_attribute("method", "intialize"))
    );

    // Check if signers and updaters are saved correctly
    for signer in signers.iter() {
        assert_eq!(IS_SIGNER.load(deps.as_mut().storage, signer).unwrap(), true);
    }

    for updater in updaters.iter() {
        assert_eq!(
            IS_UPDATER.load(deps.as_mut().storage, updater).unwrap(),
            true
        );
    }

    // Check if min_auth is saved correctly
    assert_eq!(MIN_AUTH.load(deps.as_mut().storage).unwrap(), min_auth);

    // Check if contract is marked as initialized
    assert_eq!(IS_INITIALIZED.load(deps.as_mut().storage).unwrap(), true);

    // Test re-initialization
    let result = initialize(deps.as_mut(), sender, min_auth, signers, updaters);
    assert_eq!(result, Err(ContractError::AlreadyInitialized {}));
}

#[test]
fn test_is_gov() {
    let mut deps = mock_dependencies();
    let gov_address = Addr::unchecked("governor");
    let other_address = Addr::unchecked("other");

    // Set the governor's address in storage
    GOV.save(deps.as_mut().storage, &gov_address).unwrap();

    // Test with governor's address
    let result = is_gov(deps.as_ref(), &gov_address);
    assert!(result.is_ok());

    // Test with another address
    let result = is_gov(deps.as_ref(), &other_address);
    assert_eq!(
        result.unwrap_err(),
        crate::errors::ContractError::Forbidden {}
    );
}
#[test]
fn test_only_updater() {
    let mut deps = mock_dependencies();
    let updater_addr = Addr::unchecked("updater");
    let non_updater_addr = Addr::unchecked("non_updater");

    // Set the updater in storage
    IS_UPDATER
        .save(deps.as_mut().storage, &updater_addr, &true)
        .unwrap();

    // Test with updater address
    assert_eq!(only_updater(deps.as_ref(), &updater_addr), Ok(()));

    // Test with non-updater address
    assert_eq!(
        only_updater(deps.as_ref(), &non_updater_addr),
        Err(ContractError::Forbidden {})
    );
}

#[test]
fn test_only_signer() {
    let mut deps = mock_dependencies();
    let signer_addr = Addr::unchecked("signer");
    let non_signer_addr = Addr::unchecked("non_signer");

    // Set the signer in storage
    IS_SIGNER
        .save(deps.as_mut().storage, &signer_addr, &true)
        .unwrap();

    // Test with signer address
    assert_eq!(only_signer(deps.as_ref(), &signer_addr), Ok(()));

    // Test with non-signer address
    assert_eq!(
        only_signer(deps.as_ref(), &non_signer_addr),
        Err(ContractError::Forbidden {})
    );
}

#[test]
fn test_disable_fast_price() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let sender = Addr::unchecked("sender_address");

    IS_SIGNER
        .save(deps.as_mut().storage, &sender, &true)
        .unwrap();
    let res = disable_fast_price(deps.as_mut(), env.clone(), sender.clone()).unwrap();

    // Check attributes in the response
    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "disable_fast_price");
    assert_eq!(res.attributes[1].key, "sender");
    assert_eq!(res.attributes[1].value, "sender_address");

    // Check if the vote was saved correctly
    let vote: bool = DISABLE_FAST_PRICE_VOTES
        .load(deps.as_ref().storage, &sender)
        .unwrap();
    assert_eq!(vote, true);
}

#[test]
fn test_enable_fast_price() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let sender = Addr::unchecked("sender_address");

    IS_SIGNER
        .save(deps.as_mut().storage, &sender, &true)
        .unwrap();

    // should fail as already enabled
    let res = enable_fast_price(deps.as_mut(), env.clone(), sender.clone());
    assert_eq!(res.unwrap_err(), ContractError::AlreadyEnabled {});

    // should work work
    disable_fast_price(deps.as_mut(), env.clone(), sender.clone()).unwrap();
    let res = enable_fast_price(deps.as_mut(), env.clone(), sender.clone()).unwrap();

    // Check attributes in the response
    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "enable_fast_price");
    assert_eq!(res.attributes[1].key, "sender");
    assert_eq!(res.attributes[1].value, "sender_address");

    // Check if the vote was saved correctly
    let vote: bool = DISABLE_FAST_PRICE_VOTES
        .load(deps.as_ref().storage, &sender)
        .unwrap();
    assert_eq!(vote, false);
}

#[test]
fn test_only_token_manager() {
    let mut deps = mock_dependencies();
    let token_manager_addr = Addr::unchecked("token_manager");
    let non_token_manager_addr = Addr::unchecked("non_token_manager");

    // Set the token manager in storage
    TOKEN_MANAGER
        .save(deps.as_mut().storage, &token_manager_addr)
        .unwrap();

    // Test with token manager address
    assert_eq!(
        only_token_manager(deps.as_ref(), &token_manager_addr),
        Ok(())
    );

    // Test with non-token manager address
    assert_eq!(
        only_token_manager(deps.as_ref(), &non_token_manager_addr),
        Err(ContractError::Forbidden {})
    );
}
