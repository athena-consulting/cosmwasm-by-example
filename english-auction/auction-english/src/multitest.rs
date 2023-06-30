#![cfg(test)]
use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, QueryMsg, QueryOptions, AuctionResponse, AuctionsResponse, TokenTimestampOffset,
};
use crate::state::{Auction, AuctionStatus, AuctionBid};
use cosmwasm_std::{Addr, Empty, Timestamp, coin, coins, Coin, Decimal, Uint128};
use cw721::{Cw721QueryMsg, OwnerOfResponse};
use cw721_base::msg::{ExecuteMsg as Cw721ExecuteMsg, MintMsg};
use cw_multi_test::{App, AppBuilder, BankSudo, Contract, ContractWrapper, Executor, SudoMsg as CwSudoMsg};
use royality_cw721::msg::{InstantiateMsg as Pg721InstantiateMsg, RoyaltyInfoResponse};
use royality_cw721::state::CollectionInfo;

const TOKEN_ID: &str = "123";
const CREATION_FEE: u128 = 1_000_000_000;
const INITIAL_BALANCE: u128 = 2000;
const NATIVE_DENOM: &str = "ujunox";
const USER: &str = "USER";

// Governance parameters
const TRADING_FEE_BPS: u64 = 200; // 2%
const TEN_MINS: u64 = 60 * 10; // 24 hours (in seconds)
const ONE_DAY: u64 = 24 * 60 * 60; // 24 hours (in seconds)
const SIX_MOS: u64 = 180 * 24 * 60 * 60; // 6 months (in seconds)

fn custom_mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(USER),
                vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(3_000_000),
                }],
            )
            .unwrap();
    })
}

pub fn contract_auction_english() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    // .with_sudo(crate::sudo::sudo)
    // .with_reply(crate::execute::reply);
    Box::new(contract)
}

pub fn contract_pg721() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        royality_cw721::contract::execute,
        royality_cw721::contract::instantiate,
        royality_cw721::contract::query,
    );
    Box::new(contract)
}

fn setup_block_time(router: &mut App, seconds: u64) {
    let mut block = router.block_info();
    block.time = Timestamp::from_seconds(seconds);
    router.set_block(block);
}

// Instantiates all needed contracts for testing
fn setup_contracts(
    router: &mut App,
    creator: &Addr,
) -> Result<(Addr, Addr), ContractError> {
    // Setup media contract
    let pg721_id = router.store_code(contract_pg721());
    let msg = Pg721InstantiateMsg {
        name: String::from("Test Coin"),
        symbol: String::from("TEST"),
        minter: creator.to_string(),
        collection_info: CollectionInfo {
            creator: creator.to_string(),
            description: String::from("Passage Monkeys"),
            image:
                "ipfs://bafybeigi3bwpvyvsmnbj46ra4hyffcxdeaj6ntfk5jpic5mx27x6ih2qvq/images/1.png"
                    .to_string(),
            external_link: Some("https://example.com/external.html".to_string()),
            royalty_info: Some(RoyaltyInfoResponse {
                payment_address: creator.to_string(),
                share: Decimal::percent(10),
            }),
        },
    };
    let collection = router
        .instantiate_contract(
            pg721_id,
            creator.clone(),
            &msg,
            &coins(CREATION_FEE, NATIVE_DENOM),
            "NFT",
            None,
        )
        .unwrap();

    // Instantiate auction_english contract
    let auction_english_id = router.store_code(contract_auction_english());
    let msg = crate::msg::InstantiateMsg {
        cw721_address: collection.to_string(),
        denom: String::from(NATIVE_DENOM),
        collector_address: creator.to_string(),
        trading_fee_bps: TRADING_FEE_BPS,
        operators: vec!["operator".to_string()],
        min_price: Uint128::from(5u128),
        min_bid_increment: Uint128::from(3u128),
        min_duration: ONE_DAY,
        max_duration: SIX_MOS,
        closed_duration: ONE_DAY,
        buffer_duration: TEN_MINS,
    };
    let auction_english = router
        .instantiate_contract(
            auction_english_id,
            creator.clone(),
            &msg,
            &[],
            "English Auction",
            None,
        )
        .unwrap();

    Ok((auction_english, collection))
}

