import {Connection, Keypair, PublicKey, sendAndConfirmTransaction, SystemProgram, Transaction} from "@solana/web3.js";
import * as anchor from "@project-serum/anchor";
import {
  createInitializeMint2Instruction,
  getMinimumBalanceForRentExemptMint,
  MINT_SIZE,
  TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import {Program} from "@project-serum/anchor";

export const DOMAIN = 'domination';
export const KEYCHAIN = 'keychain';

export const DOMAIN_STATE = 'domain_state';

export const KEYCHAIN_SPACE = 'keychains';
export const KEYCHAIN_STATE_SPACE = 'keychain_states';
export const KEY_SPACE = 'keys';

export const STACHE = 'stache';
export const BEARD_SPACE = 'beards';


// taken from the keychain project

export async function createTokenMint(connection: Connection, payer: Keypair, authority: PublicKey): Promise<Keypair> {

  const lamports = await getMinimumBalanceForRentExemptMint(connection);
  const mintKey = anchor.web3.Keypair.generate();

  const transaction = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: mintKey.publicKey,
        space: MINT_SIZE,
        lamports,
        programId: TOKEN_PROGRAM_ID,
      }),
      createInitializeMint2Instruction(mintKey.publicKey, 9, authority, authority, TOKEN_PROGRAM_ID),
  );

  await sendAndConfirmTransaction(connection, transaction, [payer, mintKey]);
  return mintKey;
}

export async function createNFTMint(connection: Connection, payer: Keypair, authority: PublicKey): Promise<Keypair> {

  const lamports = await getMinimumBalanceForRentExemptMint(connection);
  const mintKey = anchor.web3.Keypair.generate();

  const transaction = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: mintKey.publicKey,
        space: MINT_SIZE,
        lamports,
        programId: TOKEN_PROGRAM_ID,
      }),
      createInitializeMint2Instruction(mintKey.publicKey, 0, authority, authority, TOKEN_PROGRAM_ID),
  );

  await sendAndConfirmTransaction(connection, transaction, [payer, mintKey]);
  return mintKey;
}

export const findDomainPda = (domain: string, keychainprogid: PublicKey): [PublicKey, number] => {
  return anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from(anchor.utils.bytes.utf8.encode(domain)),
        Buffer.from(anchor.utils.bytes.utf8.encode(KEYCHAIN))],
      keychainprogid
  );
}

export const findDomainStatePda = (domain: string, keychainprogid: PublicKey): [PublicKey, number] => {
  return anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode(DOMAIN_STATE)),
        Buffer.from(anchor.utils.bytes.utf8.encode(domain)),
        Buffer.from(anchor.utils.bytes.utf8.encode(KEYCHAIN))],
      keychainprogid
  );
}

// finds the keychain pda for the given playername (for the domination domain)
export const findKeychainPda = (name: string, domain: string, keychainprogid: PublicKey): [PublicKey, number] => {
  return anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode(name)),
        Buffer.from(anchor.utils.bytes.utf8.encode(KEYCHAIN_SPACE)),
        Buffer.from(anchor.utils.bytes.utf8.encode(domain)),
        Buffer.from(anchor.utils.bytes.utf8.encode(KEYCHAIN)),
      ],
      keychainprogid,
  );
};

export const findKeychainStatePda = (keychainPda: PublicKey, domain: string, keychainprogid: PublicKey): [PublicKey, number] => {
  return anchor.web3.PublicKey.findProgramAddressSync(
      [
        keychainPda.toBuffer(),
        Buffer.from(anchor.utils.bytes.utf8.encode(KEYCHAIN_STATE_SPACE)),
        Buffer.from(anchor.utils.bytes.utf8.encode(domain)),
        Buffer.from(anchor.utils.bytes.utf8.encode(KEYCHAIN)),
      ],
      keychainprogid,
  );
};

// find the keychain KEY pda for the given wallet address (for the domination domain)
export const findKeychainKeyPda = (walletAddress: PublicKey, domain: string, keychainprogid: PublicKey): [PublicKey, number] => {
  // const [keychainPda, keychainPdaBump] = anchor.web3.PublicKey.findProgramAddressSync(
  return anchor.web3.PublicKey.findProgramAddressSync(
      [
        walletAddress.toBuffer(),
        Buffer.from(anchor.utils.bytes.utf8.encode(KEY_SPACE)),
        Buffer.from(anchor.utils.bytes.utf8.encode(domain)),
        Buffer.from(anchor.utils.bytes.utf8.encode(KEYCHAIN)),
      ],
      keychainprogid,
  );
};


///// stache pda finders
export const findStachePda = (name: string, domainPda: PublicKey, stacheprogid: PublicKey): [PublicKey, number] => {
  return anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode(name)),
        Buffer.from(anchor.utils.bytes.utf8.encode(BEARD_SPACE)),
        domainPda.toBuffer(),
        Buffer.from(anchor.utils.bytes.utf8.encode(STACHE)),
      ],
      stacheprogid,
  );
};
