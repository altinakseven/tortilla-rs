import path from "path";
import { deployContract } from "@/libs/utils";
import { AlkaneId, BaseTokenContract } from "tacoclicker-sdk";
import { provider } from "@/consts";
import { walletSigner } from "@/crypto/wallet";
import { taskLogger as logger } from "@/consts";
import { consumeAll } from "@/boxed";

const readableAlkaneId = (id: AlkaneId) =>
  `(block→${Number(id.block)}n : tx→${Number(id.tx)}n)`;

/** Deploy, initialise and mint a Free-Mint token contract. */
export const runFreeMint = async (enableDeploy?: boolean): Promise<boolean> => {
  const root = logger.start("deploy & inspect free-mint token");
  enableDeploy = enableDeploy ?? true;

  try {
    const freeMintId = enableDeploy
      ? await deployContract(
          path.join(__dirname, "../..", "./contracts/free-mint"),
          [100n] // view-method quirk
        )
      : ({
          block: 2n,
          tx: 75n,
        } as AlkaneId);

    logger.success(`contract at ${readableAlkaneId(freeMintId)}`);

    const tokenContract = new BaseTokenContract(
      provider,
      freeMintId,
      walletSigner.signPsbt
    );

    await logger.progressExecute(
      "initialize",
      tokenContract.initialize(walletSigner.address, {
        name: "test",
        symbol: "TEST",
        valuePerMint: 10n,
        cap: 1000n,
        premine: 10_000n,
      })
    );

    await logger.progressExecute(
      "mint",
      tokenContract.mintTokens(walletSigner.address)
    );

    await logger.info("Getting contract state…");

    let tokenContractReturnValues = consumeAll(
      await Promise.all([
        tokenContract.viewGetName(),
        tokenContract.viewGetSymbol(),
        tokenContract.viewGetTotalSupply(),
        tokenContract.viewGetValuePerMint(),
        tokenContract.viewGetMinted(),
        tokenContract.viewGetCap(),
        tokenContract.viewGetBalance(walletSigner.address),
      ] as const)
    );

    logger.info("Asserting contract state...");
    logger.deepAssert(
      [
        "test",
        "TEST",
        10_010, // totalSupply + minted
        10, // valuePerMint
        1, // minted
        1000, // cap
        10_010, //balance (premine + mint)
      ],
      tokenContractReturnValues
    );
    logger.success("All asserts passed. Contract state asserted successfully.");

    const [name, symbol, totalSupply, valuePerMint, minted, cap] =
      tokenContractReturnValues;

    logger.info(`name:          ${name}`);
    logger.info(`symbol:        ${symbol}`);
    logger.info(`totalSupply:   ${totalSupply}`);
    logger.info(`valuePerMint:  ${valuePerMint}`);
    logger.info(`minted:        ${minted}`);
    logger.info(`cap:           ${cap}`);
    root.close();
    return true;
  } catch (err) {
    logger.error(err as Error);
    root.close();
    process.exit(1);
  }
};