// Intializes accounts with balances
fn setup_accounts(router: &mut App) -> Result<(Addr, Addr, Addr, Addr), ContractError> {
    let owner: Addr = Addr::unchecked("owner");
    let bidder: Addr = Addr::unchecked("bidder");
    let bidder2: Addr = Addr::unchecked("bidder2");
    let creator: Addr = Addr::unchecked("creator");
    let creator_funds: Vec<Coin> = coins(CREATION_FEE, NATIVE_DENOM);
    let funds: Vec<Coin> = coins(INITIAL_BALANCE, NATIVE_DENOM);
    router
        .sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: owner.to_string(),
                amount: funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();
    router
        .sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: bidder.to_string(),
                amount: funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();
    router
        .sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: bidder2.to_string(),
                amount: funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();
    router
        .sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: creator.to_string(),
                amount: creator_funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

    // Check native balances
    let owner_native_balances = router.wrap().query_all_balances(owner.clone()).unwrap();
    assert_eq!(owner_native_balances, funds);
    let bidder_native_balances = router.wrap().query_all_balances(bidder.clone()).unwrap();
    assert_eq!(bidder_native_balances, funds);
    let bidder2_native_balances = router.wrap().query_all_balances(bidder2.clone()).unwrap();
    assert_eq!(bidder2_native_balances, funds);
    let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(creator_native_balances, creator_funds);

    Ok((owner, bidder, creator, bidder2))
}

// Mints an NFT for a creator
fn mint(router: &mut App, creator: &Addr, collection: &Addr, token_id: String) {
    let mint_for_creator_msg = Cw721ExecuteMsg::Mint(MintMsg {
        token_id: token_id,
        owner: creator.clone().to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Empty {},
    });
    let res = router.execute_contract(
        creator.clone(),
        collection.clone(),
        &mint_for_creator_msg,
        &[],
    );
    assert!(res.is_ok());
}

fn approve(
    router: &mut App,
    creator: &Addr,
    collection: &Addr,
    auction_english: &Addr,
    token_id: String,
) {
    let approve_msg = Cw721ExecuteMsg::<Empty>::Approve {
        spender: auction_english.to_string(),
        token_id: token_id,
        expires: None,
    };
    let res = router.execute_contract(creator.clone(), collection.clone(), &approve_msg, &[]);
    assert!(res.is_ok());
}

fn auction(
    router: &mut App,
    creator: &Addr,
    auction_english: &Addr,
    token_id: String,
    start_time: Timestamp,
    end_time: Timestamp,
    starting_price: u128,
    reserve_price: u128,
    funds_recipient: Option<String>
) {
    let set_auction = ExecuteMsg::SetAuction {
        token_id,
        start_time,
        end_time,
        starting_price: coin(starting_price, NATIVE_DENOM),
        reserve_price: Some(coin(reserve_price, NATIVE_DENOM)),
        funds_recipient,
    };
    let res = router.execute_contract(creator.clone(), auction_english.clone(), &set_auction, &[]);
    assert!(res.is_ok());
}

fn auction_bid(
    router: &mut App,
    creator: &Addr,
    auction_english: &Addr,
    token_id: String,
    price: u128,
) {
    let coin_send = coin(price, NATIVE_DENOM);
    let set_auction_bid = ExecuteMsg::SetAuctionBid {
        token_id: token_id,
        price: coin_send.clone(),
    };
    let res = router.execute_contract(creator.clone(), auction_english.clone(), &set_auction_bid, &[coin_send]);
    assert!(res.is_ok());
}

