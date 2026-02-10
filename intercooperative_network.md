# THE INTERCOOPERATIVE NETWORK: COMPREHENSIVE VISION, ARCHITECTURE & PROGRESS

*Executable Consent. Verifiable Legitimacy. Democratic Coordination at Global Scale.*

**Matt Faherty — February 2026 — Compiled from documentation system and verified against codebase**

---

## EXECUTIVE SUMMARY

The InterCooperative Network (ICN) is a decentralized coordination substrate for cooperative organizations — a Rust-based peer-to-peer daemon that enables cooperatives, communities, and federations to coordinate identity, trust, governance, economics, contracts, and compute without central servers or blockchain-style global consensus. Individuals hold sovereign cryptographic identity through a post-quantum hybrid signature system (Ed25519 + ML-DSA-65). Organizations are established — not registered — through signed charters with scoped consensus and auditable receipts. The Cooperative Contract Language (CCL) serves as the universal contract grammar, interpreted by a fuel-metered, capability-gated constraint engine that permanently enforces structural guardrails while allowing democratic governance to adjust all parameters within those guardrails. Economics operates through mutual-credit accounting and typed claims on real deliverables — tracked through the same auditable infrastructure. Markets are allowed; opacity and capture are not. Scaling works through federations of federations: voluntary, contractual, nestable, and preserving lower-level sovereignty at every layer. Corporations can transact through the network but cannot govern within it.

The system is approximately 272,000 lines of Rust across 28+ workspace crates, with 2,287+ passing tests and 18+ completed development phases. The goal is government with receipts — legitimacy you can prove, consent you can verify, coordination you can audit.

---

## I. THE CORE THESIS

ICN makes consent operational by providing sovereign identity, democratic governance primitives, and federated economic coordination — expressed as auditable contracts with constitutional guardrails enforced by the protocol.

The premise is historically grounded: infrastructure that lowers coordination cost reshapes power. The printing press reorganized religion and politics. Joint-stock corporations reorganized production. The internet reorganized information distribution. Each shift occurred because a new coordination substrate made the old structure unnecessary for an expanding share of human activity.

ICN proposes the next substrate: one in which cooperation scales by federation-through-agreement rather than consolidation-through-ownership, where legitimacy is cryptographically provable rather than merely asserted, and where economic coordination operates through enforceable claims on real deliverables rather than through a single abstracted value scalar controlled by those who already hold power.

### The Problem

Modern democratic governance is largely ceremonial. Citizens are born into systems they did not consent to, governed by rules they cannot audit, subject to resource allocations they cannot trace. The social contract is a metaphor with no implementation.

Economic coordination is monopolized by two structures: corporations, which coordinate through centralized ownership and extract surplus upward; and states, which coordinate through bureaucratic hierarchy and are susceptible to capture. Both operate behind closed doors. Both claim legitimacy they cannot prove.

In the cooperative movement, an alternative exists in principle but not in practice. Cooperatives lack the connective tissue that would allow them to function as a coherent economic and political force. Today, cooperatives rent their compute, storage, and identity services from corporations — AWS, Google, Microsoft — that extract profit and control their data.

ICN is the coordination infrastructure. And ICN aims to replace corporate dependency with a cooperative cloud: distributed infrastructure collectively owned and operated by the cooperative movement itself, governed democratically and shared via mutual aid rather than rent extraction.

### The Thesis in One Sentence

**ICN makes legitimacy something you can prove:** entities exist only when their charter is written, scoped, and consented to through an auditable consensus process, under constitutional guardrails enforced by a constraint engine that governance can parameterize but never disable.

---

## II. THE ENTITY MODEL: FOUR CO-EQUAL PILLARS

ICN recognizes four co-equal entity types. None is ontologically superior. Each is sovereign. Each can run infrastructure. Each can enter contracts. Each can participate in governance. Each can federate. These are implemented in the `icn-entity` crate with type-safe `EntityId`, `EntityType`, `EntityStatus`, and membership validation.

### 1. Individuals

The individual is the root of sovereignty. **You do not create an identity with a provider. You are the provider. Organizations issue you membership, not existence.**

An individual holds cryptographic identity through the Sovereign Digital Identity System (SDIS), implemented in `icn-identity` and `icn-crypto-pq`. Each identity is anchored to a permanent Anchor ID computed from a Verifiable Unique Identifier (VUI) and a genesis key, meaning keys can be rotated without changing identity itself. The signature system is post-quantum hybrid: every signature requires verification under both Ed25519 (classical, 64 bytes) and ML-DSA-65 (post-quantum lattice-based, ~3,293 bytes) — both must verify. The identity can be chained to additional devices through an Age-encrypted keystore (v4/v5 format). No platform owns this identity.

**Implementation reality:** The `icn-crypto-pq` crate implements `HybridKeypair`, `HybridSignature`, and `HybridPublicKey`. The `icn-identity` crate provides `KeyPair` with optional PQ support via feature flag, `Anchor` types with VUI commitments, `KeyBundle` for rotatable keys bound to anchors, and `AgeKeyStore` with SDIS initialization. The React Native SDK (`sdk/react-native/src/hybrid-crypto.ts`) provides the `HybridCrypto` class for mobile wallet integration.

The individual's wallet is the identity root. An individual can also operate personal node(s) — usually as a power user or developer, but also for personal use. A personal node can cache and verify receipts independently, host personal services (data vaults, cooperative communications, local apps), contribute compute to commons pools under personal policy, and act as a developer/operator for org apps and services.

