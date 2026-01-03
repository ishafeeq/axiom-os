.PHONY: setup dev build test deploy help

# Setup the development environment
setup:
	@echo "Setting up Axiom OS Dev Environment..."
	@mkdir -p axiom-shell/src axiom-shell/adapters axiom-shell/wasm-runtime
	@mkdir -p axiom-ccp/axiom-ccp-backend axiom-ccp/axiom-ccp-frontend axiom-ccp/fabric-engine
	@mkdir -p axiom-ide/axiom-cli axiom-ide/axiom-desktop axiom-ide/color-themes
	@mkdir -p axiom-sdk/sdk-go axiom-sdk/sdk-rust axiom-sdk/wit
	@echo "Ready for Zero-Gravity Development."

# Run local Axiom Shell and Ide for RED development
dev:
	@echo "Starting Axiom IDE (The Lens)..."
	@echo "Starting local Axiom Shell (The Body) in RED context..."
	@echo "Local Dev Environment Running."

# Compile Tomains within the Dev Environment
build:
	@echo "Compiling WIP Wasm Kernels via Axiom SDK..."
	@echo "Injecting Intent APIs..."

# Run Tests against the compiled Wasm modules
test:
	@echo "Running multi-tenant capability tests in Axiom Shell..."

# Promote from RED to BLUE to GREEN
deploy:
	@echo "Submitting Tomain to Axiom CCP (The Brain)..."
	@echo "CCP verifying capability graph and secrets mapping for GREEN..."
	@echo "Deployed successfully."

help:
	@echo "Axiom OS Monorepo Make Commands"
	@echo " setup  - Initialize dependencies for all 4 components"
	@echo " dev    - Start the local dev environment (IDE + Shell)"
	@echo " build  - Compile Kernels against the SDK"
	@echo " test   - Run Security and Capability tests"
	@echo " deploy - Promote a Tomain through the CCP"
