from invoke import task


@task
def build(c):
    c.run("dfx canister --no-wallet create --all")
    c.run("dfx build --all")
    print("\033[0;32;40m build completed\033[0m")


@task(build)
def install(c):
    c.run("dfx canister --no-wallet  install graphql")
    c.run("dfx canister --no-wallet  install issuanceTool")
    print("\033[0;32;40m install completed\033[0m")


@task(build)
def upgrade(c):
    c.run("dfx canister --no-wallet  install graphql --mode reinstall")
    c.run("dfx canister --no-wallet  install issuanceTool --mode reinstall")
    print("\033[0;32;40m upgrade completed\033[0m")


@task(upgrade, default=True)
def test_tool(c):
    print("\033[0;32;40m testing issue tool...\033[0m")
    tool_id = c.run("dfx canister id issuanceTool").stdout.replace("\n", "")
    c.run("dfx canister call issuanceTool initialize")
    c.run("dfx canister call graphql set_tool_canister_id '(principal \"" + tool_id + "\")'")
    issue_res = c.run(
        "dfx canister call issuanceTool issueToken '(record { logo = null ; name = \"Deland Token\" ; symbol = \"DLD\" ;decimals = 18 : nat8; total_supply = 100000000000000000000000000 : nat; fee = record { lowest = 1 : nat ;rate = 0 : nat ;};})'").stdout
    # (variant{Ok=record{canister_id=principal"qoctq-giaaa-aaaaa-aaaea-cai"}},)
    tid = issue_res.replace("\n", "").replace(" ", "").replace(
        "(variant{Ok=record{canister_id=principal\"", "").replace(
        "\"}},)", "")
    assert "\"name\":\"Deland Token\",\"symbol\":\"DLD\"" in c.run(
        "dfx canister call issuanceTool  graphql_query '(\"query { readTokenInfo {id,issuer,token_id,name,symbol,decimals,total_supply,fee_lowest,fee_rate,timestamp} }\", \"{}\")'").stdout
    print("\033[0;32;40m pass issue tool test\033[0m")

    print("\033[0;32;40m testing the new token...\033[0m")
    assert "Deland Token" in c.run("dfx canister call " + tid + " name").stdout
    assert "DLD" in c.run("dfx canister call " + tid + " symbol").stdout
    assert "18 : nat8" in c.run(
        "dfx canister call " + tid + " decimals").stdout
    assert "100_000_000_000_000_000_000_000_000 : nat" in c.run(
        "dfx canister call " + tid + " totalSupply").stdout
    assert "1 : nat" in c.run("dfx canister call " + tid + " fee").stdout
    assert "Deland Token" in c.run("dfx canister call " + tid + " meta").stdout
    print("\033[0;32;40m pass the new token test\033[0m")