### 2. Communities

A community is any social or civic entity defined by shared membership, governance rules, and purpose. Communities can be informal affinity groups, civic service entities, municipal service providers, territorial governance bodies, or scopes of the global commons itself. Communities are civic actors. They govern services, territory, or affinity. They run nodes. They enter federation agreements when coordination needs exceed local scope.

**Implementation reality:** The `icn-community` crate provides four community types (Geographic, Interest, Solidarity, Ecosystem), mixed membership (both individuals and cooperatives), resource pooling (shared compute, storage, credit pools), and lifecycle management (Forming → Active → Suspended → Dissolved). The `icn-entity` crate's `EntityRegistry` validates membership: Communities accept Individuals, Cooperatives, and other Communities (nested). Charter-based governance is defined via `icn-governance::Charter::community()`.

### 3. Cooperatives

Cooperatives are economic production entities. They produce goods and services, employ or engage members, distribute surplus, and govern production democratically. In ICN, cooperatives are first-class civic actors — they run nodes, define charters in CCL, enter economic agreements, participate in supply chains, pool capital, and coordinate production.

**Implementation reality:** The `icn-coop` crate provides a `CoopActor` with message-passing interface for cooperative lifecycle: `CreateFromRequest`, `ActivateCooperative` (requires charter hash), `SignCharter`, `AddMember`, `ListCooperatives`, `DeleteCooperative`, etc. The `LifecycleManager` handles state transitions: create → activate → suspend → resume → start_dissolution → complete_dissolution. The `Cooperative` struct tracks `charter_hash`, `capital_pool`, `bylaws` (CCL contract IDs), membership `tiers`, and formation `required_signatures`. Membership is managed through the `apps/membership` crate's `CoopMembershipManager` with share-based voting and labor assignment tracking.

### 4. Federations

Federations are first-class coordination and enforcement entities. A federation exists when multiple sovereign entities need shared coordination without surrendering sovereignty. A federation can contain any combination of communities, cooperatives, individuals, and other federations. **Federations are nestable** — a federation of federations is the natural scaling mechanism.

**Implementation reality:** The `icn-federation` crate provides `CooperativeRegistry` for member discovery, `FederationMember` enum (Cooperative | Community), `BilateralClearingAgreement` for economic coordination, `ClearingManager` for cross-cooperative transfers, `NettingEngine` for multilateral debt resolution, `FederatedDidResolver` for cross-boundary identity, `FederatedGossipRouter` with scoped message routing, `AttestationStore` for trust bridging, and `ReceiptClearingManager` for cross-scope settlement. The `EntityRegistry` validates that Federations accept Individuals, Cooperatives, Communities, and other Federations (recursive).

| Entity | Primary Function | Infrastructure | Scaling Pattern |
|---|---|---|---|
| **Individual** | Sovereign identity; root of consent | SDIS wallet (mobile + chained devices) + optional personal node | Joins multiple entities; contributes compute |
| **Community** | Civic governance; social coordination; service delivery | Node or cell of nodes | Federates with other communities and coops |
| **Cooperative** | Economic production; democratic labor; surplus distribution | Node or cell of nodes | Federates for supply chains and capital pooling |
| **Federation** | Cross-entity coordination; validation; enforcement; clearing | Federation node(s) or cluster | Nests into larger federations recursively |

---

## III. THE ESTABLISHMENT DOCTRINE: CONSENT AS PROTOCOL

ICN entities are not registered. They are *established*. An entity achieves legitimacy within the network only when: (1) a charter exists in CCL defining governance, membership, rights, and obligations; (2) the charter is scoped; (3) participants have consented with cryptographically signed membership; (4) a consensus mechanism exists; (5) the resulting establishment is cryptographically signed as an **Establishment Receipt**.

**No consent, no entity. No scope, no entity. No defined consensus path, no entity.**

### Scope-Bound Consensus (Not Blockchain)

ICN does not use global consensus. It uses *scope-bound consensus*: each charter declares its own consensus rules, and those rules apply only within that charter's scope. Nodes operate independently under a local-first principle and reconcile via gossip protocol — eventual consistency through push/pull synchronization with anti-entropy mechanisms and causal ordering via vector clocks.

| Aspect | ICN | Blockchain |
|---|---|---|
| **Consensus** | Local autonomy, scope-bound, eventual consistency | Global consensus, total ordering |
| **Trust model** | Assumes social relationships; trust computed from participation | Assumes hostility; trust from computation |
| **Tokens** | Mutual credit (internal accounting, not tradeable) | Tradeable assets |
| **Speculation** | Impossible by design within scope | Common |
| **Energy** | Minimal (no mining) | Often significant |
| **Governance** | Democratic, cooperative, one-member-one-vote | Token-weighted |

### Legitimacy Receipts

Legitimacy is not a status — it is an artifact. Named, verifiable receipts are produced at every decision point as chained Ed25519+ML-DSA signed execution receipts with clearing semantics:

- **Establishment Receipt:** charter hash + scope definition + membership set + ratification proof + founding signatures.
- **Decision Receipt:** proposal identifier + scope + participation threshold + tally proof + enactment hash.
- **Allocation Receipt:** budget authority reference + disbursement record + invoice and delivery links + reconciliation state.

**Implementation reality:** The execution receipt system is implemented in the kernel/app separation initiative. The `icn-kernel-api` crate defines `ScopeLevel` (Local, Cell, Org, Federation, Commons) and receipt types. The `icn-federation::ReceiptClearingManager` handles cross-scope settlement by converting `ExecutionReceipt`s into `CrossCoopTransfer`s for Federation/Commons scoped operations.

