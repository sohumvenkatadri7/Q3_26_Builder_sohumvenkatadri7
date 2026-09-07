import { createUmi } from '@metaplex-foundation/umi-bundle-defaults';
import { mplCore, transfer, fetchAsset } from '@metaplex-foundation/mpl-core';
import { publicKey, createSignerFromKeypair, signerIdentity } from '@metaplex-foundation/umi';
import wallet from '../../devnet-wallet.json';

const umi = createUmi('https://api.devnet.solana.com').use(mplCore());

const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);
umi.use(signerIdentity(signer)); 

const assetAddress = publicKey('3UPbs9SRNYMwRMm2r5B4EFAJx1YseP2WUrVGoeqP5MFR');
const newOwnerAddress = publicKey('3X3MD9BFHBnGwki3XQ9GLCJSAfrnSewzQTrf4MKrmg9V');

(async () => {
  // 1. Fetch the asset object first
  const asset = await fetchAsset(umi, assetAddress);

  // 2. Transfer using the fetched asset
  const result = await transfer(umi, {
    asset,
    newOwner: newOwnerAddress,
  }).sendAndConfirm(umi);

  console.log('Asset transferred! Tx signature:', result.signature);
})();