#[test]
fn try_auction_creation_and_removal() {
    let mut router = custom_mock_app();
    let block_time = router.block_info().time;
    // Setup intial accounts
    let (_owner, _bidder, creator, _bidder2) = setup_accounts(&mut router).unwrap();

    // Instantiate and configure contracts
    let (auction_english, collection) = setup_contracts(&mut router, &creator).unwrap();

    // Mint NFT for owner
    mint(&mut router, &creator, &collection, TOKEN_ID.to_string());
    approve(&mut router, &creator, &collection, &auction_english, TOKEN_ID.to_string());

    // Should error with duration lower than min
    let set_auction = ExecuteMsg::SetAuction {
        token_id: TOKEN_ID.to_string(),
        start_time: block_time.plus_seconds(ONE_DAY),
        end_time: block_time.plus_seconds(ONE_DAY),
        starting_price: coin(110, NATIVE_DENOM),
        reserve_price: Some(coin(210, NATIVE_DENOM)),
        funds_recipient: None,
    };
    let res = router.execute_contract(creator.clone(), auction_english.clone(), &set_auction, &[]);
    assert!(res.is_err());

    // Should error with duration above_max
    let set_auction = ExecuteMsg::SetAuction {
        token_id: TOKEN_ID.to_string(),
        start_time: block_time.plus_seconds(ONE_DAY),
        end_time: block_time.plus_seconds(SIX_MOS * 2),
        starting_price: coin(110, NATIVE_DENOM),
        reserve_price: Some(coin(210, NATIVE_DENOM)),
        funds_recipient: None,
    };
    let res = router.execute_contract(creator.clone(), auction_english.clone(), &set_auction, &[]);
    assert!(res.is_err());

    // Should error with invalid denom
    let set_auction = ExecuteMsg::SetAuction {
        token_id: TOKEN_ID.to_string(),
        start_time: block_time.plus_seconds(ONE_DAY),
        end_time: block_time.plus_seconds(ONE_DAY * 2),
        starting_price: coin(110, NATIVE_DENOM),
        reserve_price: Some(coin(210, "ujuno")),
        funds_recipient: None,
    };
    let res = router.execute_contract(creator.clone(), auction_english.clone(), &set_auction, &[]);
    assert!(res.is_err());

    // Should error with reserve price below starting price
    let set_auction = ExecuteMsg::SetAuction {
        token_id: TOKEN_ID.to_string(),
        start_time: block_time.plus_seconds(ONE_DAY),
        end_time: block_time.plus_seconds(ONE_DAY * 2),
        starting_price: coin(200, NATIVE_DENOM),
        reserve_price: Some(coin(100, NATIVE_DENOM)),
        funds_recipient: None,
    };
    let res = router.execute_contract(creator.clone(), auction_english.clone(), &set_auction, &[]);
    assert!(res.is_err());

    // An auction is made by the creator
    auction(
        &mut router,
        &creator,
        &auction_english,
        TOKEN_ID.to_string(),
        block_time.plus_seconds(ONE_DAY),
        block_time.plus_seconds(ONE_DAY * 2),
        110u128,
        210u128,
        None,
    );

    // Validate Auction data is correct
    let query_auction = QueryMsg::Auction {
        token_id: TOKEN_ID.to_string(),
    };
    let res: AuctionResponse = router
        .wrap()
        .query_wasm_smart(auction_english.clone(), &query_auction)
        .unwrap();

    let current_auction = match res.auction {
        Some(auction) => Ok(auction),
        None => Err("Auction not found")
    }.unwrap();
    assert_eq!(Auction {
        token_id: TOKEN_ID.to_string(),
        start_time: block_time.plus_seconds(ONE_DAY),
        end_time: block_time.plus_seconds(ONE_DAY * 2),
        starting_price: coin(110, NATIVE_DENOM),
        reserve_price: Some(coin(210, NATIVE_DENOM)),
        seller: creator.clone(),
        funds_recipient: None,
        highest_bid: None,
    }, current_auction);
    
    // Check NFT is transferred to auction_english contract
    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: TOKEN_ID.to_string(),
        include_expired: None,
    };
    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, auction_english.to_string());

    // Close an auction with no bids
    let close_auction = ExecuteMsg::CloseAuction {
        token_id: TOKEN_ID.to_string(),
        accept_highest_bid: false
    };
    let res = router.execute_contract(creator.clone(), auction_english.clone(), &close_auction, &[]);
    assert!(res.is_ok());

    // Validate Auction is deleted
    let query_auction = QueryMsg::Auction {
        token_id: TOKEN_ID.to_string(),
    };
    let res: AuctionResponse = router
        .wrap()
        .query_wasm_smart(auction_english.clone(), &query_auction)
        .unwrap();
    assert_eq!(res.auction, None);

    // Check NFT is transferred back to the owner
    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: TOKEN_ID.to_string(),
        include_expired: None,
    };
    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, creator.to_string());
}