### Governance Pluralism Within Guardrails

ICN does not mandate a single governance model. Different entities can choose different internal structures: direct democracy, delegated councils, liquid delegation, rotating stewardship, or any other model their members consent to. What ICN mandates is that whatever model is chosen must be declared explicitly, that membership must be explicit, that decision procedures must be auditable, that exit and appeal paths must be defined.

**Implementation reality:** The `icn-governance` crate provides `GovernanceConfig` with configurable `quorum_threshold`, `approval_threshold` (simple majority, supermajority, unanimous), `voting_period_seconds`, and `deliberation_period_seconds`. The `Charter` type supports `OrgType` variants (Cooperative, Community, Federation) each with tailored policies. `GovernanceDomain` enables scoped decision-making (e.g., finance, security). Governance profiles include `ConsentBased`, `MajorityRule`, `SupermajorityRule`, and `Custom` configurations.

### Constitutional Invariants

Some rules are constitutional — enforced at the protocol level and modifiable only through commons-level consensus. These include: baseline member rights, minimum transparency/auditability, anti-capture constraints, exit rights, due process primitives, prohibition on purchasing governance standing, scope constraints, and immutability of audit trails.

### The Commons

The "commons" is the root federation-of-federations: the highest scope level. It is a constitutional scope whose sole function is maintaining protocol-level guardrails. Changes at commons scope require supermajority thresholds, multi-stage ratification, and cooling-off periods. Exit from the commons scope is possible — identity is portable, local agreements are portable, sovereignty is preserved even in disagreement.

### Audit Visibility Model

Four tiers: **Public facts** (entity existence, charter hashes, decision receipt outcomes), **Member-visible facts** (budget details, vote records, allocation receipts), **Need-to-know facts** (personal data protected by STARK-based ZK proofs for selective disclosure), and **Regulatory export** (optional compliance artifacts).

---

## IV. THE CONSTRAINT ENGINE AND COOPERATIVE CONTRACT LANGUAGE

### The Kernel/App Separation: The Meaning Firewall

The most important architectural decision is the strict boundary between the kernel (pure enforcement) and domain-specific applications (Policy Oracles):

**The kernel enforces constraints. The kernel does not decide them.**

The kernel is domain-agnostic. It enforces five permanent mechanisms:

1. **Transport authentication:** Every message is signature-verified against a DID.
2. **Replay protection:** Every message is sequence-checked.
3. **Rate limiting:** Every actor is bounded by configurable rate limits.
4. **Capability gating:** Every action requires explicit, revocable capability grants.
5. **Credit gating:** Every economic action is bounded by a credit ceiling.

These mechanisms are permanent. Governance can change parameters but cannot remove the mechanisms. This is enforced at the Rust crate dependency level: kernel crates do not depend on any domain crate.

**Implementation reality:** The `icn-kernel-api` crate defines the `PolicyOracle` trait with `evaluate(&self, request: &PolicyRequest) -> Result<PolicyDecision, PolicyError>`, accepting `ConstraintSet` and `Domain` parameters. The `ServiceRegistry` provides `TrustService`, `LedgerService`, `GovernanceService`, and `BlobService` trait interfaces through which domain logic communicates with the kernel. The kernel crate has no `use` statements importing trust, governance, or economic logic — only trait bounds.

### Policy Oracles

Domain-specific logic lives in Policy Oracles:

- **Trust Oracle** (`apps/trust/`): Computes web-of-participation scores and supplies trust-derived constraints. Initializes `TrustServices` with shared `MisbehaviorDetector` and integrates via `TrustService` trait.
- **Ledger Oracle** (`icn-ledger`): Manages mutual-credit accounting and supplies credit ceilings.
- **Governance Oracle** (`apps/governance/`): Processes proposals and votes via `GovernanceActor` and supplies governance-derived policy changes. Implements `GovernanceOps` trait for RPC integration.
- **Membership Oracle** (`apps/membership/`): Manages membership state with entity-type-specific managers (`CoopMembershipManager`, `CommunityMembershipManager`) and supplies capability sets.

### CCL: The Cooperative Contract Language

CCL is the universal grammar of the ICN — a governance and economic DSL with built-in safety constraints.

**Implementation reality:** The `icn-ccl` crate (~22,000 lines, 249 passing tests) implements:

- **AST-Based Interpreter:** Deterministic execution with `Contract`, `Rule`, `Stmt`, `Expr` types.
- **Capabilities:** `ReadLedger`, `WriteLedger`, `ReadTrust`, `Compute` — each must be explicitly declared.
- **Fuel Metering:** Prevents infinite loops — each operation has a configurable fuel cost (`FuelConfig` with `expr_cost`, `stmt_cost`, `call_cost`, etc.). Default costs: expression evaluation = 1, transfer = 100, state read = 5, state write = 20.
- **Built-in Functions:** `balance()`, `trust_score()`, `record_transaction()`.
- **Type System:** Int, String, Bool, List, Map.
- **Not Turing-complete:** No recursion, bounded iteration. Same inputs always produce same outputs.
- **Dispute System:** Re-execution for verification with misbehavior penalties.
- **Fuel Estimation:** Static analysis via `FuelEstimator` for pre-execution fuel limit recommendations.
- **Dry Run Mode:** Execute without side effects for accurate fuel estimation.

