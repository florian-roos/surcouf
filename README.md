# Surcouf: Deterministic Obstruction-free consensus simulation

_Note: This repository is currently under active development._

## Overview

Surcouf is a deterministic simulation of an Obstruction-free consensus (OFC) protocol built in Rust designed to solve the fundamental testing limitations of traditional concurrent systems. Typically, evaluating consensus algorithms (like Paxos) in standard multi-threaded environments introduces OS-level scheduler non-determinism (e.g. AKKA in Java). This makes it difficult to reproduce edge-case race conditions or rigorously benchmark latency under specific network constraints.

Surcouf solves this by running the protocol within a strictly single-threaded, event-driven simulation loop (using the DScale simulator https://github.com/kshprenger/dscale). It trades real-world execution for absolute reproducibility. Logical time, network latency distributions and bandwidth limits are strictly controlled. For any given random seed, the sequence of message interleavings, probabilistic crash failures and leader elections is mathematically identical, allowing precise performance profiling across varying cluster sizes ($N$).

## Architecture and Role Isolation

### 1. The Consensus Node

Each node operates as a strictly non-blocking state machine maintaining local state for the consensus rounds. Since blocking operations are forbidden in the event loop, all state transitions happen via pattern matching on incoming strongly-typed messages.

- **Read Phase:** Upon receiving a proposal, the node broadcasts a `READ` request.
- **Gather Phase:** Nodes reply with their highest seen ballots and current estimates. The proposer buffers these to determine the safest value to propose.
- **Impose/Ack Phase:** The proposer attempts to force a value. If a majority quorum ($> N/2$) acknowledges the `IMPOSE` without seeing a higher ballot, the value is decided.

### 2. The Chaos Orchestrator

Consensus must survive an asynchronous network with up to $f < N/2$ crash failures. The orchestrator operates entirely outside the consensus protocol to inject faults and manage contention.

- **Crash Injection:** It selects $f$ nodes and assigns them a crash probability ($\alpha$). On every processed network event, a faulty node rolls against $\alpha$. If triggered, it permanently drops all future network I/O, simulating a hard crash.
- **Leader Election:** It maintains a global logical timeout ($t_{le}$). When the timeout expires, it selects a non-crashed node as the leader and broadcasts a `HOLD` command to the cluster, halting competing proposals to resolve livelock contention.

## Formal Mathematical Model

The protocol ensures agreement on a value $v \in V = \{0, 1\}$, returning either the decided value or an `abort` state.

- **Validity:** Every decided value is a proposed value.
- **Agreement:** No two processes decide differently.
- **Obstruction-free termination:**
- If a correct process proposes, it eventually decides or aborts.
- If a correct process decides, no correct process aborts infinitely often.
- If exactly one correct process proposes sufficiently many times, it eventually decides.

## Complexity and Fault Tolerance

### Network and Failure Model

- **Nodes:** $N \in \{3, 10, 30, 70, 100\}$
- **Fault Tolerance:** Survives up to $f < N/2$ crash failures.
- **Message Complexity:** $O(N^2)$ per round due to the all-to-all broadcast nature of the `READ` and `IMPOSE` phases.

### Benchmark Parameters

The simulation evaluates consensus latency depending multiple parameters:

1. Cluster size ($N$)
2. Leader election timeout ($t_{le}$)
3. Crash probability ($\alpha$)

## Running the Benchmarks

This benchmark evaluate the latency of the algorithm for multiple $N$, $f$, $\alpha$, and $t_{le}$ and outputs the results as a CSV. Every measure is repeated five times with 5 different seeds.

$N$ = 3, 10, 30, 70, 100 (with $f$ = 1, 4, 14, 34, 49, respectively)
$\alpha$ = 0, 0.1, 1
$t_{le}$ = 200, 100, 50, 30, 20, 10 (`Jiffies` cf. (1) at the end of the README)

To run the benchmark suite:

```bash
cargo run > results.csv
```

### Tracing the Protocol

To observe the simulated network events:

```bash
RUST_LOG=surcouf=debug cargo run
```

### Output Format

The output uses the following CSV headers:
`N, f, Alpha, T_le, Seed, Latency`

(1) The latency is measured in DScale's logical time units (`Jiffies`). This decouples the simulation's timeline from local hardware execution speed. In our simulation we consider 1 `Jiffies` = 1 ms and we use for the latency of the messages between the processes a normal distribution N($\mu$=10ms, $\sigma$=3ms) to emulate a network behavior.
(2) The simulation is deterministic. Re-running the same parameter tuple (including the seed) will yield the exact same event sequence and latency.