#[test]
fn try_auction_bid_creation_and_removal() {
    let mut router = custom_mock_app();
    let block_time = router.block_info().time;
    // Setup intial accounts
    let (_owner, bidder, creator, bidder2) = setup_accounts(&mut router).unwrap();

    // Instantiate and configure contracts
    let (auction_english, collection) = setup_contracts(&mut router, &creator).unwrap();

    // Mint NFT for owner
    mint(&mut router, &creator, &collection, TOKEN_ID.to_string());
    approve(&mut router, &creator, &collection, &auction_english, TOKEN_ID.to_string());
    auction(
        &mut router,
        &creator,
        &auction_english,
        TOKEN_ID.to_string(),
        block_time.plus_seconds(ONE_DAY),
        block_time.plus_seconds(ONE_DAY * 2),
        110u128,
        210u128,
        None,
    );

    // AuctionBid creation should error without a matching auction
    let set_auction_bid = ExecuteMsg::SetAuctionBid {
        token_id: String::from("999"),
        price: coin(120u128, NATIVE_DENOM),
    };
    let res = router.execute_contract(bidder.clone(), auction_english.clone(), &set_auction_bid, &[]);
    assert_eq!(&res.unwrap_err().root_cause().to_string(), "auction_english::state::Auction not found");

    // AuctionBid creation should error when auction status is pending
    let set_auction_bid = ExecuteMsg::SetAuctionBid {
        token_id: TOKEN_ID.to_string(),
        price: coin(120u128, NATIVE_DENOM),
    };
    let res = router.execute_contract(bidder.clone(), auction_english.clone(), &set_auction_bid, &[]);
    assert_eq!(&res.unwrap_err().root_cause().to_string(), "Auction invalid status: Pending");

    setup_block_time(&mut router, block_time.plus_seconds(ONE_DAY + 10u64).seconds());

    // AuctionBid creation should error when funds are not sent
    let set_auction_bid = ExecuteMsg::SetAuctionBid {
        token_id: TOKEN_ID.to_string(),
        price: coin(120u128, NATIVE_DENOM),
    };
    let res = router.execute_contract(bidder.clone(), auction_english.clone(), &set_auction_bid, &[]);
    assert_eq!(&res.unwrap_err().root_cause().to_string(), "No funds sent");

    // AuctionBid creation should error when bid is below starting price
    let set_auction_bid = ExecuteMsg::SetAuctionBid {
        token_id: TOKEN_ID.to_string(),
        price: coin(100u128, NATIVE_DENOM),
    };
    let res = router.execute_contract(bidder.clone(), auction_english.clone(), &set_auction_bid, &[coin(100u128, NATIVE_DENOM)]);
    assert_eq!(&res.unwrap_err().root_cause().to_string(), "Auction bid too low");

    let bidder_balance_a = router.wrap().query_all_balances(bidder.clone()).unwrap().into_iter().nth(0).unwrap();
    let bidder2_balance_a = router.wrap().query_all_balances(bidder2.clone()).unwrap().into_iter().nth(0).unwrap();

    // AuctionBid creation should error when bid is less than or equal to the highest bid + minimum increment
    auction_bid(&mut router, &bidder, &auction_english, TOKEN_ID.to_string(), 140u128);
    let set_auction_bid = ExecuteMsg::SetAuctionBid {
        token_id: TOKEN_ID.to_string(),
        price: coin(142u128, NATIVE_DENOM),
    };
    let res = router.execute_contract(bidder.clone(), auction_english.clone(), &set_auction_bid, &[coin(142u128, NATIVE_DENOM)]);
    assert_eq!(&res.unwrap_err().root_cause().to_string(), "Auction bid too low");

    // Verify that new auction bids update the auction obj
    auction_bid(&mut router, &bidder2, &auction_english, TOKEN_ID.to_string(), 150u128);
    let query_auction = QueryMsg::Auction {
        token_id: TOKEN_ID.to_string()
    };
    let res: AuctionResponse = router
        .wrap()
        .query_wasm_smart(auction_english.clone(), &query_auction)
        .unwrap();
    assert_eq!(Auction {
        token_id: TOKEN_ID.to_string(),
        seller: creator.clone(),
        start_time: block_time.plus_seconds(ONE_DAY),
        end_time: block_time.plus_seconds(ONE_DAY * 2),
        starting_price: coin(110u128, NATIVE_DENOM),
        reserve_price: Some(coin(210u128, NATIVE_DENOM)),
        funds_recipient: None,
        highest_bid: Some(AuctionBid {
            bidder: bidder2.clone(),
            price: coin(150u128, NATIVE_DENOM),
        }),
    }, res.auction.unwrap());

    // Verify that new auction bids refund the previous high bidder
    let bidder_balance_b = router.wrap().query_all_balances(bidder.clone()).unwrap().into_iter().nth(0).unwrap();
    let bidder2_balance_b = router.wrap().query_all_balances(bidder2.clone()).unwrap().into_iter().nth(0).unwrap();
    assert_eq!(bidder_balance_a.amount, bidder_balance_b.amount);
    assert_eq!(bidder2_balance_a.amount - Uint128::from(150u128), bidder2_balance_b.amount);

    // Auction with bids can be closed, and the highest bid can be accepted
    let close_auction = ExecuteMsg::CloseAuction {
        token_id: TOKEN_ID.to_string(),
        accept_highest_bid: true
    };
    let res = router.execute_contract(creator.clone(), auction_english.clone(), &close_auction, &[]);
    assert!(res.is_ok());
    
    // Check NFT is transferred back to the bidder
    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: TOKEN_ID.to_string(),
        include_expired: None,
    };
    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, bidder2.to_string());

    // Check balances, validate that the bidder was debited, and that the seller was credited
    let bidder2_balance_c = router.wrap().query_all_balances(bidder2.clone()).unwrap().into_iter().nth(0).unwrap();
    let owner_balance = router.wrap().query_all_balances(creator.clone()).unwrap().into_iter().nth(0);
    assert_eq!(bidder2_balance_a.amount - Uint128::from(150u128), bidder2_balance_c.amount);
    assert_eq!(Uint128::from(150u128), owner_balance.unwrap().amount);
}

