{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
  };

  outputs = { nixpkgs, ... }:
    let
      inherit (nixpkgs) lib;

      makePackage = (system: dev:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
        in
        rec {
          default = pkgs.rustPlatform.buildRustPackage {
            name = "songrec";
            src = lib.cleanSourceWith rec {
              src = ./.;
              filter = path: type:
                lib.cleanSourceFilter path type
                && (
                  let
                    baseName = builtins.baseNameOf (builtins.toString path);
                    relPath = lib.removePrefix (builtins.toString ./.) (builtins.toString path);
                  in
                  lib.any (re: builtins.match re relPath != null) [
                    "/Cargo.toml"
                    "/Cargo.lock"
                    "/src"
                    "/src/.*"
                  ]
                );
            };

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
            ] ++ (if dev then
              with pkgs; with pkgs.gst_all_1; [
                clippy
                rustfmt
                rust-analyzer
              ] else [ ]);

            buildInputs = with pkgs; [
              alsa-lib
              openssl
            ];

            doCheck = false;
          };
        });
    in
    builtins.foldl' lib.recursiveUpdate { } (builtins.map
      (system: {
        devShells.${system} = makePackage system true;
        packages.${system} = makePackage system false;
      }
      )
      lib.systems.flakeExposed);
}
