use crate::contract::execute;
use crate::contract::instantiate;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{PRICE_FEED_ADMINS, PRICE_FEED_GOV};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::Addr;

#[test]
fn test_set_latest_answer() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("admin", &[]);

    let resp = instantiate(deps.as_mut(), env.clone(), info.clone(), InstantiateMsg {});
    assert!(resp.is_ok());

    // Set latest answer by non-admin address
    let info = mock_info("random", &[]);
    let msg = ExecuteMsg::SetLatestAnswer(123.into());
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert!(res.is_err());

    // Set latest answer by admin address
    let info = mock_info("admin", &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    assert!(res.is_ok());
}

#[test]
fn test_set_admin() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let new_admin = Addr::unchecked("new_admin");

    let msg = InstantiateMsg {};
    let info = mock_info("creator", &[]);
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Set the governance
    let gov_addr = Addr::unchecked("gov_address");
    PRICE_FEED_GOV
        .save(deps.as_mut().storage, &gov_addr)
        .unwrap();

    // non-gov address should error
    let non_gov_info = mock_info("non_gov_address", &[]);
    let set_admin_msg = ExecuteMsg::SetAdmin {
        admin: new_admin.clone(),
        status: true,
    };

    let res = execute(deps.as_mut(), env, non_gov_info, set_admin_msg.clone());
    match res {
        Err(ContractError::Forbidden {}) => {} // expected
        _ => panic!("Unexpected error"),
    }

    let gov_info = mock_info(gov_addr.as_str(), &[]);
    let res = execute(deps.as_mut(), mock_env(), gov_info, set_admin_msg).unwrap();

    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "set_admin");
    assert_eq!(res.attributes[1].key, "admin");
    assert_eq!(res.attributes[1].value, new_admin.to_string());
    assert_eq!(res.attributes[2].key, "status");
    assert_eq!(res.attributes[2].value, "true");

    // check admin status
    let admin_status = PRICE_FEED_ADMINS
        .load(deps.as_ref().storage, new_admin)
        .unwrap();
    assert!(admin_status);
}