#[test]
fn try_auction_bid_reserve_price_met() {
    let mut router = custom_mock_app();
    let block_time = router.block_info().time;
    // Setup intial accounts
    let (_owner, bidder, creator, _bidder2) = setup_accounts(&mut router).unwrap();

    // Instantiate and configure contracts
    let (auction_english, collection) = setup_contracts(&mut router, &creator).unwrap();

    let prev_bidder_balance = router.wrap().query_all_balances(bidder.clone()).unwrap().into_iter().nth(0).unwrap();

    // Mint NFT for owner
    mint(&mut router, &creator, &collection, TOKEN_ID.to_string());
    approve(&mut router, &creator, &collection, &auction_english, TOKEN_ID.to_string());
    auction(
        &mut router,
        &creator,
        &auction_english,
        TOKEN_ID.to_string(),
        block_time.plus_seconds(ONE_DAY),
        block_time.plus_seconds(ONE_DAY * 2),
        110u128,
        210u128,
        None,
    );

    // Meet reserve price
    let bid_amount = 220u128;
    setup_block_time(&mut router, block_time.plus_seconds(ONE_DAY + 10u64).seconds());
    auction_bid(&mut router, &bidder, &auction_english, TOKEN_ID.to_string(), 220u128);

    // Verify auctions that have met reserve price cannot be closed
    let close_auction = ExecuteMsg::CloseAuction {
        token_id: TOKEN_ID.to_string(),
        accept_highest_bid: false
    };
    let res = router.execute_contract(creator.clone(), auction_english.clone(), &close_auction, &[]);
    assert_eq!(&res.unwrap_err().root_cause().to_string(), "Reserve price restriction: must finalize auction when reserve price is met");

    // Auction cannot be finalized while Auction is still open
    let finalize_auction = ExecuteMsg::FinalizeAuction {
        token_id: TOKEN_ID.to_string(),
    };
    let res = router.execute_contract(bidder.clone(), auction_english.clone(), &finalize_auction, &[]);
    assert_eq!(&res.unwrap_err().root_cause().to_string(), "Auction invalid status: Open");

    // Auction can be finalized when Auction is closed
    setup_block_time(&mut router, block_time.plus_seconds(ONE_DAY * 2 + 10u64).seconds());
    let finalize_auction = ExecuteMsg::FinalizeAuction {
        token_id: TOKEN_ID.to_string(),
    };
    let res = router.execute_contract(bidder.clone(), auction_english.clone(), &finalize_auction, &[]);
    assert!(res.is_ok());

    // Check NFT is transferred to the bidder
    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: TOKEN_ID.to_string(),
        include_expired: None,
    };
    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, bidder.to_string());

    // Check balances, validate that the bidder was debited, and that the seller was credited
    let post_bidder_balance = router.wrap().query_all_balances(bidder.clone()).unwrap().into_iter().nth(0).unwrap();
    let post_owner_balance = router.wrap().query_all_balances(creator.clone()).unwrap().into_iter().nth(0).unwrap();
    assert_eq!(prev_bidder_balance.amount - Uint128::from(bid_amount), post_bidder_balance.amount);
    assert_eq!(Uint128::from(bid_amount), post_owner_balance.amount);
}

