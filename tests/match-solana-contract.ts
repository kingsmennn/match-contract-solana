import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Marketplace } from "../target/types/marketplace";
import { USER_COUNTER_PUBKEY, USER_TAG } from "../utils";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { utf8 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";

describe("match-solana-contract", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Marketplace as Program<Marketplace>;

  it("Can create a new user and return true values", async () => {
    const [profilePda, _] = PublicKey.findProgramAddressSync(
      [utf8.encode(USER_TAG), wallet!.value!.publicKey!.toBuffer()],
      programID
    );

    const payload = {
      username: "test",
      phone: "1234567890",
      latitude: 0,
      longitude: 0,
      account_type: "buyer",
    };

    const tx = await program.methods
      .createUser(
        payload.username,
        payload.phone,
        payload.latitude,
        payload.longitude,
        {
          [payload.account_type]: {},
        }
      )
      .accounts({
        user: profilePda,
        systemProgram: SystemProgram.programId,
        userCounter: USER_COUNTER_PUBKEY,
        authority: wallet!.value!.publicKey!,
      })
      .rpc();
  });
});
