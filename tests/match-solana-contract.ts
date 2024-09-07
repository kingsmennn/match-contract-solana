import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Marketplace } from "../target/types/marketplace";
import {
  LOCATION_DECIMALS,
  OFFER_COUNTER,
  REQUEST_COUNTER,
  REQUEST_TAG,
  STORE_COUNTER,
  STORE_TAG,
  USER_COUNTER,
  USER_TAG,
} from "../utils";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { utf8 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import { BN } from "bn.js";
import { expect } from "chai";

describe("match-solana-contract", function () {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Marketplace as Program<Marketplace>;

  let USER_COUNTER_PUBKEY: PublicKey;
  let STORE_COUNTER_PUBKEY: PublicKey;
  let REQUEST_COUNTER_PUBKEY: PublicKey;
  let OFFER_COUNTER_PUBKEY: PublicKey;
  let profilePda: PublicKey;
  let sellerPda: PublicKey;

  const buyerPayload = {
    username: "test",
    phone: "1234567890",
    latitude: new BN(Math.trunc(3.4 * 10 ** LOCATION_DECIMALS).toString()),
    longitude: new BN(Math.trunc(6.2 * 10 ** LOCATION_DECIMALS).toString()),
    account_type: { buyer: {} },
  };

  const sellerPayload = {
    username: "test2",
    phone: "0987654321",
    latitude: new BN(Math.trunc(4.5 * 10 ** LOCATION_DECIMALS).toString()),
    longitude: new BN(Math.trunc(7.8 * 10 ** LOCATION_DECIMALS).toString()),
    account_type: { seller: {} },
  };

  const seller = anchor.web3.Keypair.generate();

  beforeEach(async function () {
    if (profilePda) return;
    await provider.connection.requestAirdrop(
      seller.publicKey,
      anchor.web3.LAMPORTS_PER_SOL * 1
    );
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
    await program.methods
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

    const [sellerPda_] = PublicKey.findProgramAddressSync(
      [utf8.encode(USER_TAG), seller.publicKey.toBuffer()],
      program.programId
    );

    profilePda = profilePda_;
    sellerPda = sellerPda_;

    await program.methods
      .createUser(
        buyerPayload.username,
        buyerPayload.phone,
        buyerPayload.latitude,
        buyerPayload.longitude,
        buyerPayload.account_type
      )
      .accounts({
        user: profilePda,
        systemProgram: SystemProgram.programId,
        userCounter: USER_COUNTER_PUBKEY,
        authority: provider.publicKey,
      })

      .rpc();

    await program.methods
      .createUser(
        sellerPayload.username,
        sellerPayload.phone,
        sellerPayload.latitude,
        sellerPayload.longitude,
        sellerPayload.account_type
      )
      .accounts({
        user: sellerPda,
        systemProgram: SystemProgram.programId,
        userCounter: USER_COUNTER_PUBKEY,
        authority: seller.publicKey,
      })
      .signers([seller])
      .rpc();
  });

  it("Can create a new user and return true values", async function () {
    const user = await program.account.user.fetch(profilePda);
    expect(user.username).to.be.equal(buyerPayload.username);
    expect(user.phone).to.be.equal(buyerPayload.phone);
    expect(Number(user.location.latitude)).to.be.equal(
      Number(buyerPayload.latitude)
    );
    expect(Number(user.location.longitude)).to.be.equal(
      Number(buyerPayload.longitude)
    );
    expect(user.accountType).to.be.deep.equal(buyerPayload.account_type);
  });

  it("Can update a user and return true values", async function () {
    await program.methods
      .updateUser(
        sellerPayload.username,
        sellerPayload.phone,
        sellerPayload.latitude,
        sellerPayload.longitude,
        sellerPayload.account_type
      )
      .accounts({
        user: profilePda,
        authority: provider.publicKey,
      })
      .rpc();

    const user = await program.account.user.fetch(profilePda);
    expect(user.username).to.be.equal(sellerPayload.username);
    expect(user.phone).to.be.equal(sellerPayload.phone);
    expect(Number(user.location.latitude)).to.be.equal(
      Number(sellerPayload.latitude)
    );
    expect(Number(user.location.longitude)).to.be.equal(
      Number(sellerPayload.longitude)
    );
    expect(user.accountType).to.be.deep.equal(sellerPayload.account_type);
  });

  it("Should allow a user to create a store", async function () {
    const payload = {
      name: "test store",
      description: "test description",
      phone: "1234567890",
      long: Math.trunc(3.38 * 10 ** LOCATION_DECIMALS),
      lat: Math.trunc(4.383 * 10 ** LOCATION_DECIMALS),
    };

    const [profilePda, _] = PublicKey.findProgramAddressSync(
      [utf8.encode(USER_TAG), provider.publicKey.toBuffer()],
      program.programId
    );

    const storeCounter = await program.account.counter.fetch(
      STORE_COUNTER_PUBKEY
    );

    const [storePda] = PublicKey.findProgramAddressSync(
      [
        utf8.encode(STORE_TAG),
        provider.publicKey.toBuffer(),
        Buffer.from(storeCounter.current.toArray("le", 8)),
      ],
      program.programId
    );

    const receipt = await program.methods
      .createStore(
        payload.name,
        payload.description,
        payload.phone,
        new BN(payload.lat.toString()),
        new BN(payload.long.toString())
      )
      .accounts({
        user: profilePda,
        systemProgram: SystemProgram.programId,
        storeCounter: STORE_COUNTER_PUBKEY,
        authority: provider.publicKey,
        store: storePda,
      })
      .rpc();

    const store = await program.account.store.fetch(storePda);
    expect(store.name).to.be.equal(payload.name);
    expect(store.description).to.be.equal(payload.description);
    expect(store.phone).to.be.equal(payload.phone);
    expect(Number(store.location.latitude)).to.be.equal(payload.lat);
  });

  it("Should allow a user to create a request", async function () {
    const payload = {
      name: "test store",
      description: "test description",
      phone: "1234567890",
      long: Math.trunc(3.38 * 10 ** LOCATION_DECIMALS),
      lat: Math.trunc(4.383 * 10 ** LOCATION_DECIMALS),
      images: ["image1", "image2"],
    };
    const [profilePda, _] = PublicKey.findProgramAddressSync(
      [utf8.encode(USER_TAG), provider.publicKey.toBuffer()],
      program.programId
    );

    const requestCounter = await program.account.counter.fetch(
      REQUEST_COUNTER_PUBKEY
    );

    const [requestPda] = PublicKey.findProgramAddressSync(
      [
        utf8.encode(REQUEST_TAG),
        provider.publicKey.toBuffer(),
        Buffer.from(requestCounter.current.toArray("le", 8)),
      ],
      program.programId
    );

    const receipt = await program.methods
      .createRequest(
        payload.name,
        payload.description,
        payload.images,
        new BN(payload.lat.toString()),
        new BN(payload.long.toString())
      )
      .accounts({
        user: profilePda,
        systemProgram: SystemProgram.programId,
        requestCounter: REQUEST_COUNTER_PUBKEY,
        authority: provider.publicKey,
        request: requestPda,
      })
      .rpc();
  });
});
