use crate::contract::execute;
use crate::execute::*;
use crate::helpers::{is_gov, only_signer, only_token_manager, only_updater};
use crate::msg::ExecuteMsg;
use crate::state::*;

use crate::errors::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Addr, DepsMut, MessageInfo, Uint256, Uint64};

fn generate_config() -> Config {
    Config {
        price_duration: Uint64::zero(),
        max_price_update_delay: Uint64::zero(),
        min_block_interval: Uint64::zero(),
        max_deviation_basis_points: Uint256::zero(),
        fast_price_events: Addr::unchecked(""),
        token_manager: Addr::unchecked(""),
    }
}

fn setup_with_gov(deps: DepsMut) -> MessageInfo {
    let gov = Addr::unchecked("gov");
    GOV.save(deps.storage, &gov).unwrap();
    let info = mock_info(gov.as_str(), &[]);
    info
}

fn setup_with_updater(deps: DepsMut) -> MessageInfo {
    let updater = Addr::unchecked("updated");
    IS_UPDATER.save(deps.storage, &updater, &true).unwrap();
    let info = mock_info(updater.as_str(), &[]);
    info
}

#[test]
fn test_set_compacted_prices() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let mock_info = mock_info("admin", &[]);

    VAULT_ADDRESS
        .save(&mut deps.storage, &Addr::unchecked("mock_vault_address"))
        .unwrap();
    CONFIG
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

    TOKEN_DATA
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

    PRICE_DATA_INTERVAL
        .save(deps.as_mut().storage, &Uint64::one())
        .unwrap();
    MAX_TIME_DEVIATION
        .save(deps.as_mut().storage, &1000u64)
        .unwrap();

    // generate price for functions and pack then in Uint256
    let prices: [u64; 4] = [1u64, 2u64, 3u64, 4u64];
    let mut combined_bytes = [0u8; 32];
    for (i, price) in prices.iter().enumerate() {
        combined_bytes[i * 8..(i + 1) * 8].copy_from_slice(&price.to_le_bytes());
    }

    let msg = ExecuteMsg::SetCompactedPrices {
        price_bit_array: vec![Uint256::from_le_bytes(combined_bytes)],
        timestamp: env.block.time.seconds().into(),
    };

    let result = execute(deps.as_mut(), env, mock_info, msg);
    assert!(result.is_ok());

    for i in 0..prices.len() - 1 {
        assert_eq!(
            PRICES
                .load(&deps.storage, &Addr::unchecked(format!("token{}", i)))
                .unwrap(),
            Uint256::from(prices[i])
                .checked_mul(crate::execute::PRICE_PRECISION)
                .unwrap()
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
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let min_auth = Uint256::from(12345u64);
    let signers = vec![Addr::unchecked("signer1"), Addr::unchecked("signer2")];
    let updaters = vec![Addr::unchecked("updater1"), Addr::unchecked("updater2")];

    // Test initialization
    GOV.save(deps.as_mut().storage, &sender).ok();
    let msg = ExecuteMsg::Initialize {
        min_auth,
        signers: signers.clone(),
        updaters: updaters.clone(),
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    assert!(res.is_ok());

    // Check if signers and updaters are saved correctly
    for signer in signers.iter() {
        assert_eq!(IS_SIGNER.load(deps.as_mut().storage, signer).unwrap(), true);
    }

    for updater in updaters.iter() {
        assert!(IS_UPDATER.load(deps.as_mut().storage, updater).unwrap(),)
    }

    // Check if min_auth is saved correctly
    assert_eq!(MIN_AUTH.load(deps.as_mut().storage).unwrap(), min_auth);

    // Check if contract is marked as initialized
    assert_eq!(IS_INITIALIZED.load(deps.as_mut().storage).unwrap(), true);

    // Test re-initialization
    let msg = ExecuteMsg::Initialize {
        min_auth,
        signers: signers.clone(),
        updaters: updaters.clone(),
    };
    let res = execute(deps.as_mut(), env, info, msg);
    assert_eq!(res.unwrap_err(), ContractError::AlreadyInitialized {});
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

#[test]
fn test_set_signer() {
    let mut deps = mock_dependencies();
    let info = setup_with_gov(deps.as_mut());
    let account = Addr::unchecked("account");
    let env = mock_env();

    let msg = ExecuteMsg::SetSigner {
        account: account.clone(),
        is_active: true,
    };
    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());

    let status = IS_SIGNER.load(deps.as_ref().storage, &account).unwrap();
    assert!(status);
}

#[test]
fn test_set_updater() {
    let mut deps = mock_dependencies();
    let info = setup_with_gov(deps.as_mut());
    let account = Addr::unchecked("account");
    let env = mock_env();

    let msg = ExecuteMsg::SetUpdater {
        account: account.clone(),
        is_active: true,
    };
    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());

    let status = IS_UPDATER.load(deps.as_ref().storage, &account).unwrap();
    assert!(status);
}

#[test]
fn test_set_fast_price_events() {
    let mut deps = mock_dependencies();
    let info = setup_with_gov(deps.as_mut());
    let account = Addr::unchecked("fast_price_event");
    let env = mock_env();

    //init with defualt config
    CONFIG
        .save(deps.as_mut().storage, &generate_config())
        .unwrap();

    let msg = ExecuteMsg::SetFastPriceEvents {
        fast_price_events: account.clone(),
    };
    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config.fast_price_events, account);
}

