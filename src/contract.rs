use std::thread::current;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Addr, Order};
//use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{MessagesResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{CURRENT_ID, MESSAGES,Message};

// version info for migration info
//const CONTRACT_NAME: &str = "crates.io:messages";
//const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    CURRENT_ID.save(deps.storage, &Uint128::zero().u128())?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddMessage { topic, message } => add_message(deps, info, topic, message),
    }
}

pub fn add_message(deps: DepsMut, info:MessageInfo, topic:String, message:String) -> Result<Response, ContractError> {

    let mut current_id = CURRENT_ID.load(deps.storage)?;
    let new_message = Message {id: Uint128::from(current_id), owner: info.sender, topic, message}; 
    current_id = current_id.checked_add(1).unwrap();
    MESSAGES.save(deps.storage, new_message.id.u128(), &new_message)?;
    CURRENT_ID.save(deps.storage, &current_id)?;
    Ok(Response::new()
        .add_attribute("execute", "add message")
        .add_attribute("message id", new_message.id)
        .add_attribute("topic", new_message.topic)
        .add_attribute("message text" , new_message.message)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCurrentId {  } => to_binary(&query_current_id(deps)?),
        QueryMsg::GetAllMessage {} => to_binary(&query_all_messages(deps)?),
        QueryMsg::GetMessagesByAddr { address } => to_binary(&query_messages_by_addr(deps, address)?),
        QueryMsg::GetMessagesByTopic { topic } => to_binary(&query_messages_by_topic(deps, topic)?),
        QueryMsg::GetMessagesById { id } => to_binary(&query_messages_by_id(deps, id)?),
    }
}

fn query_current_id(deps: Deps) -> StdResult<Uint128> {
    let current_id = CURRENT_ID.load(deps.storage)?;
    Ok(Uint128::from(current_id))
}

fn query_all_messages(deps: Deps) -> StdResult<MessagesResponse> {
    let messages: Vec<Message> = MESSAGES.range(deps.storage, None, None, Order::Ascending)
    .map(|item| item.unwrap().1)
    .collect();
    Ok(MessagesResponse { messages: { messages }})
}

fn query_messages_by_addr(deps: Deps, address: String) -> StdResult<MessagesResponse> {
    let messages: Vec<Message> = MESSAGES.range(deps.storage, None, None, Order::Ascending)
    .map(|item| item.unwrap().1)
    .filter(|message| message.owner == address)
    .collect();
    Ok(MessagesResponse { messages: { messages }})
}

fn query_messages_by_topic(deps: Deps, topic: String) -> StdResult<MessagesResponse> {
    let messages: Vec<Message> = MESSAGES.range(deps.storage, None, None, Order::Ascending)
    .map(|item| item.unwrap().1)
    .filter(|message| message.topic == topic)
    .collect();
    Ok(MessagesResponse { messages:{messages}})
}

