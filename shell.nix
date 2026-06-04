{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    llvmPackages_18.llvm
    llvmPackages_18.libllvm
    llvmPackages_18.libclang
    pkg-config
    rustc
    cargo

    # Required for linking LLVM
    libffi
    zlib
    ncurses
    libxml2

    # HTTP / networking support
    curl.dev
    curl
    gcc
    binutils
  ];

  LLVM_SYS_180_PREFIX = "${pkgs.llvmPackages_18.llvm.dev}";
  LLVM_CONFIG_PATH = "${pkgs.llvmPackages_18.llvm.dev}/bin/llvm-config";

  shellHook = ''
    echo "Atomic Language Development Environment"
    echo "LLVM version: $(llvm-config --version)"
    echo "Rust version: $(rustc --version)"
  '';
}