#[test]
fn test_set_vault_price_feed() {
    let mut deps = mock_dependencies();
    let info = setup_with_gov(deps.as_mut());
    let account = Addr::unchecked("vault_address");
    let env = mock_env();

    let msg = ExecuteMsg::SetVaultPriceFeed {
        vault_price_feed: account.clone(),
    };
    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());

    let vault = VAULT_ADDRESS.load(deps.as_ref().storage).unwrap();
    assert_eq!(vault, account);
}

#[test]
fn test_max_time_deviation() {
    let mut deps = mock_dependencies();
    let info = setup_with_gov(deps.as_mut());
    let env = mock_env();

    let msg = ExecuteMsg::SetMaxTimeDeviation {
        max_time_deviation: Uint64::one(),
    };
    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());

    let time_diviation = MAX_TIME_DEVIATION.load(deps.as_ref().storage).unwrap();
    assert_eq!(time_diviation, 1);
}

#[test]
fn test_set_price_duration() {
    let mut deps = mock_dependencies();
    let info = setup_with_gov(deps.as_mut());
    let env = mock_env();

    CONFIG
        .save(deps.as_mut().storage, &generate_config())
        .unwrap();

    let msg = ExecuteMsg::SetPriceDuration {
        price_duration: Uint64::one(),
    };
    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config.price_duration.u64(), 1);
}

#[test]
fn test_set_max_price_update_delay() {
    let mut deps = mock_dependencies();
    let info = setup_with_gov(deps.as_mut());
    let env = mock_env();

    CONFIG
        .save(deps.as_mut().storage, &generate_config())
        .unwrap();

    let msg = ExecuteMsg::SetMaxPriceUpdateDelay {
        max_price_update_delay: Uint64::one(),
    };
    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config.max_price_update_delay.u64(), 1);
}

#[test]
fn test_set_basis_points() {
    let mut deps = mock_dependencies();
    let info = setup_with_gov(deps.as_mut());
    let env = mock_env();

    SPREAD_BASIS_POINT_STATE
        .save(
            deps.as_mut().storage,
            &SpreadBasisPoint {
                spread_basis_points_if_chain_error: Uint256::zero(),
                spread_basis_points_if_inactive: Uint256::zero(),
            },
        )
        .unwrap();

    let msg = ExecuteMsg::SetSpreadBasisPointsIfChainError {
        spread_basis_points_if_chain_error: Uint256::one(),
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    assert!(res.is_ok());

    let msg = ExecuteMsg::SetSpreadBasisPointsIfInactive {
        spread_basis_points_if_inactive: Uint256::one(),
    };
    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());

    let spread = SPREAD_BASIS_POINT_STATE
        .load(deps.as_ref().storage)
        .unwrap();
    assert_eq!(spread.spread_basis_points_if_chain_error, Uint256::one());
    assert_eq!(spread.spread_basis_points_if_inactive, Uint256::one());
}

#[test]
fn test_set_prices() {
    let mut deps = mock_dependencies();
    let info = setup_with_updater(deps.as_mut());
    let env = mock_env();

    let tokens = vec![Addr::unchecked("token0"), Addr::unchecked("token1")];
    let prices = vec![Uint256::from(100u64), Uint256::from(200u64)];
    let timestamp = env.block.time;

    CONFIG
        .save(deps.as_mut().storage, &generate_config())
        .unwrap();
    MAX_TIME_DEVIATION
        .save(deps.as_mut().storage, &1000u64)
        .unwrap();
    VAULT_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("vault"))
        .unwrap();
    PRICE_DATA_INTERVAL
        .save(deps.as_mut().storage, &Uint64::one())
        .unwrap();

    let msg = ExecuteMsg::SetPrices {
        tokens,
        prices: prices.clone(),
        timestamp: Uint64::from(timestamp.seconds()),
    };
    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());

    // check prices
    for i in 0..prices.len() {
        let price = PRICES
            .load(&deps.storage, &Addr::unchecked(format!("token{}", i)))
            .unwrap();
        assert_eq!(price, prices[i]);
    }
}
