import {
  appendTransactionMessageInstruction,
  appendTransactionMessageInstructions,
  assertIsTransactionMessageWithBlockhashLifetime,
  assertIsTransactionWithBlockhashLifetime,
  createKeyPairSignerFromBytes,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createTransactionMessage,
  generateKeyPairSigner,
  getSignatureFromTransaction,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
} from "@solana/kit";
import {
  getInitializeMintInstruction,
  getMintSize,
  TOKEN_PROGRAM_ADDRESS,
} from "@solana-program/token";
import { getCreateAccountInstruction } from "@solana-program/system";

//import your wallet
import wallet from "../../devnet-wallet.json";
import { assert } from "node:console";

const rpc = createSolanaRpc("https://api.devnet.solana.com");

const rpcSubscriptions = createSolanaRpcSubscriptions(
  "wss://api.devnet.solana.com",
);

(async () => {
  try { 
    //create a signer from your wallet
  const signer = await createKeyPairSignerFromBytes(new Uint8Array(wallet));

  //generate a new mint signer for address
  const mint = await generateKeyPairSigner();

  //get the size of the mint 
  const space = BigInt(getMintSize());

  //get the minimum balance for rent expemption
  const rent = await rpc.getMinimumBalanceForRentExemption(space).send();

  const {value: latestBlockhash} = await rpc.getLatestBlockhash().send();

  const sendAndConfirm = sendAndConfirmTransactionFactory({
    rpc,
    rpcSubscriptions,
  });

  const msg = createTransactionMessage({version: 0});

  const msgWithPayer = setTransactionMessageFeePayerSigner(signer, msg);

  const msgWithLifetime = setTransactionMessageLifetimeUsingBlockhash(
    latestBlockhash,
    msgWithPayer,
  );

  const txMessage = appendTransactionMessageInstructions(
    [
      getCreateAccountInstruction({
        payer: signer,
        newAccount : mint,
        lamports: rent,
        space,
        programAddress: TOKEN_PROGRAM_ADDRESS
      }),

      getInitializeMintInstruction({
        mint: mint.address,
        decimals: 6,
        mintAuthority: signer.address,
      }),
    ],
    msgWithLifetime,
  );

    //Sign the transaction 
    const signedTx = await signTransactionMessageWithSigners(txMessage);

    assertIsTransactionWithBlockhashLifetime(signedTx);

    //Send and confirm the transaction
    console.log("Sending transaction and waiting for confirmation...");
    await sendAndConfirm(signedTx, {
      commitment: "confirmed", 
    });

    const signature = getSignatureFromTransaction(signedTx);

    console.log(`mint address: ${mint.address}. Transaction Signature: ${signature}`);



  } catch (error) {
    console.log(error);
  }
})();


//mint address: 4xaTz8bEzFaYE1eXpaVS5R1GVAafZxiCXt6Phnu7Ka3r.
//Transaction Signature: 2PknoC661jNHZR9zQtCuVUY8R5iSxLCVbM1aroWpL9ZJCGUFCdJsbzxesD9VTXe6yXcBaKWAmmt1cZD1gwJkweY8