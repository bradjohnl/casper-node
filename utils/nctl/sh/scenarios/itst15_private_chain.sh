#!/usr/bin/env bash

source "$NCTL"/sh/scenarios/common/itst.sh

# Exit if any of the commands fail.
set -e

#######################################
# TODO: Description here
#######################################
function generate_keys()
{
    echo "Installing casper-client"
    cargo install casper-client

    echo "Generating keys"
    casper-client keygen /tmp/bob
    casper-client keygen /tmp/alice

}

function assert_equals()
{
    if [ "$1" != "$2" ]; then
        echo "Assertion failed: $1 != $2"
        exit 1
    fi
}

function main() {
    log "------------------------------------------------------------"
    log "Starting Scenario: itst15_private_chain"
    log "------------------------------------------------------------"

    #0. Generate Alice and Bob keys:
    generate_keys()

    # 0. Fund alice
    cargo run --  \
    transfer \
    -n $(get_node_address_rpc) \
    --chain-name casper-net-1 \
    --secret-key /tmp/bob/secret_key.pem \
    --session-account=$(</tmp/bob/public_key_hex) \
    --target-account=$(</tmp/alice/public_key_hex) \
    --amount=100000000000 \
    --payment-amount=3000000000 \
    --transfer-id=100

    # 1. Disable Alice
    cargo run --  \
    put-deploy \
    -n $(get_node_address_rpc) \
    --chain-name casper-net-1 \
    --secret-key /tmp/bob/secret_key.pem \
    --session-account=$(</tmp/alice/public_key_hex) \
    --session-path $NCTL/resources/test/set_action_thresholds.wasm  \
    --payment-amount 3000000000 \
    --session-arg "key_management_threshold:u8='255'" \
    --session-arg "deploy_threshold:u8='255'"

    # 2. nctl-view-chain-account account-key=$(casper-client account-address --public-key alice/public_key.pem)
    nctl-view-chain-account account-key=$(casper-client account-address --public-key alice/public_key.pem) | jq -r '.action_thresholds.deployment' > /tmp/chain_account.json

    # 3. has to contain account_thresholds.deployment = 255 and account_thresholds.key_management = 255
    assert_equals "255" $(jq -r '.account_thresholds.deployment' /tmp/chain_account.json)
    assert_equals "255" $(jq -r '.account_thresholds.key_management' /tmp/chain_account.json)


    # Last. Run Health Checks
    # ... restarts=1: due to node being stopped and started
    source "$NCTL"/sh/scenarios/common/health_checks.sh \
            errors=0 \
            equivocators=0 \
            doppels=0 \
            crashes=0 \
            restarts=1 \
            ejections=0

    log "------------------------------------------------------------"
    log "Scenario itst15_private_chain complete"
    log "------------------------------------------------------------"
}

# ----------------------------------------------------------------
# ENTRY POINT
# ----------------------------------------------------------------

unset SYNC_TIMEOUT_SEC
unset LFB_HASH
unset PUBLIC_KEY_HEX
STEP=0

for ARGUMENT in "$@"; do
    KEY=$(echo "$ARGUMENT" | cut -f1 -d=)
    VALUE=$(echo "$ARGUMENT" | cut -f2 -d=)
    case "$KEY" in
        timeout) SYNC_TIMEOUT_SEC=${VALUE} ;;
        *) ;;
    esac
done

SYNC_TIMEOUT_SEC=${SYNC_TIMEOUT_SEC:-"300"}

main "$NODE_ID"
