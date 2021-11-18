from invoke import task


@task
def build(c):
    c.run("dfx canister --no-wallet create --all")
    c.run("dfx build --all")
    print("\033[0;32;40m build completed\033[0m")


@task(build)
def install(c):
    c.run("dfx canister --no-wallet  install issuanceTool")
    print("\033[0;32;40m install completed\033[0m")


@task(build)
def upgrade(c):
    c.run("dfx canister --no-wallet  install issuanceTool --mode reinstall")
    print("\033[0;32;40m upgrade completed\033[0m")


@task(upgrade, default=True)
def test_tool(c):
    print("\033[0;32;40m testing issue tool...\033[0m")
    owner = c.run("dfx identity  get-principal").stdout.replace("\n", "")
    tool_id = c.run("dfx canister --no-wallet id issuanceTool").stdout.replace("\n", "")
    token_id = c.run("dfx canister --no-wallet id empty").stdout.replace("\n", "")
    # set empty controller

    print("\033[0;32;40m update controller...\033[0m")
    c.run("dfx canister --no-wallet update-settings empty  --controller " + tool_id)
    c.run("dfx canister  --no-wallet  call issuanceTool setOwner '(principal \"" + owner + "\")' ")

    print("\033[0;32;40m upload wasm...\033[0m")
    c.run("ic-repl --replica local upload_wasm.sh")
    print("\033[0;32;40m install token...\033[0m")
    issue_res = c.run(
        "dfx canister  --no-wallet  call issuanceTool issueToken '(record { canister_id = principal \""
        + token_id +
        "\";  sub_account = null ; logo = null ; name = \"Deland Token\" ; symbol = \"DLD\" ;decimals = 18 : nat8; total_supply = 100000000000000000000000000 : nat; fee = record { minimum = 1 : nat ;rate = 0 : nat ;};})'").stdout
    # (variant{Ok=record{canister_id=principal"qoctq-giaaa-aaaaa-aaaea-cai"}},)
    tid = issue_res.replace("\n", "").replace(" ", "").replace(
        "(variant{Ok=record{canister_id=principal\"", "").replace(
        "\"}},)", "")
    assert "symbol = \"DLD\"" in c.run(
        "dfx canister  --no-wallet  call issuanceTool  tokenOf '(principal \"" + tid + "\")'").stdout
    print("\033[0;32;40m pass issue tool test\033[0m")

    print("\033[0;32;40m testing the new token...\033[0m")
    assert "Deland Token" in c.run("dfx canister call " + tid + " name").stdout
    assert "DLD" in c.run("dfx canister call " + tid + " symbol").stdout
    assert "18 : nat8" in c.run(
        "dfx canister  --no-wallet  call " + tid + " decimals").stdout
    assert "100_000_000_000_000_000_000_000_000 : nat" in c.run(
        "dfx canister call " + tid + " totalSupply").stdout
    assert "1 : nat" in c.run(
        "dfx canister  --no-wallet  call " + tid + " fee").stdout
    assert "Deland Token" in c.run(
        "dfx canister  --no-wallet  call " + tid + " meta").stdout
    print("\033[0;32;40m pass the new token test\033[0m")
