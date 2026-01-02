# Axiom OS Monorepo

Welcome to **Axiom OS**, the zero-gravity development environment for building and deploying distributed capabilities via WebAssembly (Wasm) Kernels and Tomains.

## Architecture 

The monorepo contains the four primary pillars of the Axiom ecosystem:

1. **`axiom-shell` (The "Body")**:
   - **Responsibility:** Native Host for Wasm Kernels.
   - **Tech Stack:** Rust, Wasmtime, Axum.
   - **Focus:** Multi-tenant performance and capability-based security.

2. **`axiom-ccp` (Central Control Plane, The "Brain")**:
   - **Responsibility:** Global registry of Tomains and environment-to-resource mapping.
   - **Tech Stack:** Go/Node.js for backend, React/Next.js for frontend.
   - **Focus:** Visualization of RED/BLUE/GREEN environments and secret management.

3. **`axiom-ide` (The "Lens")**:
   - **Responsibility:** Environment-agnostic coding interface.
   - **Tech Stack:** Electron/React or VS Code Extension.
   - **Focus:** Session-locking (Color Contexts), local Shell orchestration, and SDK integration.

4. **`axiom-sdk` (The "Language")**:
   - **Responsibility:** High-level 'Intent' APIs.
   - **Tech Stack:** Wit (WebAssembly Interface Types).
   - **Focus:** Defining the standard interface for DB, Cache, and Networking that Kernels must use.

## The Zero-Gravity Lifecycle (Dev-to-Prod)

Axiom OS eliminates the need for `.env` files, environment variables in source code, or messy local setups. Instead, it relies on Contextual Session-Locking (RED, BLUE, GREEN).

Here is the flow of a Kernel from development to production:

### 1. Development Context (RED)
*   The developer uses the **Axiom IDE** (`axiom-ide`), which inherently establishes a **RED** context. 
*   In this context, hardcoded secrets do not exist. Instead, the IDE seamlessly maps high-level abstract connections (via `axiom-sdk`) to a local, transient instance of the **Axiom Shell** (`axiom-shell`).
*   The developer focuses solely on the "Intent" (e.g., "Store this data") rather than the "How" (e.g., Postgres connection strings).

### 2. Staging / Integration Context (BLUE)
*   Once the Kernel is stable, the developer pushes the Tomain to the **Central Control Plane** (`axiom-ccp`).
*   The CCP locks the Tomain into a **BLUE** context. Here, the abstract Intent APIs are bound to staging database instances and mock third-party endpoints.
*   The underlying execution is handled by an isolated, staging-tier Axiom Shell cluster. Capabilities are verified, and integration tests are executed.

### 3. Production Context (GREEN)
*   After passing the BLUE context, the Tomain is promoted to **GREEN**. 
*   The CCP injects production-grade resource bindings dynamically into the Wasm host runner (`axiom-shell`).
*   The original Kernel code remains identical to the code written in the RED phase. There are absolutely no environment checks like `if (process.env.NODE_ENV === "production")` inside the code itself. The execution environment (The Shell) securely handles the actual fulfillment of Intent using production credentials, which the Wasm Kernel never sees.

## Getting Started

See the included `Makefile` to orchestrate this monorepo locally:

```bash
make setup  # Prepare the directories
make dev    # Start the IDE and a local RED Shell
make build  # Compile your Wasm Kernels via the SDK
make deploy # Promote to CCP
```
# axiom-ai
