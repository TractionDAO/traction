import "chai-bn";

import { chaiSolana, expectTX } from "@saberhq/chai-solana";
import type { Provider } from "@saberhq/solana-contrib";
import { TransactionEnvelope } from "@saberhq/solana-contrib";
import {
  createMintAndVault,
  getATAAddress,
  getOrCreateATAs,
  getTokenAccount,
  Price,
  sleep,
  SPLToken,
  Token,
  TOKEN_PROGRAM_ID,
  TokenAmount,
  u64,
} from "@saberhq/token-utils";
import { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import BN from "bn.js";
import chai, { expect } from "chai";
import invariant from "tiny-invariant";

import type { TractionSDK } from "../src/traction";
import { dateToTimestamp } from "../src/utils";
import { makeSDK } from "./workspace";

chai.use(chaiSolana);

describe("Traction options", () => {
  let provider: Provider;
  let sdk: TractionSDK;

  beforeEach("Initialize", () => {
    sdk = makeSDK();
    provider = sdk.provider;
  });

  it("happy path", async () => {
    const { connection } = provider;

    const ownerKP = Keypair.generate();
    const adminKP = Keypair.generate();

    await connection.confirmTransaction(
      await connection.requestAirdrop(adminKP.publicKey, 10 * LAMPORTS_PER_SOL)
    );
    await connection.confirmTransaction(
      await connection.requestAirdrop(ownerKP.publicKey, 10 * LAMPORTS_PER_SOL)
    );

    const underlyingAmount = new u64(1_000_000 * LAMPORTS_PER_SOL);
    const quoteAmount = new u64(1_000_000 * 10 ** 6);

    const [underlyingMint, underlyingTokens] = await createMintAndVault(
      provider,
      underlyingAmount,
      undefined,
      9
    );
    const [quoteMint, quoteTokens] = await createMintAndVault(
      provider,
      quoteAmount,
      undefined,
      6
    );
    const underlying = Token.fromMint(underlyingMint, 9);
    const quote = Token.fromMint(quoteMint, 6);

    const ownerATAs = await getOrCreateATAs({
      provider,
      mints: {
        underlying: underlyingMint,
        quote: quoteMint,
      },
      owner: ownerKP.publicKey,
    });
    const txEnv = new TransactionEnvelope(provider, [
      ...ownerATAs.instructions,
      SPLToken.createTransferInstruction(
        TOKEN_PROGRAM_ID,
        underlyingTokens,
        ownerATAs.accounts.underlying,
        provider.wallet.publicKey,
        [],
        underlyingAmount
      ),
      SPLToken.createTransferInstruction(
        TOKEN_PROGRAM_ID,
        quoteTokens,
        ownerATAs.accounts.quote,
        provider.wallet.publicKey,
        [],
        quoteAmount
      ),
    ]);
    await expectTX(txEnv, "seed accounts").to.be.fulfilled;

    // strike of 1 SOL (underlying) = $100
    const strike = new Price(
      underlying,
      quote,
      LAMPORTS_PER_SOL,
      100 * 10 ** 6
    );
    const expiry = new Date(Date.now() + 5 * 1000); // 5 seconds to expiry
    const expiryTs = dateToTimestamp(expiry);

    const { tx } = await sdk.newContract({
      strike,
      expiryTs,
      direction: "call",
    });

    await expectTX(tx, "new contract").to.be.fulfilled;

    // load owner
    const ownerSDK = sdk.withSigner(ownerKP);
    const optionsContract = ownerSDK.loadContract({
      strike,
      expiryTs,
      direction: "call",
    });
    const optionToken = await optionsContract.generateToken();
    const writerToken = await optionsContract.fetchWriterToken();

    // ensure that we can load the contract by key
    const [optionsContractKey] = await optionsContract.findAddress();
    const loaded = await sdk.loadContractFromKey({ key: optionsContractKey });
    expect(loaded).to.not.be.null;
    invariant(loaded);
    expect(loaded.strike.equalTo(strike));
    expect(loaded.expiryTs).equal(expiryTs);

    const [loadedAddr] = await loaded.findAddress();
    expect(loadedAddr).to.eqAddress(optionsContractKey);

    const ownerWriterAccount = await getATAAddress({
      mint: writerToken.mintAccount,
      owner: ownerKP.publicKey,
    });
    const ownerOptionTokenAccount = await getATAAddress({
      mint: optionToken.mintAccount,
      owner: ownerKP.publicKey,
    });

    // write 1k of SOL options
    const writeAmount = new TokenAmount(optionToken, 1_000 * LAMPORTS_PER_SOL);
    const writeTX = await optionsContract.write({
      writeAmount,
    });
    await expectTX(writeTX, "write options").to.be.fulfilled;

    expect(
      (await getTokenAccount(provider, ownerATAs.accounts.underlying)).amount
    ).to.bignumber.eq(underlyingAmount.sub(writeAmount.toU64()));
    // have all my cash still
    expect(
      (await getTokenAccount(provider, ownerATAs.accounts.quote)).amount
    ).to.bignumber.eq(quoteAmount);
    expect(
      (await getTokenAccount(provider, ownerWriterAccount)).amount
    ).to.bignumber.eq(writeAmount.toU64());
    expect(
      (await getTokenAccount(provider, ownerOptionTokenAccount)).amount
    ).to.bignumber.eq(writeAmount.toU64());

    // exercise 1k of SOL options
    const exerciseTX = await optionsContract.exercise({
      optionAmount: new TokenAmount(optionToken, 1_000 * LAMPORTS_PER_SOL),
    });
    await expectTX(exerciseTX, "exercise options").to.be.fulfilled;

    // SOL should not be touched
    expect(
      (await getTokenAccount(provider, ownerATAs.accounts.underlying)).amount
    ).to.bignumber.eq(underlyingAmount);
    // cash account should have paid 1k * $100 for the SOL
    expect(
      (await getTokenAccount(provider, ownerATAs.accounts.quote)).amount
    ).to.bignumber.eq(quoteAmount.sub(new u64(1_000 * 100 * 10 ** 6)));

    // wait for expiry... is there a better way to do this?
    await sleep(5_000);
    const redeemTX = await optionsContract.redeem({
      writerAmount: new TokenAmount(writerToken, writeAmount.raw),
    });
    await expectTX(redeemTX, "redeem").to.be.fulfilled;

    // we should have the cash again, minus the 1bp fee
    expect(
      (await getTokenAccount(provider, ownerATAs.accounts.quote)).amount
    ).to.bignumber.eq(
      quoteAmount.sub(new u64(1_000 * 100 * 10 ** 6).div(new BN(10_000)))
    );
    // and we have all of our SOL back.
    expect(
      (await getTokenAccount(provider, ownerATAs.accounts.underlying)).amount
    ).to.bignumber.eq(underlyingAmount);
    expect(
      (await getTokenAccount(provider, ownerWriterAccount)).amount
    ).to.bignumber.eq(new u64(0));
  });
});
