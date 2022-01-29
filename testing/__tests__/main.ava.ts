import { Workspace } from "near-workspaces-ava";

function intNEAR(x: number): number {
  return Math.round(x * 1e24);
}
function NEAR(x: number): string {
  return Math.round(x * 1000).toString() + "0".repeat(21);
}

const workspace = Workspace.init(async ({ root }) => {
  const someone = await root.createAccount("someone", {
    initialBalance: NEAR(5),
  });
  const sometwo = await root.createAccount("sometwo", {
    initialBalance: NEAR(5),
  });

  return { root, someone, sometwo };
});

workspace.test(
  "BuyMeACoffee migration works",
  async (test, { root, someone, sometwo }) => {
    // Deploy contract
    let contract = await root.createAndDeploy(
      "coffee",
      "../target/wasm32-unknown-unknown/release/buy_me_a_coffee_v1.wasm",
      { method: "initialize", args: { owner: root.accountId } }
    );
    const topCoffeeBuyer = async () => contract.view("top_coffee_buyer", {});

    // First coffee bought
    await someone.call(contract, "buy_coffee", {}, { attachedDeposit: NEAR(1) });
    test.is(
      await contract.view("coffee_near_from", { account: someone.accountId }),
      intNEAR(1)
    );
    test.deepEqual(await topCoffeeBuyer(), [someone.accountId, intNEAR(1)]);

    // Second coffee bought
    await sometwo.call(contract, "buy_coffee", {}, { attachedDeposit: NEAR(2) });
    test.is(
      await contract.view("coffee_near_from", { account: sometwo.accountId }),
      parseInt(NEAR(2))
    );
    test.deepEqual(await topCoffeeBuyer(), [sometwo.accountId, intNEAR(2)]);

    // Perform migration
    const tx = (
      await contract
        .createTransaction(contract)
        .deployContractFile(
          "../target/wasm32-unknown-unknown/release/buy_me_a_coffee_v2.wasm"
        )
    ).functionCall("migrate", {});
    await tx.signAndSend();

    // Existing state/methods are untouched
    test.is(
      await contract.view("coffee_near_from", { account: someone.accountId }),
      parseInt(NEAR(1))
    );
    test.is(
      await contract.view("coffee_near_from", { account: sometwo.accountId }),
      parseInt(NEAR(2))
    );
    test.deepEqual(await topCoffeeBuyer(), [sometwo.accountId, intNEAR(2)]);

    const topCoffeeBought = async () => contract.view("top_coffee_bought", {});
    test.deepEqual(await topCoffeeBought(), null);
  }
);
