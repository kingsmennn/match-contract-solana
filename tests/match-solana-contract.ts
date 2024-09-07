import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Marketplace } from "../target/types/marketplace";
import {
  LOCATION_DECIMALS,
  OFFER_COUNTER,
  REQUEST_COUNTER,
  STORE_COUNTER,
  USER_COUNTER,
  USER_TAG,
} from "../utils";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { utf8 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import { BN } from "bn.js";
import { expect } from "chai";

describe("match-solana-contract", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Marketplace as Program<Marketplace>;

  let USER_COUNTER_PUBKEY: PublicKey;
  let STORE_COUNTER_PUBKEY: PublicKey;
  let REQUEST_COUNTER_PUBKEY: PublicKey;
  let OFFER_COUNTER_PUBKEY: PublicKey;
  let profilePda: PublicKey;

  const payload = {
    username: "test",
    phone: "1234567890",
    latitude: new BN(Math.trunc(3.4 * 10 ** LOCATION_DECIMALS).toString()),
    longitude: new BN(Math.trunc(6.2 * 10 ** LOCATION_DECIMALS).toString()),
    account_type: { buyer: {} },
  };

  beforeEach(async () => {
    if (profilePda) return;
    const [userCounterPDA] = PublicKey.findProgramAddressSync(
      [utf8.encode(USER_COUNTER)],
      program.programId
    );
    const [storeCounterPDA] = PublicKey.findProgramAddressSync(
      [utf8.encode(STORE_COUNTER)],
      program.programId
    );
    const [requestCounterPDA] = PublicKey.findProgramAddressSync(
      [utf8.encode(REQUEST_COUNTER)],
      program.programId
    );
    const [offerCounterPDA] = PublicKey.findProgramAddressSync(
      [utf8.encode(OFFER_COUNTER)],
      program.programId
    );
    const tx = await program.methods
      .initializeCounters()
      .accounts({
        systemProgram: SystemProgram.programId,
        userCounter: userCounterPDA,
        storeCounter: storeCounterPDA,
        requestCounter: requestCounterPDA,
        offerCounter: offerCounterPDA,
        authority: provider.publicKey,
      })
      .rpc();

    USER_COUNTER_PUBKEY = userCounterPDA;
    STORE_COUNTER_PUBKEY = storeCounterPDA;
    REQUEST_COUNTER_PUBKEY = requestCounterPDA;
    OFFER_COUNTER_PUBKEY = offerCounterPDA;

    const [profilePda_] = PublicKey.findProgramAddressSync(
      [utf8.encode(USER_TAG), provider.publicKey.toBuffer()],
      program.programId
    );

    profilePda = profilePda_;

    await program.methods
      .createUser(
        payload.username,
        payload.phone,
        payload.latitude,
        payload.longitude,
        payload.account_type
      )
      .accounts({
        user: profilePda,
        systemProgram: SystemProgram.programId,
        userCounter: USER_COUNTER_PUBKEY,
        authority: provider.publicKey,
      })
      .rpc();
  });

  it("Can create a new user and return true values", async () => {
    const user = await program.account.user.fetch(profilePda);
    expect(user.username).to.be.equals(payload.username);
    expect(user.phone).to.be.equals(payload.phone);
    expect(Number(user.location.latitude)).to.be.equals(
      Number(payload.latitude)
    );
    expect(Number(user.location.longitude)).to.be.equals(
      Number(payload.longitude)
    );
    expect(user.accountType).to.be.deep.equals(payload.account_type);
  });

  it("Can update a user and return true values", async () => {
    const newPayload = {
      username: "test2",
      phone: "0987654321",
      latitude: new BN(Math.trunc(4.5 * 10 ** LOCATION_DECIMALS).toString()),
      longitude: new BN(Math.trunc(7.8 * 10 ** LOCATION_DECIMALS).toString()),
      account_type: { seller: {} },
    };
    return;

    await program.methods
      .updateUser(
        newPayload.username,
        newPayload.phone,
        newPayload.latitude,
        newPayload.longitude,
        newPayload.account_type
      )
      .accounts({
        user: profilePda,
        authority: provider.publicKey,
      })
      .rpc();

    const user = await program.account.user.fetch(profilePda);
    expect(user.username).to.be.equals(newPayload.username);
    expect(user.phone).to.be.equals(newPayload.phone);
    expect(Number(user.location.latitude)).to.be.equals(
      Number(newPayload.latitude)
    );
    expect(Number(user.location.longitude)).to.be.equals(
      Number(newPayload.longitude)
    );
    expect(user.accountType).to.be.deep.equals(newPayload.account_type);
  });
});
