# Intro 
With respect to existing terminology and concepts, Stache is 
essentiall a “smart contract wallet,” the main purpose of which is to offer the user a superior set of features, 
functionality, security and accessibility, over what traditional wallets currently provide. The set of features that a 
smart contract wallet offer can GREATLY reduce the barrier to entry for non-crypto natives by providing a user experience 
that’s orders of magnitude better than what’s currently available, effectively reducing the complexity of usability to 
that of a normal web2 experience that users are accustomed to.

The ability to scale the user experience without sacrificing self-custody (or just the concept of being able to scale self-custody), 
is critical for the mass adoption of web3. The primary goal of Stache, therefore, is: **To improve the user experience of 
interacting with web3 wallets and services to the point that mass adoption becomes feasible.** Everything follows this 
basic goal, and is the driving force behind everything the team is working on wrt Stache. To that end, the overarching
vision of Stache goes beyond that of a smart contract wallet, and extends to the idea of a smart wallet 
_system_ (or platform), which incorporates a way for service providers to provide an integrated set of services to offer
users superior experiences resembling those that they’re already accustomed to outside of web3.

# Grizzlython Goals

There are 3 primary goals for the initial v1 of Stache that we’ll be trying to accomplish *specifically for the Grizzlython hackathon:*

1. Introduce a brand new (and first) “smart contract wallet” into the Solana ecosystem, along with a new model for a smart
2. contract wallet that nobody has ever seen or even fuckin thought about before (as far as I can tell).
2. Demonstrate new functionality that’s now possible with Stache (and not available with current wallets).
3. Show that with the functionality and features that Stache provides, we can provide a delightful user experience that
4. eliminates web3 usability friction and fear.

# Functional Description

An important mental model to keep in mind is that while a user has direct control over assets in a standard wallet, and must 
explicitly control any wallet activity (generally manually), in a smart contract wallet, a smart contract has control (and
ownership) of assets and activity, and this activity can be initiated and controlled by things OTHER than the user (such as
other programs, both on and off-chain), but via user-controlled parameters. In the case of Stache, it’s a combination of a
smart contract wallet working in conjunction with potentially several user-controlled “standard” wallets, but linked together 
through a Solana program.

### Creating a Stache

A user creates a Stache by selecting a unique Stache name (aka a username), and submitting a transaction to the Stache program.
This will create a Stache with a single linked wallet. Additional wallets can then be linked to the Stache via a 2-step process:

1. The 1st linked wallet will need to add the 2nd wallet’s address to the Stache
2. The user will then need to “log in” to the Stache using the added wallet along with Stache name, and then verify ownership 
of the 2nd wallet by sending a transaction to the Stache.

At this point, the Stache will now have 2 linked wallets.

*Note: A wallet may only be added and linked to a single Stache. In order to link it to another Stache, it must first be removed 
from the first one.

### Connecting to a Stache

A user accesses his Stache via a single Stache-linked wallet. From here, the user is able to control assets in the Stache and 
in the connected wallet. This means he can deposit assets from his connected wallet into the Stache, and he can withdraw assets
from the Stache. He CAN NOT, however, control assets in other linked wallets.

This happens with the wallet that initially created the Stache, or a wallet that an already-linked wallet added to the Stache,
and which was subsequently verified via a transaction submitted to the Stache by the added wallet.

# v1
For the initial version of Stache (v1), the focus will be limited to the user’s direct experience and interaction with
Stache and his/her associated wallets. Potential 3rd party dapp/service integration may be offered in a limited fashion
but will be a secondary goal.
