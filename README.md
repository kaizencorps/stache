# Intro 

With respect to existing terminology and concepts, Stache is essentially a “smart contract wallet,” the main purpose of which is to offer the user a superior set of features, 
functionality, security and accessibility, over what traditional wallets currently provide. However, we're billing 
Stache as a "smart wallet system," due to the nature of its architecture and the fact that contrary to actual 
smart contract wallets, each Stache smart wallet system isn't deployed as a separate smart contract (or Solana program
in this case).

The set of features that a smart wallet can offer can GREATLY reduce the barrier to entry for non-crypto natives by providing a user experience 
that’s orders of magnitude better than what’s currently available, effectively reducing the complexity of usability to 
that of a normal web2 experience that users are accustomed to.

The ability to scale the user experience without sacrificing self-custody (or just the concept of being able to scale self-custody), 
is critical for the mass adoption of web3. The primary goal of Stache, therefore, is: **To improve the user experience of 
interacting with web3 wallets and services to the point that mass adoption becomes feasible.** Everything follows this 
basic goal, and is the driving force behind everything the team is working on wrt Stache. To that end, the overarching
vision of Stache goes beyond that of a smart contract wallet, and extends to the idea of a smart wallet 
_platform_, comprised of multiple new smart wallet protocols. This platform will allow service providers to offer seamless 
and integrated services to users, resembling the superior experiences they're already familiar with outside of the web3 environment

# Grizzlython Goals

There are 3 primary goals for the initial v1 of Stache that we’ll be trying to accomplish *specifically for the Grizzlython hackathon:*

1. Introduce a brand new (and first) “smart wallet system" into the Solana ecosystem.
2. Demonstrate new functionality that’s now possible with Stache (and not available with current wallets).
3. Show that with the functionality and features that Stache provides, it'll be possible to provide a great user experience that eliminates web3 UX barriers. 
# Functional Description

An important mental model to keep in mind is that while a user has direct control over assets in a standard wallet, and must 
explicitly control any wallet activity (generally manually), in a smart wallet, a program has control (and
ownership) of assets and activity, and this activity can be initiated and controlled by things OTHER than the user (such as
other programs, both on and off-chain), but via user-specified parameters. In the case of Stache, it’s a combination of a
smart wallet working in conjunction with potentially several user-controlled “standard” wallets, but linked together 
through a Solana program.

### Creating a Stache

A user creates a Stache by selecting a unique Stache ID (aka a username), and submitting a transaction to the Stache program.
This will create a Stache with a single linked wallet. Additional wallets can then be linked to the Stache via a 2-step process:

1. The 1st linked wallet will need to add the 2nd wallet’s address to the Stache
2. The user will then need to “log in” to the Stache using the added wallet along with Stache name, and then verify ownership 
of the 2nd wallet by sending a transaction to the Stache.

At this point, the Stache will now have 2 linked wallets.

*Note: A wallet may only be added and linked to a single Stache. In order to link it to another Stache, it must first be removed 
from the first one. Though technically, a wallet CAN be added to more than one Stache through the use of "domains," which 
are set up in the Keychain program (https://github.com/kaizencorps/keychain).

### Connecting to a Stache

A user accesses his Stache via a single Stache-linked wallet, the same way he'd connect to any existing dapp. From here, the user is able to control assets in the Stache and 
in the connected wallet. This means he can deposit assets from his connected wallet into the Stache, and he can withdraw assets
from the Stache. 

This happens with the wallet that initially created the Stache, or a wallet that an already-linked wallet added to the Stache,
and which was subsequently verified via a transaction submitted to the Stache by the added wallet.

# Technicals

Stache works in conjunction with Keychain, as Keychain is used to store the linked wallets, which are then verified
by Stache during usage. In order to create a Stache account, the Keychain account needs to be created first. 

To run the test suite, you'll need to first deploy Keychain to your localnet,
and then deploy Stache. To build Stache, you'll also need to have checked out and built Keychain in a sibling directory
(notice the reference in the Cargo.toml file).

Once those 2 programs are deployed to local running test validator, you'll need to update/verify Keychain's address in 
the idl/keychain.json file before you can run the tests. Then, with your test validator running, you 
can run the test suite with the command: 

```anchor test --provider.cluster localnet --skip-local-validator```


# v1
For the initial version of Stache (v1), the focus will be limited to the user’s direct experience and interaction with
Stache and his/her associated wallets. Potential 3rd party dapp/service integration may be offered in a limited fashion
but will be a secondary goal.

# Disclaimer

This code is unaudited, written very rapidly for Grizzlython without much concern for security, and is under heavy development. 
It's not even close to being production-ready, and due to the nature of it, is intentionally not yet deployed on mainnet. 
Use at your own risk.

The initial version for the Grizzlython hackathon is deployed on devnet, but is meant to serve as a proof of concept and
demonstration of the concepts presented here and in the presentation. Wrt security, the program probably  
has the resemblence of swiss cheese.

On devnet, the latest version is currently deployed at: staWbEoarryYLMGxptDQKLvVMD8HqhzmBfWsAWGJQrz

# Contact

We'd love any feedback, and of course if you have any questions, issues, or concerns, please don't hesitate to reach out to us at:

hoorah@kaizencorps.com

[Twitter](https://twitter.com/kaizencorps_)

[Discord](https://discord.gg/XefWDWrB)

