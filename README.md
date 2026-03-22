# Surcouf: Deterministic Obstruction-free Consensus Simulation

## Overview

Surcouf is a deterministic simulation of an Obstruction-Free Consensus (OFC) protocol, built in Rust.

Evaluating consensus algorithms (such as Paxos or Raft) in standard multi-threaded environments often introduces OS-level scheduler non-determinism. This complicates the reproduction of edge-case race conditions and the rigorous evaluation of latency under varying network constraints.

To address this limitation, Surcouf executes the protocol within an event-driven simulation loop using the [DScale simulator](https://github.com/kshprenger/dscale). Real-world network execution is replaced by a formal model where logical time, latency distributions, and bandwidth limits are strictly controlled. For any given random seed, the sequence of message interleavings, probabilistic crash failures, and leader elections is completely reproducible. This architecture enables precise performance profiling across different cluster sizes ($N$).

## Architecture and Role Isolation

### 1. The Consensus Node

Each node operates as a strictly non-blocking state machine maintaining local state for the consensus rounds. Blocking operations are avoided in the event loop, so all state transitions happen via pattern matching on incoming strongly-typed messages.

- **Read Phase:** Upon receiving a proposal, the node broadcasts a `READ` request.
- **Gather Phase:** Nodes reply with their highest seen ballots and current estimates. The proposer buffers these to determine the safest value to propose.
- **Impose/Ack Phase:** The proposer attempts to force a value. If a majority quorum ($> N/2$) acknowledges the `IMPOSE` without seeing a higher ballot, the value is decided.

### 2. The Chaos Orchestrator

An orchestrator operates entirely outside the consensus protocol to inject faults and manage network-level contention.

- **Crash Injection:** It selects $f$ nodes and assigns them a crash probability ($\alpha$). On every processed network event, a faulty node rolls against $\alpha$. If triggered, it permanently drops all future network I/O, simulating a hard crash.
- **Leader Election:** It maintains a global logical timeout ($t_{le}$). When the timeout expires, it selects a non-crashed node as the leader and broadcasts a `HOLD` command to the cluster, halting competing proposals to resolve livelock contention.

## Formal Mathematical Model

The protocol ensures agreement on a value $v \in V = \{true, false\}$, returning either the decided value or an `abort` state.

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

## Testing Network Distributions

The simulator provides a command-line interface to evaluate the protocol under arbitrary network delay models. It automatically benchmarks the algorithm across multiple parameter combinations (cluster size, timeouts, and crash probabilities), outputting the metrics to a CSV file.

Latency is measured in logical time units (`Jiffies`), where 1 Jiffie approximates 1 ms in the system model.

**Supported distributions:**

- **Normal distribution `N(mean, std_dev)`:** Simulates standard network jitter (e.g., `N(10,3)` represents a 10ms average latency with a 3ms standard deviation).
- **Bernoulli distribution `B(p, val)`:** Simulates an erratic network where a message has a probability `p` of beeing lost (e.g., `B(0.9,20)` represents a 20ms latency with 10% of packet loss).

### Executing the Benchmark

To run the simulator, pass the target network distribution as a parameterized string argument:

```bash
# Evaluate with a Normal distribution
cargo run "N(10,3)"

# Evaluate with a Bernoulli distribution
cargo run "B(0.9,20)"
```

_Note: The simulation is fully deterministic. Re-executing with identical parameters guarantees the exact same event sequence and latency metrics._

Upon completion, a results file named after the defined distribution (e.g., `results_N(10,3).csv`) is generated in the root directory.

### Visualizing Results

A Jupyter notebook is provided in the `notebook/` directory to analyze the latency against the metrics.

### Tracing Protocol States

To observe low-level network events, and message interleavings during execution:

```bash
RUST_LOG=surcouf=debug cargo run "N(10,3)"
```
