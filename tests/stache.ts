import * as anchor from "@project-serum/anchor";
import {Idl, Program, web3} from "@project-serum/anchor";
import { Stache } from "../target/types/stache";

import { execSync } from "child_process";

import kcidl from "../idl/keychain.json";

import {Keypair, PublicKey, sendAndConfirmTransaction, SystemProgram, Transaction} from "@solana/web3.js";
import {
  createNFTMint, createTokenMint,
  findStachePda,
  findDomainPda,
  findDomainStatePda,
  findKeychainKeyPda,
  findKeychainPda,
  findKeychainStatePda, findVaultPda
} from "./utils";
import * as assert from "assert";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccount,
  createAssociatedTokenAccountInstruction,
  createMint,
  createMintToCheckedInstruction,
  createTransferCheckedInstruction,
  createTransferCheckedWithFeeInstruction, createTransferInstruction,
  getAccount,
  getAssociatedTokenAddress,
  getAssociatedTokenAddressSync,
  mintToChecked, TOKEN_PROGRAM_ID,
  transferChecked
} from "@solana/spl-token";
import {expect} from "chai";

const KeychainIdl = kcidl as Idl;

const KeychainProgId: PublicKey =  new PublicKey(KeychainIdl.metadata.address);


///// this test is set up to run against a local validator with the assumptions:
///// 1. the keychain program is deployed to the local validator at the address in the keychain idl
///// 2. the key set up in anchor.toml is funded with SOL (to deploy stache)

// then u can run: anchor test --provider.cluster localnet --skip-local-validator


const deployKeychain = () => {
  const deployCmd = `solana program deploy --url localhost -v --program-id $(pwd)/../keychain/target/deploy/keychain-keypair.json $(pwd)/../keychain/target/deploy/keychain.so`;
  execSync(deployCmd);
};

function randomName() {
  let name = Math.random().toString(36).substring(2, 5) + Math.random().toString(36).substring(2, 5);
  return name.toLowerCase();
}

// for setting up the keychaink
const domain = randomName();
const treasury = anchor.web3.Keypair.generate();
const renameCost = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL * 0.01);


const username = randomName();    // used as the keychain + stache name
const vaultName = randomName();


