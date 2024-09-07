import { BN, utils } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

export const LOCATION_DECIMALS = 18;
export const PROJECT_ID = "73801621aec60dfaa2197c7640c15858";
export const DEBUG = true;

export const TIME_TILL_LOCK = 15 * 60 * 1000; // mss

export const USER_TAG = "USER_STATE";

export const ADMIN_TAG = "ADMIN_TAG";

export const STORE_TAG = "STORE_STATE";

export const REQUEST_TAG = "REQUEST_STATE";

export const OFFER_TAG = "OFFER_STATE";

export const USER_COUNTER = "USER_COUNTER";

export const STORE_COUNTER = "STORE_COUNTER";

export const REQUEST_COUNTER = "REQUEST_COUNTER";

export const OFFER_COUNTER = "OFFER_COUNTER";

export const ntobs58 = (x: any) =>
  utils.bytes.bs58.encode(new BN(x).toArrayLike(Buffer, "le", 8));