```
rule "deliver_service" {
  let provider_did = "did:icn:alice";
  let consumer_did = "did:icn:bob";
  let hours = 10;
  if trust_score(provider_did) > 0.4 {
    record_transaction(consumer_did, provider_did, hours, "hours");
  }
}
```

### What CCL Expresses

CCL encodes three integrated domains:

**Identity and Membership:** Identity roots (`did:icn:<base58btc-ed25519-pubkey>` or `did:icn:<base58-anchor-id>`), attestations, membership credentials, role credentials (scoped/time-bounded), capability grants (enforced by kernel capability gate), delegation mandates, and revocation records.

**Governance Logic:** All cooperative functions as state machines:
- **Proposal lifecycle:** Draft → Deliberation → Open → {Accepted, Rejected, NoQuorum, Cancelled}. Each transition produces a Decision Receipt.
- **Budget lifecycle:** request → review → approval → disbursement → receipts → reconciliation.
- **Dispute lifecycle:** filing → notification → evidence → hearing → decision → remedy → appeal.
- **Membership lifecycle:** application → verification → admission → renewal/expiration → revocation → appeal.

**Economic Contracts:** Assets, transformations, claims, and mutual credit settlement (detailed in Section V).

### Guardrails Built Into the Language

CCL prevents a defined class of extractive mechanisms by construction: standing cannot be purchased, rights must be scoped, due process is required for punitive actions, exit rights must be explicitly defined, revocation requires stated grounds, delegation must be time-bounded and revocable, audit artifacts are immutable, economic claims must reference typed deliverables, and scope escalation requires explicit consensus.

### Dispute Resolution and Enforcement Boundaries

Enforcement is scope-bound and operates at three levels:

- **Within an entity:** Membership suspension, capability revocation, access restriction — through declared dispute process with due process protections.
- **Across entities:** Cross-entity defaults enforced by federation clearing rules — access revocation, collateral pool claims, trust/reputation impacts, denial of future participation.
- **Beyond the protocol:** Physical-world enforcement uses portable legitimacy artifacts (Establishment Receipts, Decision Receipts, audit trails) that interface with compatible legal systems. The protocol does not pretend to replace courts.

**ICN doesn't enforce outcomes with violence. It enforces them with access, standing, clearing, and proof — and makes defection visible, attributable, and containable.**

---

## V. ECONOMICS: CLAIMS ON REALITY, NOT ABSTRACTIONS

### Mutual Credit Substrate

ICN implements mutual-credit economics with double-entry bookkeeping, recorded in a Merkle-DAG journal.

**Implementation reality:** The `icn-ledger` crate (~16,000 lines, 80+ tests) implements:

- **Double-Entry Invariant:** `Σ debits == Σ credits` per currency, enforced on every `JournalEntry`.
- **Merkle-DAG Structure:** Each entry has a `ContentHash` (SHA-256 of canonical serialization), `parents: Vec<ContentHash>` (DAG links), `author: Did`, `accounts: Vec<AccountDelta>`, and `signature: Option<Signature>`.
- **Multi-Currency:** Separate balances per currency per account — hours, USD, kWh, custom units.
- **Credit Limits:** Per-participant, per-currency overdraft limits. Dynamic limits based on `participation_history`, `attestations_received`, `contracts_fulfilled`, and `dispute_record`. New member throttling with progressive ramping.
- **Quarantine:** Entries that violate invariants are isolated via `QuarantineStore`.
- **Fork Detection/Resolution:** `ForkDetector` and `ForkResolver` (Phase 18 Week 5) detect conflicting entries.
- **Gossip Sync:** Entries propagate via `ledger:sync` topic with signature, double-entry, credit limit, and trust validation.
- **Balance Computation:** Real-time balance from sum of credits minus debits per currency.

**Gap identified:** No treasury DID per cooperative. Payments are attributed to individuals, not coops. Communities can't hold collective funds yet.

### Economic Primitives

**Assets:** Typed objects with lifecycles — durables (with depreciation schedules), consumables (quantity tracking), perishables (expiration curves), services (time-bounded with completion proofs), rights/access (governed by entitlements), and commons resources (charter-governed shared pools).

**Transformations:** First-class concept — declared processes taking typed inputs to typed outputs under constraints, with yield rules, proof requirements, and waste accounting.

**Claims:** Rights to future delivery of assets/services under defined conditions. Every financial instrument is a specialization: loans, entitlements, insurance (conditional), cooperative surplus distribution, bonds.

### Multiple Economic Ecosystems

Claims reference typed deliverables rather than a universal token, so multiple economic ecosystems coexist. Scope-specific units of account: care credits, energy units, food shares, labor-hours, mutual credit in locally defined basket indices. Liquidity emerges from the clearing system, not from a coin.

**Implementation reality:** The `icn-federation::ClearingManager` manages bilateral clearing agreements with configurable exchange rates between currencies. `CrossCoopTransfer` tracks source/destination currency, amounts, and exchange rates. `ClearingPosition` tracks `coop_a_owes_b` and `coop_b_owes_a` with `net_position()` calculation.

The `NettingEngine` implements multilateral debt resolution using DFS cycle detection:
1. Build directed graph (nodes = coops, edges = debts)
2. Bilateral netting first (A↔B simplification)
3. Find cycles using DFS from each node
4. For each cycle, cancel the minimum debt
5. Repeat until no more cycles

Example: A owes B $100, B owes C $80, C owes A $60 → After netting: A owes B $40, B owes C $20, C owes A $0.

