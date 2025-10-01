{
  description = "Noita Entangled Worlds";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    systems = {
      url = "github:nix-systems/default";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, rust-overlay, systems, }:
    let
      inherit (nixpkgs) lib;
      eachSystem = lib.genAttrs (import systems);
      pkgsFor = eachSystem (system:
        import nixpkgs {
          localSystem = system;
          overlays = [ self.overlays.default ];
        });
    in {
      overlays = import ./nix/overlays { inherit self lib rust-overlay; };

      packages = lib.mapAttrs (system: pkgs: {
        default = self.packages.${system}.noita-proxy;
        inherit (pkgs) noita-proxy;
      }) pkgsFor;

      devShells = lib.mapAttrs
        (system: pkgs: { default = pkgs.callPackage ./nix/shell.nix { }; })
        pkgsFor;

      formatter =
        eachSystem (system: nixpkgs.legacyPackages.${system}.nixfmt-classic);
    };
}
