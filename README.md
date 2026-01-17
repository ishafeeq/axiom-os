<div align="center">
  <img src="docs/axiom-logo.png" alt="Axiom OS Logo" width="120" />
  <h1>Axiom OS</h1>
  <h3>The Distributed Intelligence Layer & Cloud-Native Operating System</h3>
  
  <p><b>Architecting the Post-Kubernetes Era with High-Performance Rust & Isolated Wasm Kernels.</b></p>

  <p>
    <a href="https://axiom-ccp.shafeeq.dev/" style="text-decoration:none;">
      <img src="https://img.shields.io/badge/Live_Demo-Axiom_CCP-624DE8?style=for-the-badge&logo=rocket" alt="Live Demo" />
    </a>
  </p>

  <p>
    ✨ <b>Experience the Future of Distributed Intelligence Live:</b> ✨<br/>
    <a href="https://axiom-ccp.shafeeq.dev/"><b>https://axiom-ccp.shafeeq.dev/</b></a><br/>
    <i>Witness sub-millisecond Wasm orchestration, real-time observability, and adaptive resource binding in action.</i>
  </p>
  
  <p>
    <img src="https://img.shields.io/badge/Language-Rust-orange?style=flat-square&logo=rust" alt="Rust" />
    <img src="https://img.shields.io/badge/Runtime-WebAssembly-624DE8?style=flat-square&logo=webassembly" alt="Wasm" />
    <img src="https://img.shields.io/badge/Architecture-Distributed_Intelligence-blue?style=flat-square" alt="Distributed Architecture" />
    <img src="https://img.shields.io/badge/Observability-OpenTelemetry-blue?style=flat-square&logo=opentelemetry" alt="OTel" />
  </p>
</div>

---

## 🔱 The Paradigm: Distributed Intelligence, Simplified.

Axiom OS is a **distributed operating system** designed from the ground up for the AI-native era. It eliminates the traditional "infrastructure-logic" entropy by abstracting microservices into highly isolated, sub-millisecond **Wasm Kernels**. 

While traditional architectures drown in Docker containers, K8s manifest bloat, and interpreted runtime overhead (Node.js/Python), Axiom leverages a **High-Performance Rust Host** to orchestrate intelligence at 1/10th the cost and 10x the velocity.

---

## 🏗️ Current Implementation Highlights

### ⚡ Sub-Millisecond Runtime
- **Wasmtime Integration**: Uses the `wasmtime` engine with `wasi-preview1` support for lightning-fast, secure execution.
- **Nanoprocess Isolation**: Each business logic unit (Tomain) runs in its own memory-isolated sandbox.
- **Hot-Reloading**: Deploy `.wasm` binaries directly into the shell in **under 10ms** via Unix domain sockets.

### 🛡️ Managed Egress & Security
- **Egress Guard**: A host-level proxy that intercepts all network calls. Wasm kernels use logical aliases instead of physical URLs.
- **Resilience Engine**: Built-in **Circuit Breakers** (reporting state changes to RED perspective) and **Rate Limiting** implemented in the `axiom-shell`.
- **Capability Manifests**: Kernels have zero access to host resources unless explicitly defined in their manifest.

### 🌐 Distributed Control Plane (CCP)
- **Centralized Registry**: Maintains a shared `session.json` state across all shell instances, enabling seamless multi-shell communication.
- **Real-Time Dashboards**: Built with **React** and **Axum**, providing deep visibility into capability graphs and service health.
- **Adaptive Binding**: Dynamically re-bind database and LLM aliases without touching guest code.

### 🛠️ SDK & Macros
- **axiom-sdk**: High-level Rust library for Wasm guest apps, abstracting HTTP, DB, and Logging.
- **axiom-macros**: Procedural macros (`#[axiom_api]`, `#[axiom_export_reflect]`) for automatic reflection and host-function bridging.

---

## 🛰️ AI Architect & LLM Infrastructure

Axiom OS is purposefully built to serve as the **backbone for LLM Orchestration**:

*   **Unified AI Gateway**: Professional integration with **LiteLLM**, **Portkey**, and **Helicone** for cost-optimized routing and fallback.
*   **Architectural Observability**: Native support for **Prometheus** and **Jaeger** to track token consumption and agentic trace-lines.
*   **Audit Mode**: Integrated RED perspective for auditing state-changing operations across the cluster.

---

## 🔮 Next Steps & In-line Features

- [ ] **WASI Sockets Support**: Transitioning towards standard WASI networking for broader library compatibility.
- [ ] **Evaluation Gates**: Strategic hooks for **Promptfoo** and **DeepEval** to automate quality gating during promotion.
- [ ] **Adaptive Resource Scaling**: Automatic scaling of Wasm kernels based on real-time throughput metrics.
- [ ] **Cross-Regional CCP Sync**: Global state synchronization for geo-distributed Axiom clusters.
- [ ] **Advanced Capability Discovery**: Auto-discovery of available Tomains via the reflection API.

---

## 🚀 The Developer Workflow (`ax` CLI)

```bash
# Initialize an AI-native workspace
ax init my-service

# Build and hot-swap local binary
./build.sh

# Dynamically bind a production database
ax bind --name DB_MAIN --url postgresql://user:pass@prod-cluster.aws

# Promote intelligence across environments
ax promote --from staging --to prod
```

---

## 🎨 Visual Control Plane (CCP)

The **Central Control Plane** provides a deep-visibility dashboard into the "Brain" of your cluster.

| Multi-Environment Visibility | Intelligent Service Hub |
| :--- | :--- |
| ![Dev Dashboard](docs/dev-dashboard.png) | ![Service Hub](docs/dev-service-dashboard.png) |
| *Visualizing capability graphs across 4 environments.* | *Deep-dive metrics and lifecycle management per tomain.* |

---

## 🛠️ Installation & Setup

Axiom is a lightweight, unified binary architecture.

```bash
# Clone the repository
git clone https://github.com/shafeeq/axiom-os

# Build the Full Stack (CLI, Shell, CCP)
./build.sh

# Start the local shell
ax init
```

---

## 💡 Why Axiom?

Axiom OS represents a shift in how we think about cloud infrastructure. It isn't just a deployment tool—it's a **Capability Orchestrator**. By moving the responsibility of "How" (Infra) away from the "What" (Logic), we enable:
1.  **AI Agents to code faster** (Fewer tokens wasted on boilerplate).
2.  **90% Reduction in Infrastructure Overhead** (Multi-tenancy vs Single-tenant containers).
3.  **Enterprise-Grade Security** (Wasm SFI - Software Fault Isolation / Hardware-Level Sandboxing).

---
*Built with ❤️ by [Shafeequl Islam](https://shafeeq.dev) — Senior Architect focusing on AI-native Distributed Systems.*