Settlement intervals are configurable (daily, weekly, monthly). Imbalance limits prevent runaway credit exposure.

### Price Formation

Scope-local markets: negotiated exchange among typed claims within defined scope. Matching produces commitments. Settlement follows clearing rules. Federations provide clearing and netting services that publish reference exchange ratios based on actual transaction patterns.

### Universal Basic Services

A UBS pattern: community issues a UBS charter in CCL defining eligibility, guaranteed services, authorized providers, compensation mechanisms, quality standards, and audit rules. Provider cooperatives sign service compacts. The federation coordinates capacity matching and clears settlement.

### Economic Guardrails

Structural safety limits preventing financialization: maximum abstraction layers, transparency of claim provenance, mandatory collateral above thresholds, anti-speculation constraints, anti-monopoly constraints, labor standards requirements, and separation of service revenue and governance standing.

---

## VI. COMPUTE AND THE COOPERATIVE CLOUD

### The Missing Spine: How "Government with Receipts" Becomes Running Infrastructure

In ICN, governance isn't a meeting. It's a deployment pipeline with constitutional guardrails. The node is the actual "civic body" — it hosts entity state machines, runs apps/services, enforces constraints at execution time, participates in replication and audit, and produces the receipts that make governance real.

### Four Compute Tiers

| Tier | Description | Role |
|---|---|---|
| **Wallet** (Individual) | Identity, credentials, signing, selective disclosure | Authorizes society |
| **Node** (Sovereign Operators) | Enforcement + state + services | Runs society |
| **Cell** (LAN Pool) | 3+ nodes on a LAN federated for HA + locality | Local reliability |
| **Commons Compute** (Federated Cloud) | Pooled capacity across entities/scopes | Shared infrastructure |

**Wallets don't "run society." Nodes do. Wallets authorize society.**

### The ICN Daemon (`icnd`)

**Implementation reality:** `icnd` is a Rust daemon built on Tokio's async runtime with actor-based service supervision. The `Supervisor` (`icn-core/src/supervisor/`) coordinates all subsystems through focused submodules:

- `init_network` — QUIC transport with TLS 1.3 mutual authentication, mDNS discovery, NAT traversal
- `init_gossip` — Eventually-consistent topic-based pub/sub
- `init_governance` — Proposal/voting state machines
- `init_compute` — Task scheduling and execution
- `init_federation` — Cross-entity coordination
- `init_gateway` — REST + WebSocket API (Actix-web on port 8080, JWT auth)
- `init_steward` — SDIS steward network services
- `init_community` / `init_coop` / `init_entity` — Entity management
- `init_rpc` — JSON-RPC interface
- `init_notifications` — Event notification system
- `init_resource_enforcer` — Resource limit enforcement
- `init_snapshot` — Graceful restart state persistence
- `init_contract_registry` — CCL contract management

The supervisor manages actor lifecycle: creates actors in dependency order, wires callbacks and channels, holds long-lived handles, and coordinates graceful shutdown with state snapshot export (gossip vector clocks, topic subscriptions, network state, misbehavior detector reputation scores).

Management through `icnctl` (CLI) and `icn-console` (interactive TUI).

### Node Roles

A single node may play multiple roles:

- **Identity/Membership node:** Credential verification, standing checks, revocation logs.
- **Governance node:** Proposal lifecycle, tally, decision receipts, policy parameter outputs.
- **Ledger/Clearing node:** Mutual credit journal, claim settlement, allocation receipts.
- **Service node:** Runs apps that deliver real services (UBS scheduling, procurement, logistics).
- **Federation bridge node:** Cross-entity dispute, clearing, protocol translation, auditing.

### Cells: 3+ Nodes on a LAN

When 3+ nodes federate into a cell on a LAN, you get:

- High availability (no single box "is the government")
- Local consensus for cell-scoped decisions/services
- Shared scheduling for service workloads
- Local replication/anti-entropy with fast convergence
- Org-operated trust boundary (the org owns the hardware and policy)

**Implementation reality:** The `icn-kernel-api` defines `ScopeLevel::Cell` and `CellId` identifiers. The Cells and Scopes system provides scope-aware placement logic. Cells are how a co-op office becomes a cloud and how a city becomes a civic cluster.

### Apps/Services as "Civic Software"

Organizations write apps that become part of governance and economics because they run under the same constraint model:

- Apps are signed, versioned, capability-scoped
- They run in a fuel-metered, capability-limited runtime
- Their outputs generate receipts and can mutate state only through authorized interfaces
- They can be deployed by scope: cell-only, org-wide, federation scope, commons scope

**Implementation reality:** The `icn-core/src/apps/` module provides an `AppRuntime` with `AppNamespace`, `StateFactory`, and `BlobHandle`. The `icn-compute` crate implements `LocalExecutor` and `WasmExecutor` — both execute CCL contracts through the interpreter with fuel metering. Tasks are submitted via `ComputeTask` with `TaskCode::Ccl(source)` or `TaskCode::Wasm(bytecode)`.

### Scope-Aware Execution: How Workloads Move

Every workload declares scope of authority, data locality needs, trust requirements, capabilities required, fuel budget, and audit level. Placement resolution defaults local-first:

