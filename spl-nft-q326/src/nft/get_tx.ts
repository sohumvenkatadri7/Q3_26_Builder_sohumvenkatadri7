//PS: I forgot to convert the signature to base58 signature hence writing this extra script

import { base58 } from '@metaplex-foundation/umi/serializers';

const rawSig = new Uint8Array([
  8, 111, 157, 142,  73, 204, 112, 124, 116,  96, 80,
  230, 197, 231,  50,  95,  75, 156,   9,  13, 121, 76,
  202, 235, 123,   5,  78, 108, 242,  11, 152,  59, 34,
  144, 178, 151,  22, 110, 103, 108, 122,   6, 249, 91,
  183,  26,  88, 149,  68, 212, 147, 174,  68,  82, 97,
  57, 166, 115,  18,  80,   7,  55,  70,  11
]);

const txSignature = base58.deserialize(rawSig)[0];
console.log('Your Explorer link:');
console.log(`https://explorer.solana.com/tx/${txSignature}?cluster=devnet`);

