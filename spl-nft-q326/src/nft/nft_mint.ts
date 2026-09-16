import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import wallet from "../../devnet-wallet.json";
import {
  createSignerFromKeypair,
  generateSigner,
  signerIdentity,
} from "@metaplex-foundation/umi";
import { create, mplCore } from "@metaplex-foundation/mpl-core";
import { base58 } from "@metaplex-foundation/umi/serializers";

const umi = createUmi(
  process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com",
);

const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(signerIdentity(signer));

umi.use(mplCore());

(async () => {
  try {
    const metadataUri =
      "https://gateway.irys.xyz/DQMA7LAWABboLnfBQNgimsKkEgxm54aHCA9hPNfpCNoC";
    const asset = generateSigner(umi);

    //add you nft name and metadata uri
    const tx = await create(umi, {
      asset,
      name: "SohumRug",
      uri: metadataUri,
    }).sendAndConfirm(umi)

    const signature = base58.deserialize(tx.signature)[0];

    console.log(`signature ${signature} , asset : ${asset.publicKey}`);
  } catch (e) {
    console.log(`errior ${e}`);
  }
})();

//signature 3JYHS8nfCr6UBPR5ifgcGDi3hAeM2Ryztq9oGAoEVidvCWmn71GDVuNyu2wcwGrFacmtVLbmuF62scMFLH7HptZ6
//asset : 3UPbs9SRNYMwRMm2r5B4EFAJx1YseP2WUrVGoeqP5MFR