1. **Local Cell (LAN):** Best latency, strongest locality. Primary for day-to-day governance, local service delivery.
2. **Org Cluster (multi-site):** When cell capacity fails or work is org-wide. Internal clearing, multi-department services.
3. **Federation Compute:** Cross-entity by definition. Markets, clearing, dispute coordination, supply chain edges.
4. **Commons Compute:** Shared/public workloads and overflow. Shared libraries, ecosystem services, burst compute.

**Local by default. Federated by consent. Commons by contribution.**

**Implementation reality:** The `icn-compute` crate implements a scheduling system with:
- `PlacementRequest` with `ResourceProfile` and `LocalityHint` (DataLocality, NetworkProximity, GeographicRegion, ColocateWith)
- `DefaultPlacementPolicy` scoring executors based on trust (40%), resources (30%), fuel cost (15%), locality (10%), random jitter (5%)
- `LocalityContext` tracking `submitter_rtt_ms`, `local_blob_count`, `total_blob_count`, region info
- `BlobLocationRegistry` mapping blob hashes to peer locations for data-aware placement
- `MigrationPolicy` for live task migration based on changing conditions

### The Full Interaction Loop: Identity → Governance → Economics → Compute → Receipts

**1) Identity authorizes standing:** Individual wallet signs in, proves membership/standing (selective disclosure if needed). Node verifies credentials → issues short-lived capabilities (scoped, revocable). The person doesn't "log in." They present standing.

**2) Governance produces enforceable policy outputs:** A proposal is created and processed (Draft → Deliberation → Open → Accepted/Rejected). Governance produces a Decision Receipt. The Governance Oracle outputs policy parameters / capability templates / budget rules. Decisions aren't "announcements." They are parameter changes the system enforces.

**3) Economics binds decisions to resource reality:** Budget authority created/modified by decision. Allocations and claims minted/updated under credit ceilings. Deliverables and transformations tracked. Allocation → Allocation Receipt + settlement hooks. Policy becomes resource movement with traceability.

**4) Compute executes the work that makes society real:** The decision + allocation triggers workload scheduling: "Run procurement matching," "Schedule childcare services under UBS charter," "Dispatch logistics job," "Compute clearing netting for federation market." Scheduler places workloads in the right scope based on constraints. Workloads execute in a capability-limited runtime, emit execution proofs/receipts. Governance is no longer theater; it's running workflows with receipts.

**5) Receipts close the loop into audit and legitimacy:** Execution produces auditable artifacts: proof of service delivery, proof of transformation, proof of settlement. Those become inputs to ledger state and future governance decisions. Legitimacy is recursive: past receipts become future constraints.

---

## VII. INFRASTRUCTURE: PARTICIPANT-OWNED, ADVERSARIAL BY DEFAULT

### Gossip Protocol

**Implementation reality:** The `icn-gossip` crate implements a comprehensive gossip protocol:

- **Push Protocol:** Broadcast new content hashes via `Announcement`.
- **Pull Protocol:** Request missing entries via `PullRequest`/`PullResponse` with backpressure and nonce-based correlation.
- **Anti-Entropy:** Periodic Bloom filter exchange (`emit_digest`/`emit_all_digests`) for convergence.
- **Vector Clocks:** Causal ordering per peer via `VectorClock` — detect duplicates and establish happens-before relationships.
- **Access Control:** `Public`, `Private`, `TrustGated(threshold)`, `Participants(dids)`.
- **Topics:** `ledger:sync` (journal entries), `trust:edges` (trust graph updates), `governance:votes`, `compute:submit`, `federation:registry`.
- **Peer Sync Manager:** Per-peer synchronization state with `Backoff` (300-5000ms range), outstanding request tracking, and deficit-based backpressure.
- **State Persistence:** Export/restore of vector clocks, subscriptions, and topic metadata for graceful restart (entries re-gossiped via anti-entropy).
- **Blob Transfer Protocol:** `BlobRequest`/`BlobResponse` with 64KB chunking, provider registry for peer-to-peer data transfer.
- **Scope-Aware Routing:** `FederatedGossipRouter` with `GossipScope` (Local, Federation, Public).

### Trust Graph

**Implementation reality:** The `icn-trust` crate implements web-of-participation trust computation:

- **Trust Classes:** Isolated (<0.1), Known (0.1-0.4), Partner (0.4-0.7), Federated (0.7+).
- **Multi-Graph Support:** `TypedTrustGraph` with separate graphs and weights:
  - Social: 60/40 direct/transitive (reputation spreads)
  - Economic: 80/20 (payment history matters most)
  - Technical: 90/10 (node performance is personal)
- **Algorithm:** PageRank-like: `TrustScore(A→B) = DirectTrust * weight + TransitiveTrust * (1-weight)`. Transitive trust computed as average of weighted 2-hop paths.
- **Performance Optimizations (Phase 22):** LRU cache for O(1) lookups (5 min TTL), Bloom filter for O(1) "definitely not reachable" queries, priority queue for early termination.
- **Trust Decay:** Temporal dimension for score decay.
- **Shared Computation Module:** `computation.rs` provides the core algorithm used by both `TrustGraph` (actor-backed) and standalone implementations.

### Byzantine Fault Detection

**Implementation reality:** The `icn-security` crate implements comprehensive Byzantine fault detection:

- **MisbehaviorDetector** (~490 lines): 7 violation types with severity scoring:
  - **Critical (10 points):** ConflictingLedgerEntries, ConflictingSignedStatements, ReplayAttack → auto-ban
  - **Major (5 points):** FailedComputeVerification, InvalidSignature
  - **Minor (1 point):** ExcessiveResourceUse, TrustGraphSpam