#[test]
fn try_auction_void() {
    let mut router = custom_mock_app();
    let block_time = router.block_info().time;
    // Setup intial accounts
    let (_owner, bidder, creator, bidder2) = setup_accounts(&mut router).unwrap();

    // Instantiate and configure contracts
    let (auction_english, collection) = setup_contracts(&mut router, &creator).unwrap();

    // Mint NFT for owner
    mint(&mut router, &creator, &collection, TOKEN_ID.to_string());
    approve(&mut router, &creator, &collection, &auction_english, TOKEN_ID.to_string());
    auction(
        &mut router,
        &creator,
        &auction_english,
        TOKEN_ID.to_string(),
        block_time.plus_seconds(ONE_DAY),
        block_time.plus_seconds(ONE_DAY * 2),
        110u128,
        210u128,
        None,
    );

    // Create an auction bid
    setup_block_time(&mut router, block_time.plus_seconds(ONE_DAY + TEN_MINS).seconds());
    auction_bid(&mut router, &bidder, &auction_english, TOKEN_ID.to_string(), 150u128);

    // Auction cannot be voided while Auction is still Open
    let query_auction = QueryMsg::Auction {
        token_id: TOKEN_ID.to_string()
    };
    let res: AuctionResponse = router
        .wrap()
        .query_wasm_smart(auction_english.clone(), &query_auction)
        .unwrap();
    assert_eq!(AuctionStatus::Open, res.auction_status.unwrap());

    let void_auction = ExecuteMsg::VoidAuction {
        token_id: TOKEN_ID.to_string(),
    };
    let res = router.execute_contract(bidder.clone(), auction_english.clone(), &void_auction, &[]);
    assert_eq!(&res.unwrap_err().root_cause().to_string(), "Auction invalid status: Open");

    // Auction cannot be voided while Auction is still Closed
    setup_block_time(&mut router, block_time.plus_seconds(ONE_DAY * 2 + TEN_MINS).seconds());
    let query_auction = QueryMsg::Auction {
        token_id: TOKEN_ID.to_string()
    };
    let res: AuctionResponse = router
        .wrap()
        .query_wasm_smart(auction_english.clone(), &query_auction)
        .unwrap();
    assert_eq!(AuctionStatus::Closed, res.auction_status.unwrap());

    let void_auction = ExecuteMsg::VoidAuction {
        token_id: TOKEN_ID.to_string(),
    };
    let res = router.execute_contract(bidder.clone(), auction_english.clone(), &void_auction, &[]);
    assert_eq!(&res.unwrap_err().root_cause().to_string(), "Auction invalid status: Closed");

    // Meet the reserve price
    setup_block_time(&mut router, block_time.plus_seconds(ONE_DAY + TEN_MINS).seconds());
    auction_bid(&mut router, &bidder, &auction_english, TOKEN_ID.to_string(), 240u128);

    // Auction cannot be voided if Auction reserve price is met
    setup_block_time(&mut router, block_time.plus_seconds(ONE_DAY * 3 + TEN_MINS).seconds());
    let query_auction = QueryMsg::Auction {
        token_id: TOKEN_ID.to_string()
    };
    let res: AuctionResponse = router
        .wrap()
        .query_wasm_smart(auction_english.clone(), &query_auction)
        .unwrap();
    assert_eq!(AuctionStatus::Expired, res.auction_status.unwrap());

    let void_auction = ExecuteMsg::VoidAuction {
        token_id: TOKEN_ID.to_string(),
    };
    let res = router.execute_contract(bidder.clone(), auction_english.clone(), &void_auction, &[]);
    assert_eq!(&res.unwrap_err().root_cause().to_string(), "Reserve price restriction: must finalize auction when reserve price is met");

    // Create a new Auction to test successful void auction messages
    let block_time = block_time.plus_seconds(ONE_DAY * 4);
    let token_id = "124".to_string();
    mint(&mut router, &creator, &collection, token_id.to_string());
    approve(&mut router, &creator, &collection, &auction_english, token_id.to_string());
    auction(
        &mut router,
        &creator,
        &auction_english,
        token_id.to_string(),
        block_time.plus_seconds(ONE_DAY),
        block_time.plus_seconds(ONE_DAY * 2),
        110u128,
        210u128,
        None,
    );

    // Create an auction bid
    setup_block_time(&mut router, block_time.plus_seconds(ONE_DAY + TEN_MINS).seconds());
    let prev_bidder_balance = router.wrap().query_all_balances(bidder.clone()).unwrap().into_iter().nth(0).unwrap();
    auction_bid(&mut router, &bidder, &auction_english, token_id.to_string(), 150u128);

    setup_block_time(&mut router, block_time.plus_seconds(ONE_DAY * 3 + TEN_MINS).seconds());

    // Auction can be voided by anyone, not just the bidder
    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: token_id.to_string(),
        include_expired: None,
    };
    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, auction_english.to_string());

    let void_auction = ExecuteMsg::VoidAuction {
        token_id: token_id.to_string(),
    };
    let res = router.execute_contract(bidder2.clone(), auction_english.clone(), &void_auction, &[]);
    assert!(res.is_ok());

    // Check NFT is transferred back to the original owner
    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: token_id.to_string(),
        include_expired: None,
    };
    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, creator.to_string());

    // Check balances, validate that the bidder was refunded
    let post_bidder_balance = router.wrap().query_all_balances(bidder.clone()).unwrap().into_iter().nth(0).unwrap();
    assert_eq!(prev_bidder_balance.amount, post_bidder_balance.amount);
}

