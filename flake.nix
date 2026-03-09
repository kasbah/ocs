{
  description = "Search and resume OpenCode sessions across folders";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "opencode-session-search";
          version = "0.1.0";

          src = ./.;

          cargoHash = "sha256-UiNoBocG21BjAb9326PUr/QDaFbn32KzjdAQHQu3mr4=";

          meta = with pkgs.lib; {
            description = "Search and resume OpenCode sessions across folders";
            homepage = "https://github.com/kasbah/opencode-session-search";
            license = licenses.mit;
            mainProgram = "opencode-session-search";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
          ];
        };
      }
    );
}