- **ReputationScore** (0.0 to 1.0): Dynamic penalty system — 5% loss per severity point, 1% recovery per hour (configurable). Quarantine at score < 0.5, auto-ban on critical violations.
- **Integration:** Shared between NetworkActor, GossipActor, and ComputeActor. Trust penalty callback automatically updates trust graph via `TrustService` trait.
- **Evidence Storage:** Per-violation evidence with 64KB max, SHA-256 hashing for larger evidence, max 100 violations per peer.
- **Persistence:** Reputation scores, bans, and quarantine state persisted via `security_store` across restarts.
- **Prometheus Metrics:** `icn_misbehavior_violations_detected_total`, `icn_misbehavior_reputation_updated_total`, `icn_misbehavior_quarantined_total`.

### Network Partition Detection and Healing

**Implementation reality:** Phase 18 Week 3 implemented comprehensive partition detection and healing:

- **PartitionDetector:** Last-seen timestamp tracking for all peers. Configurable threshold (default: 5 minutes). Automatic partition detection via `is_partitioned()` and `get_partitioned_peers()`.
- **PartitionHealer:** `VectorClockMerger` for detecting version gaps and concurrent updates. `ConflictResolver` with policy-based resolution per data type:
  - LedgerEntry → RequiresManual (critical financial data)
  - Contract → LastWriteWins (timestamp-based)
  - TrustEdge → LastWriteWins
  - GossipEntry → Merge (combine both)
- **Healing Flow:** `PartitionHealRequest` → `PartitionHealResponse` → vector clock merge → `PullRequest` for diverged topics.
- **Integration Tests:** 5-node partition tests validate no balance corruption and trust edge consistency after healing.
- **Chaos Tests:** `test_network_partition_recovery` in `icn-testkit/tests/chaos_tests.rs` verifies 4-node partition and convergence.

### Security Posture

Built in Rust — eliminating buffer overflows, use-after-free, null pointer dereferences. Post-quantum hybrid signatures from the ground level. Six permanent kernel guarantees: signature verification, replay protection, capability gating, credit gating, rate limiting, deterministic execution.

**Determinism boundary clarified:** Determinism applies per action within a scope: given the same scoped inputs to a charter-defined state machine, nodes compute the same canonical decision artifacts and the same receipt hashes. Gossip may arrive out of order. The world is messy. The legitimacy artifacts are not.

### Threat Model

- **Sybil attacks:** VUI via threshold PRF across steward network.
- **Governance capture:** Constitutional anti-capture constraints, separation of economic contribution from governance weight.
- **Node compromise:** Distributed topology, cryptographic verification, scope-bound blast radius, Byzantine fault detection with 7 violation types.
- **Federation collusion:** Transparency requirements, audit trails, member oversight, exit rights.
- **Fork attacks:** Fork evidence tracking in MisbehaviorDetector.
- **Metadata leakage:** STARK-based ZK proofs, selective disclosure.
- **Denial of service:** Per-actor rate limiting, cell-based load distribution.

---

## VIII. OBSERVABILITY AND OPERATIONS

**Implementation reality:** The `icn-obs` crate provides comprehensive observability:

- **100+ Prometheus Metrics** across categories: Network, Gossip, Ledger, Compute, Trust, Governance, Upgrade, Security, Supervisor.
- **Tracing:** Structured JSON logging via `tokio-tracing` with configurable levels.
- **Metrics Server:** Port 9100 (configurable) serving Prometheus-compatible `/metrics` endpoint.
- **Health Endpoints:** `GET /v1/health` returning status, peer count, gossip entry count.
- **Grafana Dashboard:** Pre-configured `monitoring/grafana-dashboard.json` with panels: Network Overview, Gossip Protocol, Ledger, Security & Rate Limiting, Graceful Restart & Snapshots, Version Negotiation.
- **Alert Rules:** 40+ Prometheus alert rules across 8 groups (Byzantine detection, network health, ledger consistency, gossip performance, compute layer, governance, system resources, monitoring system).
- **Docker Compose Stack:** Complete Prometheus + Grafana + AlertManager deployment in `monitoring/` and `docker/`.
- **Per-subsystem metrics files:** `icn-obs/src/metrics/` with modules for supervisor, gossip, ledger, network, compute, trust, governance, gateway, misbehavior.

---

## IX. LEGACY COMPATIBILITY AND THE CORPORATE BOUNDARY

### Adapter Architecture

- **Identity bridge:** OIDC, SAML-style patterns, SCIM directory sync. SDIS DID format (`did:icn:`) designed for federated resolution.
- **Payments bridge:** Gateway integration with credit unions and cooperative banks.
- **Compliance export:** Audit artifacts in standard formats for legal jurisdictions.

### The Corporate Boundary: Transact, Not Govern

**Only organizations that are democratically established, member-owned, and governed by signed charters are entities of the network.** Corporations can enter scoped contracts, transact through approved gateways, access federated markets. They cannot hold sovereign standing, participate in governance, or purchase influence.

**Protocol-level definition:** Entities must have one-member-one-vote (or equivalent standing rules) and non-transferable governance standing. Any structure with transferable governance rights or equity-weighted control is a counterparty, not a citizen.

### The Credit Union Bridge

Credit unions serve as natural gateway entities — already member-owned, democratically governed, regulated, and expert in mediating between financial systems. They facilitate exchange without granting corporate actors governance access.

---

## X. PROGRESS AND REALITY

### What Is Built (Verified Against Codebase)

