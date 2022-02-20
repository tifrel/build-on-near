import { Workspace } from "near-workspaces-ava";

function intNEAR(x: number): number {
  return Math.round(x * 1e24);
}
function NEAR(x: number): string {
  return Math.round(x * 1000).toString() + "0".repeat(21);
}

const workspace = Workspace.init(async ({ root }) => {
  // Create a subaccounts
  const someone = await root.createAccount("someone", {
    initialBalance: NEAR(5),
  });
  const sometwo = await root.createAccount("sometwo", {
    initialBalance: NEAR(5),
  });

  // Deploy contract
  const contract = await root.createAndDeploy(
    // Subaccount name
    "coffee",

    // Relative path (from package.json location) to the compiled contract
    "../target/wasm32-unknown-unknown/release/near_buy_me_a_coffee.wasm",

    // Optional: specify initialization
    {
      method: "initialize",
      args: { owner: root.accountId },
    }
  );

  // Make things accessible in tests
  return { root, someone, sometwo, contract };
});

workspace.test(
  "BuyMeACoffee works",
  async (test, { contract, root, someone, sometwo }) => {
    const topCoffeeBuyer = async () => contract.view("top_coffee_buyer", {});

    // First coffee bought
    await someone.call(contract, "buy_coffee", {}, { attachedDeposit: NEAR(1) });
    test.log("Deposited!");
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
  }
);
