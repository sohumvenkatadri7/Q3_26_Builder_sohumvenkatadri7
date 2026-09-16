import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import wallet from "../../devnet-wallet.json";
import {
  createSignerFromKeypair,
  publicKey,
  signerIdentity,
} from "@metaplex-foundation/umi";
import { create, fetchAsset, mplCore, update } from "@metaplex-foundation/mpl-core";
import { base58 } from "@metaplex-foundation/umi/serializers";
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys";

const umi = createUmi(
  process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com",
);

const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(
  irysUploader({
    address: "https://devnet.irys.xyz/",
  }),
);

umi.use(signerIdentity(signer));

umi.use(mplCore());

(async () => {
  try {

    const assetAddress = publicKey("3UPbs9SRNYMwRMm2r5B4EFAJx1YseP2WUrVGoeqP5MFR");

    const asset = await fetchAsset(umi, assetAddress);

    const image = "https://gateway.irys.xyz/ChuxR64wx8qjnAQf18ZiUmvhFaxqdg4xrYZYCypes3vz";

    const newMetadata = {
      name: "Sohum Rug 2",
      description: "Generug NFT Updated",
      category: "image",
      image,
    }

    const myUri = await umi.uploader.uploadJson(newMetadata);

    console.log(`new metadata uri: ${myUri} `);

    const tx = await update(umi, {
      asset,
      name: "Updated NFT Name",
      uri: myUri,
    }).sendAndConfirm(umi);

    const signature = base58.deserialize(tx.signature)[0];

    console.log(`Successfully updated NFT!`);
    console.log(`signature ${signature} , asset : ${asset.publicKey}`);
  } catch (e) {
    console.log(`Error updating NFT: ${e}`);
  }
})();

//new metadata uri: https://gateway.irys.xyz/8EBNB6W7W64uTbKx31efXUjb5K6KZ2npVH6LMMRGShhL 
//signature 2jgzTJyg3jMPsmpng3t8VjZX13eRcjUqtCzNZjcqS9EFKUJd72dAZetzdjQTnSpEF7q5wdwYjWZqFdKkWiw1SLup
//asset : 3UPbs9SRNYMwRMm2r5B4EFAJx1YseP2WUrVGoeqP5MFR