**Core Infrastructure (Complete):**
- `icn-identity` — Ed25519 keypairs, DID format, keystore v4/v5, Age encryption ✅
- `icn-crypto-pq` — ML-DSA-65 + ML-KEM post-quantum hybrid signatures ✅
- `icn-trust` — Web-of-participation trust graph with multi-graph support ✅
- `icn-gossip` — Push/pull/anti-entropy gossip with vector clocks, blob transfer ✅
- `icn-net` — QUIC transport, mDNS discovery, NAT traversal, blob registry ✅
- `icn-ledger` — Double-entry mutual credit with Merkle-DAG, multi-currency ✅
- `icn-ccl` — Cooperative Contract Language interpreter (249 tests) ✅
- `icn-governance` — Charter system, proposal lifecycle, voting primitives ✅
- `icn-security` — Byzantine fault detection, reputation, quarantine/ban ✅
- `icn-compute` — Task scheduling, placement, local + WASM execution ✅
- `icn-federation` — Registry, clearing, netting, attestations, scoped gossip ✅
- `icn-entity` — Unified entity model with recursive membership ✅
- `icn-coop` — Cooperative management with lifecycle and membership ✅
- `icn-community` — Community types, mixed membership, resource pooling ✅
- `icn-core` — Supervisor, actor lifecycle, graceful shutdown, state snapshots ✅
- `icn-gateway` — REST + WebSocket API with JWT auth, OpenAPI docs ✅
- `icn-obs` — 100+ Prometheus metrics, structured logging ✅
- `icn-snapshot` — Graceful restart state persistence ✅
- `icn-store` — Sled-backed content-addressable storage + hybrid blob store ✅
- `icn-testkit` — Cluster testing, chaos tests, partition simulation ✅
- `icn-kernel-api` — PolicyOracle trait, ServiceRegistry, ScopeLevel, CellId ✅
- `icn-rpc` — JSON-RPC server for daemon communication ✅
- `icn-time` — Timestamp utilities ✅

**SDIS Identity (Implemented):**
- Post-quantum hybrid signatures (Ed25519 + ML-DSA-65) ✅
- Anchor types with VUI commitments ✅
- KeyBundle with version-monotonic rotation ✅
- Keystore v4/v5 with PQ key persistence ✅
- Threshold PRF VUI computation types ✅
- React Native SDK with HybridCrypto class ✅
- TypeScript SDK ✅

**Tooling:**
- `icnd` — Daemon binary ✅
- `icnctl` — CLI management tool ✅
- `icn-console` — Interactive TUI ✅
- Docker Compose test networks (4-node including Byzantine) ✅
- K3s cluster with self-hosted CI ✅
- Monitoring stack (Prometheus + Grafana + AlertManager) ✅
- Pilot PWA web interface ✅

**Apps Layer (Kernel/App Separation):**
- `apps/trust/` — Trust oracle with TrustService integration ✅
- `apps/governance/` — Governance actor with event bus ✅
- `apps/membership/` — Unified membership management ✅

### What Remains

- Service discovery completion (~40% complete)
- Commons resource pool governance integration (~50% complete)
- SDIS enrollment flow (steward ceremony) — planned
- Federated task settlement
- Pilot cooperative selection and deployment
- Mobile-friendly endpoint optimization
- Multi-currency support and cross-currency settlement
- Weighted and delegated voting
- Governance templates for common cooperative structures
- Advanced credit policies (demurrage, seasonal adjustments)
- Cooperative treasury DIDs (per-entity economic identity)
- CoopOS (projected 2-3 years after kernel stabilization)
- OpenTelemetry/Jaeger distributed tracing integration
- Pre-built Grafana dashboard improvements
- Detection of selective message dropping (requires protocol heartbeats)
- Community reporting mechanism via governance

### What This Means

The system is approximately 75% complete toward pilot readiness. The hard architectural decisions have been made and implemented. The kernel/app separation ensures domain logic can evolve without destabilizing enforcement. The post-quantum cryptographic foundation means identity won't need rebuilding. The mutual-credit economic layer has been through agent-based simulation. The Byzantine fault detection system has been tested against fork attacks, replay attacks, and signature forgery.

---

## XI. THE NON-NEGOTIABLES

- Sovereign identity held in SDIS wallets with post-quantum hybrid signatures, not granted by platforms.
- Participant-owned infrastructure: nodes operated by communities, cooperatives, federations, and individuals.
- Explicit membership and governance expressed in signed cooperative contracts.
- Scope-bound consensus declared in charters, not imposed by a global chain.
- The constraint engine: permanent enforcement mechanisms that governance can parameterize but never disable.
- The meaning firewall: kernel does not execute governance logic, only enforces constraints.
- Federation-by-agreement as the scaling mechanism, nestable to global scope.
- Constitutional guardrails changeable only by commons-level consensus, with fork rights preserved.
- Verifiable legitimacy through named receipts: Establishment, Decision, and Allocation.
- Auditable allocation with structural separation between transparency and surveillance.
- Mutual-credit economics grounded in typed deliverables, not tradeable tokens.
- Adversarial-by-default network security posture with enumerated threat model and Byzantine fault detection.
- No governance standing without democratic establishment.
- No corporate capture: economic participation without political influence.
- The social contract, fully expressed and cryptographically enforceable.

**Everything else is implementation detail.**

---

> *"Consent you can verify. Democracy you can audit. Coordination you can trust. Government with receipts."*
>
> — The InterCooperative Network