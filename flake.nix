{
  description = "Rust development environment for kiteconnect-rs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    mcp_rustdoc = {
      url = "github:SPRAGE/mcp_rustdoc";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix, flake-utils, mcp_rustdoc }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        
        # Get the latest stable Rust toolchain with WASM target
        toolchain = with fenix.packages.${system}; combine [
          stable.rustc
          stable.cargo
          stable.clippy
          stable.rustfmt
          stable.rust-src
          targets.wasm32-unknown-unknown.stable.rust-std
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            toolchain
            
            # Development tools
            rust-analyzer
            cargo-watch
            cargo-edit
            cargo-audit
            
            # WASM tools
            wasm-pack
            binaryen
            
            # Build dependencies
            pkg-config
            openssl
            
            # Additional utilities
            git
            just
            
            # MCP Rustdoc binary
            mcp_rustdoc.packages.${system}.default
          ];

          # Environment variables
          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          
          shellHook = ''
            echo "🦀 Rust development environment loaded!"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo ""
            echo "Available targets:"
            rustup target list --installed 2>/dev/null || echo "  - x86_64-unknown-linux-gnu (default)"
            echo "  - wasm32-unknown-unknown"
            echo ""
            echo "Additional tools:"
            echo "  rdocs-mcp                      # MCP server for Rustdoc"
            echo ""
            echo "Useful commands:"
            echo "  cargo build                    # Build the project"
            echo "  cargo test                     # Run tests"
            echo "  cargo clippy                   # Run linter"
            echo "  cargo fmt                      # Format code"
            echo "  cargo build --target wasm32-unknown-unknown  # Build for WASM"
            echo "  cargo watch -x check           # Watch for changes"
          '';
        };

      });
}