fn query_messages_by_id(deps: Deps, id: Uint128) -> StdResult<MessagesResponse> {
    let messages: Vec<Message> = MESSAGES.range(deps.storage, None, None, Order::Ascending)
    .map(|item| item.unwrap().1)
    .filter(|message| message.id == id)
    .collect();
    Ok(MessagesResponse { messages:{messages}})
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, coins, from_binary};

    const SENDER: &str = "sender_address";
    const TOPIC: &str = "some funny topic";
    const MESSAGE_TEXT: &str = "ha ha, that's very funny";

    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg { };
        let info = mock_info(SENDER, &[]);
        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    fn create_single_message(deps: DepsMut, sender: &str) -> Result<Response, ContractError> {
        let msg_1 = ExecuteMsg::AddMessage { 
            topic: TOPIC.to_string(), 
            message: MESSAGE_TEXT.to_string()};        
        let res_1 = execute(
            deps, mock_env(), mock_info(sender, &[]), msg_1);
        res_1
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCurrentId {}).unwrap();
        let value: Uint128 = from_binary(&res).unwrap();
        assert_eq!(Uint128::zero(), value);
    }

    #[test]
    fn _add_message() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        // create message and get response
        let res = create_single_message(deps.as_mut(), SENDER);
        // let msg = ExecuteMsg::AddMessage { 
        //             topic: TOPIC.to_string(), 
        //             message: MESSAGE_TEXT.to_string()};
        // let res = execute(
        //     deps.as_mut(), mock_env(), mock_info(SENDER, &[]), msg).unwrap();
            
        // check attributes
        assert_eq!(
            res.unwrap().attributes, 
            vec![attr("execute", "add message"), 
                attr("message id", "0"),
                attr("topic", TOPIC.to_string()), 
                attr("message text", MESSAGE_TEXT.to_string())]);

        // check id incrementation
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCurrentId {}).unwrap();
        let value: Uint128 = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(1u128), value);
    }

    #[test]
    fn _query_all_messages() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        // create messages and check response for second message only
        // (single message response was tested in _add_message() test)
        let _res_1 = create_single_message(deps.as_mut(), SENDER);
        // let msg_1 = ExecuteMsg::AddMessage { 
        //     topic: TOPIC.to_string(), 
        //     message: MESSAGE_TEXT.to_string()};        
        // let _res_1 = execute(
        //     deps.as_mut(), mock_env(), mock_info(SENDER, &[]), msg_1);

        let topic_2 = "funniest topic";
        let msg_text_2 = "that's even better than before!";
        let msg_2 = ExecuteMsg::AddMessage { 
            topic: topic_2.to_string(), 
            message: msg_text_2.to_string()};
        let res_2 = execute(
            deps.as_mut(), mock_env(), mock_info(SENDER, &[]), msg_2).unwrap();

        // check attributes
        assert_eq!(
            res_2.attributes, 
        vec![attr("execute", "add message"), 
            attr("message id", "1"),
            attr("topic", topic_2), 
            attr("message text", msg_text_2)]); 

        // see if there are 2 messages saved
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllMessage {}).unwrap();
        let messages: MessagesResponse = from_binary(&res).unwrap();
        assert!(messages.messages.len() == 2);
    }

    #[test]
    fn _query_messages_by_owner() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        // creating 3 messages, 2 sent by the same sender
        let _res_msg = create_single_message(deps.as_mut(), SENDER);
        let _res_msg = create_single_message(deps.as_mut(), SENDER);
        let _res_msg = create_single_message(deps.as_mut(), "wladzioo");

        // there should be 3 messages in storage
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllMessage {}).unwrap();
        let messages: MessagesResponse = from_binary(&res).unwrap();
        assert!(messages.messages.len() == 3);

        // and only 1 message sent by wladzioo
        let res = query(
            deps.as_ref(), mock_env(), QueryMsg::GetMessagesByAddr {address: String::from("wladzioo")}).unwrap();
        let messages: MessagesResponse = from_binary(&res).unwrap();
        assert!(messages.messages.len() == 1);
    }

    #[test]
    fn _query_messages_by_id() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        // creating 3 messages
        let _res_msg = create_single_message(deps.as_mut(), SENDER);
        let _res_msg = create_single_message(deps.as_mut(), "wladzioo");
        let _res_msg = create_single_message(deps.as_mut(), SENDER);
        let res = query(
            deps.as_ref(), mock_env(), QueryMsg::GetMessagesById {id: Uint128::from(1u128)})
        .unwrap();

        // message with id = 1 was sent by wladzioo 
        let message: MessagesResponse = from_binary(&res).unwrap();
        assert!(message.messages[0].owner == "wladzioo");

    }

    #[test]
    fn _query_messages_by_topic() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let _res_msg = create_single_message(deps.as_mut(), "wladzioo");
        let _res_msg = create_single_message(deps.as_mut(), SENDER);
        let res = query(
            deps.as_ref(), mock_env(), QueryMsg::GetMessagesByTopic {topic: TOPIC.to_string()})
        .unwrap();
        let messages: MessagesResponse = from_binary(&res).unwrap();

        // there are 2 messages with the same topic
        assert!(messages.messages.len() == 2);

        // first message was sent by wladzioo, second by SENDER
        assert!(messages.messages[0].owner == "wladzioo");
        assert!(messages.messages[1].owner == SENDER);

        // checking ids incrementation
        assert_ne!(messages.messages[1].id, Uint128::zero());
        assert_ne!(messages.messages[1].id, Uint128::from(2u128));
    }
}