describe("stache", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const stacheProgram = anchor.workspace.Stache as Program<Stache>;
  const keychainProgram = new Program(KeychainIdl, KeychainIdl.metadata.address, provider);

  let userKeychainPda: PublicKey;
  let domainPda: PublicKey;
  let mint: Keypair;
  let adminAta: PublicKey;
  let userAta: PublicKey;
  let stachePda: PublicKey;
  let stachePdaBump: number;
  let vaultPda: PublicKey;
  let vaultPdaBump: number;
  let vaultAta: PublicKey;

  // for admin stuff
  const admin = anchor.web3.Keypair.generate();

  console.log(`\n\n...>>> user: ${provider.wallet.publicKey.toBase58()}`);
  console.log(`\n\n...>>> admin: ${admin.publicKey.toBase58()}`);


  it("sets up testing env", async () => {
    // Add your test here.
    // const tx = await program.methods.initialize().rpc();

    // just deploy by yourself
    // console.log(`deploying Keychain...`);
    // deployKeychain();
    // console.log("✔ Keychain Program deployed.");

    await connection.confirmTransaction(
        await connection.requestAirdrop(provider.wallet.publicKey, anchor.web3.LAMPORTS_PER_SOL * 50),
        "confirmed"
    );
    await connection.confirmTransaction(
        await connection.requestAirdrop(admin.publicKey, anchor.web3.LAMPORTS_PER_SOL * 50),
        "confirmed"
    );

    // create the keychain domain + user's keychain

    // our domain account
    [domainPda] = findDomainPda(domain, keychainProgram.programId);
    const [domainStatePda, domainStatePdaBump] = findDomainStatePda(domain, keychainProgram.programId);

    // our keychain accounts
    [userKeychainPda] = findKeychainPda(username, domain, keychainProgram.programId);
    const [userKeychainStatePda] = findKeychainStatePda(userKeychainPda, domain, keychainProgram.programId);
    // the "pointer" keychain key account
    const [userKeychainKeyPda] = findKeychainKeyPda(provider.wallet.publicKey, domain, keychainProgram.programId);

    console.log(`creating keychain domain: ${domain}...`);

    // first create the domain
    let txid = await keychainProgram.methods.createDomain(domain, renameCost).accounts({
      domain: domainPda,
      domainState: domainStatePda,
      authority: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId,
      treasury: treasury.publicKey
    }).rpc();
    console.log(`created keychain domain tx: ${txid}`);

    console.log(`creating keychain for : ${username}...`);

    // then create the keychain
    txid = await keychainProgram.methods.createKeychain(username).accounts({
      keychain: userKeychainPda,
      keychainState: userKeychainStatePda,
      key: userKeychainKeyPda,
      domain: domainPda,
      authority: provider.wallet.publicKey,
      wallet: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId,
    }).rpc();

    console.log(`created keychain for ${username}. tx: ${txid}`);

    // now let's create a token
    mint = await createTokenMint(connection, admin, provider.wallet.publicKey);
    console.log(`created token mint: ${mint.publicKey.toBase58()}`);
    // ata for the user
    userAta = await createAssociatedTokenAccount(connection, admin, mint.publicKey, provider.wallet.publicKey);
    adminAta = await createAssociatedTokenAccount(connection, admin, mint.publicKey, admin.publicKey);
    console.log(`created user ata: ${userAta.toBase58()}`);

    // now mint 10k tokens to each ata
    const numTokens = 10000;
    const tx = new Transaction().add(
        createMintToCheckedInstruction(
            mint.publicKey,
            userAta,
            provider.wallet.publicKey,
            numTokens * 1e9,
            9
        ),
    createMintToCheckedInstruction(
        mint.publicKey,
        adminAta,
        provider.wallet.publicKey,
        numTokens * 1e9,
        9
    )
    );
    txid = await provider.sendAndConfirm(tx);
    console.log(`minted ${numTokens} to user ata: ${userAta.toBase58()}, and admin ata: ${adminAta.toBase58()}, txid: ${txid}`);

  });

  it("creates a stache", async () => {

    [stachePda, stachePdaBump] = findStachePda(username, domainPda, stacheProgram.programId);

    let txid = await stacheProgram.methods.createStache().accounts({
      stache: stachePda,
      keychain: userKeychainPda,
      keychainProgram: keychainProgram.programId,
      systemProgram: SystemProgram.programId,
    }).rpc();

    let stache = await stacheProgram.account.currentStache.fetch(stachePda);
    console.log(`----> created stache for ${username} >>>> ${stachePda.toBase58()} <<<< bump: ${stache.bump} in tx: ${txid}`);
    assert.equal(stache.stacheid, username);
  });

  it("basic stash/unstash", async () => {

    const stacheMintAta  = getAssociatedTokenAddressSync(mint.publicKey, stachePda, true);

    // stash: this tx will create the stache's mint ata and deposit some tokens in there
    let tx = new Transaction().add(
        createAssociatedTokenAccountInstruction(provider.wallet.publicKey, stacheMintAta, stachePda, mint.publicKey),
        createTransferCheckedInstruction(userAta, mint.publicKey, stacheMintAta, provider.wallet.publicKey, 100 * 1e9, 9)
    );
    let txid = await provider.sendAndConfirm(tx);
    // console.log(`created stache mint ata: ${stacheMintAta.toBase58()}, txid: ${txid}`);
    // tx = new Transaction().add(
    //     // createAssociatedTokenAccountInstruction(provider.wallet.publicKey, stacheMintAta, beardPda, mint.publicKey),
    //     createTransferCheckedInstruction(userAta, mint.publicKey, stacheMintAta, beardPda, 100 * 1e9, 9)
    // );
    // txid = await provider.sendAndConfirm(tx);
    console.log(`created stache mint ata: ${stacheMintAta.toBase58()}, and deposited 100 tokens, txid: ${txid}`);
    let tokenAmount = await connection.getTokenAccountBalance(stacheMintAta);
    console.log(`>> stache mint ata balance: ${tokenAmount.value.uiAmount}`);
    tokenAmount = await connection.getTokenAccountBalance(userAta);
    console.log(`>> user ata balance: ${tokenAmount.value.uiAmount}`);


    // now let's stash via the stash instruction
    tx = await stacheProgram.methods.stash(new anchor.BN(500 * 1e9)).accounts({
      stache: stachePda,
      keychain: userKeychainPda,
      stacheAta: stacheMintAta,
      mint: mint.publicKey,
      owner: provider.wallet.publicKey,
      fromToken: userAta,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    }).transaction();

    txid = await provider.sendAndConfirm(tx);

    console.log(`called > stash: stash 500 more tokens, txid: ${txid}`);
    tokenAmount = await connection.getTokenAccountBalance(stacheMintAta);
    console.log(`new stache mint ata balance: ${tokenAmount.value.uiAmount}`);
    tokenAmount = await connection.getTokenAccountBalance(userAta);
    console.log(`new user ata balance: ${tokenAmount.value.uiAmount}`);

    // unstash

    tx = await stacheProgram.methods.unstash(new anchor.BN(225 * 1e9)).accounts({
      stache: stachePda,
      keychain: userKeychainPda,
      stacheAta: stacheMintAta,
      mint: mint.publicKey,
      owner: provider.wallet.publicKey,
      toToken: userAta,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    }).transaction();

    txid = await provider.sendAndConfirm(tx);

    console.log(`unstashed 225 tokens, txid: ${txid}`);
    tokenAmount = await connection.getTokenAccountBalance(stacheMintAta);
    console.log(`new stache mint ata balance: ${tokenAmount.value.uiAmount}`);
    tokenAmount = await connection.getTokenAccountBalance(userAta);
    console.log(`new user ata balance: ${tokenAmount.value.uiAmount}`);

  });

  it('creates a vault', async () => {

      // first vault index = 1
      [vaultPda, vaultPdaBump] = findVaultPda(1, username, domainPda, stacheProgram.programId);
      vaultAta  = getAssociatedTokenAddressSync(mint.publicKey, vaultPda, true);

    let txid = await stacheProgram.methods.createVault(vaultName, {twoSig: {}}).accounts({
        stache: stachePda,
        keychain: userKeychainPda,
        vault: vaultPda,
        authority: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      }).rpc();

      console.log(`created vault for ${username} >>>> ${vaultPda} <<<< bump: ${vaultPdaBump} in tx: ${txid}`);

      let userTokenBalance = await connection.getTokenAccountBalance(userAta);
      console.log(`user token balance: ${userTokenBalance.value.uiAmount}`);

      let tx = new Transaction().add(
          createAssociatedTokenAccountInstruction(provider.wallet.publicKey, vaultAta, vaultPda, mint.publicKey),
          createTransferCheckedInstruction(userAta, mint.publicKey, vaultAta, provider.wallet.publicKey, 5 * 1e9, 9)
      );
      txid = await provider.sendAndConfirm(tx);
      console.log(`deposited 5 tokens into vault, txid: ${txid}`);
      userTokenBalance = await connection.getTokenAccountBalance(userAta);
      console.log(`user token balance: ${userTokenBalance.value.uiAmount}`);
      let vaultTokenBalance = await connection.getTokenAccountBalance(vaultAta);
      console.log(`vault token balance: ${vaultTokenBalance.value.uiAmount}`);

  });

  it('destroys a vault', async () => {

    // now destroy the vault

    let txid = await stacheProgram.methods.destroyVault().accounts({
      stache: stachePda,
      keychain: userKeychainPda,
      vault: vaultPda,
      authority: provider.wallet.publicKey,
    }).rpc();

    console.log(`destroyed vault ${vaultName} >>>> ${vaultPda} <<<< in tx: ${txid}`)

    let userTokenBalance = await connection.getTokenAccountBalance(userAta);
    console.log(`new user token balance after vault destruction: ${userTokenBalance.value.uiAmount}`);

    // check that the vault is gone
    let vault = await stacheProgram.account.vault.fetchNullable(vaultPda);
    expect(vault).to.be.null;

    // check that the vault ata is gone
    let accountInfo = await provider.connection.getAccountInfo(vaultAta);
    expect(vault).to.be.null;

  });


  it("destroys a stache", async () => {
    let tx = await stacheProgram.methods.destroyStache().accounts({
      stache: stachePda,
      keychain: userKeychainPda,
      authority: provider.wallet.publicKey,
      keychainProgram: keychainProgram.programId,
      systemProgram: SystemProgram.programId,
    }).rpc();

    console.log(`destroyed stache for ${username} in tx: ${tx}`);

    let stache = await stacheProgram.account.currentStache.fetchNullable(stachePda);
    expect(stache).to.be.null;
  });

});