#[test]
fn try_auction_queries() {
    let mut router = custom_mock_app();
    let block_time = router.block_info().time;
    // Setup intial accounts
    let (_owner, bidder, creator, bidder2) = setup_accounts(&mut router).unwrap();

    // Instantiate and configure contracts
    let (auction_english, collection) = setup_contracts(&mut router, &creator).unwrap();

    // Prep
    for idx in 1..5 {
        mint(&mut router, &creator, &collection, idx.to_string());
        approve(&mut router, &creator, &collection, &auction_english, idx.to_string());
        auction(
            &mut router,
            &creator,
            &auction_english,
            idx.to_string(),
            block_time.plus_seconds(ONE_DAY + idx as u64),
            block_time.plus_seconds(ONE_DAY * 2 + idx as u64),
            100u128 + idx as u128,
            200u128 + idx as u128,
            None,
        );
    }

    // Verify that auctions can be queried by token id
    let token_id = 2u64;
    let query_auction = QueryMsg::Auction {
        token_id: token_id.to_string()
    };
    let res: AuctionResponse = router
        .wrap()
        .query_wasm_smart(auction_english.clone(), &query_auction)
        .unwrap();
    assert_eq!(Auction {
        token_id: token_id.to_string(),
        seller: creator.clone(),
        start_time: block_time.plus_seconds(ONE_DAY + token_id),
        end_time: block_time.plus_seconds(ONE_DAY * 2 + token_id),
        starting_price: coin(100u128 + token_id as u128, NATIVE_DENOM),
        reserve_price: Some(coin(200u128 + token_id as u128, NATIVE_DENOM)),
        funds_recipient: None,
        highest_bid: None,
    }, res.auction.unwrap());
    assert_eq!(AuctionStatus::Pending, res.auction_status.unwrap());

    // Verify that auctions can be sorted by start time
    let query_auctions = QueryMsg::AuctionsByStartTime {
        query_options: QueryOptions {
            descending: Some(false),
            filter_expiry: None,
            start_after: None,
            limit: None,
        }
    };
    let res: AuctionsResponse = router
        .wrap()
        .query_wasm_smart(auction_english.clone(), &query_auctions)
        .unwrap();
    for n in 1..5 {
        assert_eq!(Auction {
            token_id: n.to_string(),
            seller: creator.clone(),
            start_time: block_time.plus_seconds(ONE_DAY + n),
            end_time: block_time.plus_seconds(ONE_DAY * 2 + n),
            starting_price: coin(100u128 + n as u128, NATIVE_DENOM),
            reserve_price: Some(coin(200u128 + n as u128, NATIVE_DENOM)),
            funds_recipient: None,
            highest_bid: None
        }, res.clone().auctions.into_iter().nth(n as usize - 1).unwrap());
    }

    // Verify that auctions can be sorted by end time
    let query_auctions = QueryMsg::AuctionsByEndTime {
        query_options: QueryOptions {
            descending: Some(true),
            filter_expiry: None,
            start_after: None,
            limit: None,
        }
    };
    let res: AuctionsResponse = router
        .wrap()
        .query_wasm_smart(auction_english.clone(), &query_auctions)
        .unwrap();
    for n in 4..0 {
        assert_eq!(Auction {
            token_id: token_id.to_string(),
            seller: creator.clone(),
            start_time: block_time.plus_seconds(ONE_DAY + n),
            end_time: block_time.plus_seconds(ONE_DAY * 2 + n),
            starting_price: coin(100u128 + n as u128, NATIVE_DENOM),
            reserve_price: Some(coin(200u128 + n as u128, NATIVE_DENOM)),
            funds_recipient: None,
            highest_bid: None
        }, res.clone().auctions.into_iter().nth(n as usize).unwrap());
    }

    // Verify that auctions can be sorted by highest bid price
    setup_block_time(&mut router, block_time.plus_seconds(ONE_DAY + 10u64).seconds());
    auction_bid(&mut router, &bidder, &auction_english, "1".to_string(), 140u128);
    auction_bid(&mut router, &bidder2, &auction_english, "3".to_string(), 250u128);
    let query_auctions = QueryMsg::AuctionsByHighestBidPrice {
        query_options: QueryOptions {
            descending: Some(true),
            filter_expiry: None,
            start_after: None,
            limit: Some(3),
        }
    };
    let res: AuctionsResponse = router
        .wrap()
        .query_wasm_smart(auction_english.clone(), &query_auctions)
        .unwrap();
    let n = 3;
    assert_eq!(Auction {
        token_id: n.to_string(),
        seller: creator.clone(),
        start_time: block_time.plus_seconds(ONE_DAY + n),
        end_time: block_time.plus_seconds(ONE_DAY * 2 + n),
        starting_price: coin(100u128 + n as u128, NATIVE_DENOM),
        reserve_price: Some(coin(200u128 + n as u128, NATIVE_DENOM)),
        funds_recipient: None,
        highest_bid: Some(AuctionBid { price: coin(250u128, "ujunox".to_string()), bidder: bidder2.clone() }),
    }, res.clone().auctions.into_iter().nth(0).unwrap());
    let n = 1;
    assert_eq!(Auction {
        token_id: n.to_string(),
        seller: creator.clone(),
        start_time: block_time.plus_seconds(ONE_DAY + n),
        end_time: block_time.plus_seconds(ONE_DAY * 2 + n),
        starting_price: coin(100u128 + n as u128, NATIVE_DENOM),
        reserve_price: Some(coin(200u128 + n as u128, NATIVE_DENOM)),
        funds_recipient: None,
        highest_bid: Some(AuctionBid { price: coin(140u128, "ujunox".to_string()), bidder: bidder.clone() }),
    }, res.clone().auctions.into_iter().nth(1).unwrap());
    let n = 4;
    assert_eq!(Auction {
        token_id: n.to_string(),
        seller: creator.clone(),
        start_time: block_time.plus_seconds(ONE_DAY + n),
        end_time: block_time.plus_seconds(ONE_DAY * 2 + n),
        starting_price: coin(100u128 + n as u128, NATIVE_DENOM),
        reserve_price: Some(coin(200u128 + n as u128, NATIVE_DENOM)),
        funds_recipient: None,
        highest_bid: None,
    }, res.clone().auctions.into_iter().nth(2).unwrap());

    // Verify that auctions can be queried by seller
    let query_auctions = QueryMsg::AuctionsBySellerEndTime {
        seller: creator.to_string(),
        query_options: QueryOptions {
            descending: None,
            filter_expiry: None,
            start_after: Some(TokenTimestampOffset {
                token_id: "1".to_string(),
                timestamp: block_time.plus_seconds(ONE_DAY * 2 + 1),
            }),
            limit: Some(2),
        }
    };
    let res: AuctionsResponse = router
        .wrap()
        .query_wasm_smart(auction_english.clone(), &query_auctions)
        .unwrap();
    for n in 2..4 {
        let highest_bid = match n {
            3 => Some(AuctionBid { price: coin(250u128, "ujunox".to_string()), bidder: bidder2.clone() }),
            _ => None,
        };
        assert_eq!(Auction {
            token_id: n.to_string(),
            seller: creator.clone(),
            start_time: block_time.plus_seconds(ONE_DAY + n),
            end_time: block_time.plus_seconds(ONE_DAY * 2 + n),
            starting_price: coin(100u128 + n as u128, NATIVE_DENOM),
            reserve_price: Some(coin(200u128 + n as u128, NATIVE_DENOM)),
            funds_recipient: None,
            highest_bid: highest_bid
        }, res.clone().auctions.into_iter().nth(n as usize - 2).unwrap());
    }


    // Verify that auctions can be queried by bidder
    let query_auctions = QueryMsg::AuctionsByBidderEndTime {
        bidder: bidder.to_string(),
        query_options: QueryOptions {
            descending: None,
            filter_expiry: None,
            start_after: None,
            limit: None,
        }
    };
    let res: AuctionsResponse = router
        .wrap()
        .query_wasm_smart(auction_english.clone(), &query_auctions)
        .unwrap();
    assert_eq!(res.auctions.len(), 1);
    let n = 1;
    assert_eq!(Auction {
        token_id: n.to_string(),
        seller: creator.clone(),
        start_time: block_time.plus_seconds(ONE_DAY + n),
        end_time: block_time.plus_seconds(ONE_DAY * 2 + n),
        starting_price: coin(100u128 + n as u128, NATIVE_DENOM),
        reserve_price: Some(coin(200u128 + n as u128, NATIVE_DENOM)),
        funds_recipient: None,
        highest_bid: Some(AuctionBid { price: coin(140u128, "ujunox".to_string()), bidder: bidder.clone() }),
    }, res.clone().auctions.into_iter().nth(0).unwrap());
}