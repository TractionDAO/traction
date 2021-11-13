import { Percent } from "@saberhq/token-utils";
import { PublicKey } from "@solana/web3.js";

/**
 * Traction program addresses.
 */
export const TRACTION_ADDRESSES = {
  Traction: new PublicKey("TRXf3r361YRfV6Zktov3nvdEqJwAuCowkjh4PUUBYEc"),
};

/**
 * Owner of all protocol fee accounts.
 */
export const FEE_OWNER = new PublicKey(
  "2DDSpDyRbu9gZbcp2JCq2ZaA9FrCzXzoiyiGLyUFYSP5"
);

/**
 * Exercise fee. (1bp)
 */
export const EXERCISE_FEE = new Percent(1, 10_000);
