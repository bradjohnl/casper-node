use num_traits::Zero;
use once_cell::sync::Lazy;

use casper_engine_test_support::{
    internal::{
        utils, ExecuteRequestBuilder, InMemoryWasmTestBuilder, StepRequestBuilder,
        DEFAULT_ACCOUNTS, DEFAULT_GENESIS_TIMESTAMP_MILLIS, DEFAULT_LOCKED_FUNDS_PERIOD_MILLIS,
        TIMESTAMP_MILLIS_INCREMENT,
    },
    AccountHash, DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE,
    MINIMUM_ACCOUNT_CREATION_BALANCE,
};
use casper_execution_engine::core::engine_state::{
    genesis::GenesisValidator, GenesisAccount, RewardItem,
};
use casper_types::{
    runtime_args,
    system::auction::{self, DelegationRate, BLOCK_REWARD, INITIAL_ERA_ID},
    Motes, ProtocolVersion, PublicKey, RuntimeArgs, SecretKey, U512,
};

const CONTRACT_TRANSFER_TO_ACCOUNT: &str = "transfer_to_account_u512.wasm";
const ARG_AMOUNT: &str = "amount";
const ARG_TARGET: &str = "target";

static DELEGATOR_1_SECRET_KEY: Lazy<SecretKey> =
    Lazy::new(|| SecretKey::ed25519_from_bytes([226; SecretKey::ED25519_LENGTH]).unwrap());
static VALIDATOR_1_SECRET_KEY: Lazy<SecretKey> =
    Lazy::new(|| SecretKey::ed25519_from_bytes([227; SecretKey::ED25519_LENGTH]).unwrap());

static VALIDATOR_1: Lazy<PublicKey> = Lazy::new(|| PublicKey::from(&*VALIDATOR_1_SECRET_KEY));
static DELEGATOR_1: Lazy<PublicKey> = Lazy::new(|| PublicKey::from(&*DELEGATOR_1_SECRET_KEY));
static DELEGATOR_1_ADDR: Lazy<AccountHash> = Lazy::new(|| AccountHash::from(&*DELEGATOR_1));

const VALIDATOR_STAKE: u64 = 1_000_000_000;
const DELEGATE_AMOUNT: u64 = 1_234_567;

#[ignore]
#[test]
fn should_run_ee_1152_regression_test() {
    let accounts = {
        let validator_1 = GenesisAccount::account(
            VALIDATOR_1.clone(),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Some(GenesisValidator::new(
                Motes::new(VALIDATOR_STAKE.into()),
                DelegationRate::zero(),
            )),
        );
        let validator_2 = GenesisAccount::account(
            DELEGATOR_1.clone(),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Some(GenesisValidator::new(
                Motes::new(VALIDATOR_STAKE.into()),
                DelegationRate::zero(),
            )),
        );

        let mut tmp: Vec<GenesisAccount> = DEFAULT_ACCOUNTS.clone();
        tmp.push(validator_1);
        tmp.push(validator_2);
        tmp
    };
    let run_genesis_request = utils::create_run_genesis_request(accounts);

    let fund_request_1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_TRANSFER_TO_ACCOUNT,
        runtime_args! {
            ARG_TARGET => PublicKey::System.to_account_hash(),
            ARG_AMOUNT => U512::from(MINIMUM_ACCOUNT_CREATION_BALANCE)
        },
    )
    .build();

    let fund_request_2 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_TRANSFER_TO_ACCOUNT,
        runtime_args! { ARG_TARGET => *DELEGATOR_1_ADDR, ARG_AMOUNT => U512::from(MINIMUM_ACCOUNT_CREATION_BALANCE) },
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::default();

    builder.run_genesis(&run_genesis_request);

    builder.exec(fund_request_1).commit().expect_success();
    builder.exec(fund_request_2).commit().expect_success();

    let auction_hash = builder.get_auction_contract_hash();

    let delegate_request_1 = ExecuteRequestBuilder::contract_call_by_hash(
        *DELEGATOR_1_ADDR,
        auction_hash,
        auction::METHOD_DELEGATE,
        runtime_args! {
            auction::ARG_DELEGATOR => DELEGATOR_1.clone(),
            auction::ARG_VALIDATOR => VALIDATOR_1.clone(),
            auction::ARG_AMOUNT => U512::from(DELEGATE_AMOUNT),
        },
    )
    .build();

    let undelegate_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DELEGATOR_1_ADDR,
        auction_hash,
        auction::METHOD_UNDELEGATE,
        runtime_args! {
            auction::ARG_DELEGATOR => DELEGATOR_1.clone(),
            auction::ARG_VALIDATOR => VALIDATOR_1.clone(),
            auction::ARG_AMOUNT => U512::from(DELEGATE_AMOUNT),
        },
    )
    .build();

    builder.exec(delegate_request_1).expect_success().commit();

    let mut timestamp_millis =
        DEFAULT_GENESIS_TIMESTAMP_MILLIS + DEFAULT_LOCKED_FUNDS_PERIOD_MILLIS;

    // In reality a step request is made, but to simplify the test I'm just calling the auction part
    // only.
    builder.run_auction(timestamp_millis, Vec::new());
    timestamp_millis += TIMESTAMP_MILLIS_INCREMENT;

    builder.run_auction(timestamp_millis, Vec::new());
    timestamp_millis += TIMESTAMP_MILLIS_INCREMENT;

    builder.run_auction(timestamp_millis, Vec::new()); // At this point paying out rewards would fail
    timestamp_millis += TIMESTAMP_MILLIS_INCREMENT;

    builder.run_auction(timestamp_millis, Vec::new());
    timestamp_millis += TIMESTAMP_MILLIS_INCREMENT;

    let era_validators = builder.get_era_validators();

    assert!(!era_validators.is_empty());

    let (era_id, trusted_era_validators) = era_validators
        .into_iter()
        .last()
        .expect("should have last element");
    assert!(era_id > INITIAL_ERA_ID, "{}", era_id);

    builder.exec(undelegate_request).expect_success().commit();

    let mut step_request = StepRequestBuilder::new()
        .with_parent_state_hash(builder.get_post_state_hash())
        .with_protocol_version(ProtocolVersion::V1_0_0)
        // Next era id is used for returning future era validators, which we don't need to inspect
        // in this test.
        .with_next_era_id(era_id);

    for (public_key, _stake) in trusted_era_validators.clone().into_iter() {
        let reward_amount = BLOCK_REWARD / trusted_era_validators.len() as u64;
        step_request = step_request.with_reward_item(RewardItem::new(public_key, reward_amount));
    }

    builder.step(step_request.build());

    builder.run_auction(timestamp_millis, Vec::new());
}
