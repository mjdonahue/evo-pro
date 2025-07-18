Architectural Write-Up: A Secure, Decentralized, Local-First Messaging System


This document outlines the key architectural pillars for the application, focusing on identity, communication, data consistency, and security.


1. Core Communication: Direct Actor-to-Actor Messaging


 * Problem: How do peers communicate privately and securely?
 * Solution: Rely on direct, end-to-end encrypted, actor-to-actor messaging via libp2p. Avoid gossipsub for private conversations.
 * Implementation:
     * The GatewayActor on each peer serves as the single, secure entry point for all incoming P2P messages.
     * To send a message in a conversation, the sender's ConversationManagerActor looks up the PeerId of each participant and sends a Signed message
       directly to their respective GatewayActor.
 * Key Point: This model provides the strongest privacy guarantees, as there is no public topic to eavesdrop on. It aligns perfectly with the existing
   actor-based architecture.

2. Identity Management: Decentralized Identifiers (DIDs)


 * Problem: A user may have multiple devices, each with a different PeerId. We need a stable, persistent identifier for the user, not the device.
 * Solution: Use a Decentralized Identifier (DID) as the canonical, user-level ID. The did:key method is an excellent starting point.
 * Implementation:
     * A user's identity is their master public key, represented as a DID string (e.g., did:key:z...). This DID is stored in the users table in the local
       database.
     * The DID is used to resolve a DID Document, which is a public, signed record containing the user's master public key and a list of their current,
       online PeerIds (as service endpoints).
 * Key Point: This decouples user identity from device identity, which is essential for multi-device support and long-term account stability.

3. Peer & User Discovery: The Kademlia DHT


 * Problem: How do users find each other on the network without a central server?
 * Solution: Use a general-purpose Kademlia Distributed Hash Table (DHT) to store and look up DID Documents.
 * Implementation:
     * Instantiate a second, dedicated libp2p::kad::Behaviour in the main Swarm, separate from kameo's internal DHT. This DHT is for application-level
       data.
     * The Key for the DHT is the user's full DID string (did:key:z...). Using the raw public key bytes is not recommended as it lacks context,
       versioning, and interoperability.
     * The Value is the user's DID Document, signed by their master private key.
     * A dedicated DhtManagerActor will provide a clean API for the rest of the application to put and get records from the DHT.
 * Key Point: The DHT provides a scalable, decentralized, and censorship-resistant mechanism for peer discovery. The cryptographic link between the
   did:key and the content of the DID Document makes it secure against spoofing.


4. Offline Messaging & Data Sync: A Hybrid CRDT Approach


 * Problem: Users must be able to send and receive messages even if they are not online at the same time. The application must be fully functional offline.
 * Solution: Use a CRDT library (automerge or cr-sqlite) for local data management, combined with a decentralized mailbox system for transport.
 * Implementation:
     1. Local-First with CRDTs: All database writes (new messages, document edits, etc.) are committed to a local CRDT-enabled database (cr-sqlite) or
        document store (automerge). This makes the app feel instantaneous and work perfectly offline.
     2. Scoped Sync: Syncing is handled on a per-conversation basis to ensure efficiency and privacy. We do not sync the entire database state with every
        peer.
     3. Mailbox Nodes: A user designates a high-availability "Mailbox Node" in their DID Document. This node's job is to store encrypted CRDT patches for
        the user when they are offline.
     4. Transport: When a user comes online, they generate a patch of their local changes (scoped to a conversation) and send it to other participants. If
        a participant is offline, the patch is sent to their designated Mailbox Node. When the recipient comes online, they fetch the patches from their
        mailbox.
 * Key Point: This hybrid model provides the seamless, "it just works" user experience of a centralized app while retaining the resilience and user control
   of a decentralized one.



5. The Cornerstone: Verifiable, Secure Data


 * Problem: In a multi-writer system, how do you prevent malicious peers from forging or tampering with data?
 * Solution: Every individual change must be cryptographically signed and independently verifiable. The "Apply-then-Verify" pattern is the most robust
   and elegant solution.
 * Implementation:
     1. Schema Design: Every user-modifiable table must contain columns for authorship proof (e.g., author_did, signature, row_version).
     2. Write Path: When creating or modifying a row, the application must create a signature of the row's canonical data and store it within the row
        itself.
     3. Sync Path: The application sends the raw, efficient CRDT patches generated by automerge or cr-sqlite.
     4. Verification Path (Crucial): When a patch is received, the application:
        a. Starts a transaction or creates a temporary fork of the data.
        b. Applies the patch using the CRDT library's efficient merge algorithm.
        c. Diffs the result to see which specific rows were changed.
        d. For each changed row, it re-verifies the signature contained within the row against the data in that same row.
        e. If all signatures are valid, the transaction is committed. If even one is invalid, the transaction is rolled back, and the fraudulent data
is discarded.
 * Key Point: This makes the data itself self-authenticating. It is the ultimate source of truth, protecting the system's integrity even if the network
   or other peers are compromised. It correctly uses the CRDT library for merging while layering on a generic, non-negotiable security guarantee.
