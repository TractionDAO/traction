import type { AnchorTypes } from "@saberhq/anchor-contrib";

import type { TractionIDL } from "../idls/traction";

export type TractionTypes = AnchorTypes<
  TractionIDL,
  {
    optionsContract: OptionsContractData;
  }
>;

type Accounts = TractionTypes["Accounts"];

/**
 * An American option.
 */
export type OptionsContractData = Accounts["OptionsContract"];

export type TractionError = TractionTypes["Error"];
export type TractionProgram = TractionTypes["Program"];

export type OptionWriteEvent = TractionTypes["Events"]["OptionWriteEvent"];
export type OptionRedeemEvent = TractionTypes["Events"]["OptionRedeemEvent"];
export type OptionExerciseEvent =
  TractionTypes["Events"]["OptionExerciseEvent"];
