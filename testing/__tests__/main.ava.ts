import { Workspace } from "near-workspaces-ava";

function NEAR(x: number): string {
  return Math.round(x * 1000).toString() + "0".repeat(21);
}

const workspace = Workspace.init(async ({ root }) => {
  const outer = await root.createAndDeploy(
    "outer",
    "../target/wasm32-unknown-unknown/release/xcc_outer.wasm"
  );

  const innerA = await root.createAndDeploy(
    "inner-a", // we cannot use uppercase letters in NEAR account names
    "../target/wasm32-unknown-unknown/release/xcc_inner.wasm"
  );

  const innerB = await root.createAndDeploy(
    "inner-b", // we cannot use uppercase letters in NEAR account names
    "../target/wasm32-unknown-unknown/release/xcc_inner.wasm"
  );

  return { root, outer, innerA, innerB };
});

workspace.test(
  "Cross-contract calls work",
  async (test, { root, outer, innerA, innerB }) => {
    const xccA = await root.call_raw(
      outer,
      "dispatch_call",
      {
        account: innerA.accountId,
        method: "deposit",
        args: JSON.stringify({ msg: "Call to A" }),
      },
      { attachedDeposit: NEAR(1) }
    );
    test.deepEqual(xccA.logs, [
      `Dispatching call: ${innerA.accountId}.deposit({"msg":"Call to A"})`,
      `${innerA.accountId}: Call to A (deposit: ${NEAR(1)})`,
    ]);
    test.is(xccA.parseResult<string>(), NEAR(1));

    const xccB = await root.call_raw(
      outer,
      "dispatch_call",
      {
        account: innerB.accountId,
        method: "deposit",
        args: JSON.stringify({ msg: "Call to B" }),
      },
      { attachedDeposit: NEAR(2) }
    );
    test.deepEqual(xccB.logs, [
      `Dispatching call: ${innerB.accountId}.deposit({"msg":"Call to B"})`,
      `${innerB.accountId}: Call to B (deposit: ${NEAR(2)})`,
    ]);
    test.is(xccB.parseResult<string>(), NEAR(2));

    test.is(await innerA.view("get_deposited"), NEAR(1));
    test.is(await innerB.view("get_deposited"), NEAR(2));
    test.is(await outer.view("get_dispatched_calls"), 2);
  }
);
