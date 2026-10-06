{
  description = "Tatami: a keyboard-driven Hyprland desktop for NixOS, inspired by Omarchy";
  inputs.nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";
  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      # A minimal system with Tatami, to check that the module evaluates.
      example = nixpkgs.lib.nixosSystem {
        inherit system;
        modules = [
          self.nixosModules.default
          {
            programs.tatami.enable = true;
            fileSystems."/" = {
              device = "/dev/disk/by-label/nixos";
              fsType = "ext4";
            };
            boot.loader.systemd-boot.enable = true;
            system.stateVersion = "26.11";
          }
        ];
      };
    in
    {
      # programs.tatami.enable adds the Tatami session to the login screen.
      nixosModules.default = ./module.nix;
      packages.${system} = {
        # The `tatami` helper the session and its menus run.
        tatami = pkgs.callPackage ./tool/package.nix { };
        default = self.packages.${system}.tatami;
      };
      checks.${system} = {
        tatami = self.packages.${system}.tatami;
        module = pkgs.writeText "tatami-example-system" (
          builtins.unsafeDiscardOutputDependency example.config.system.build.toplevel.drvPath
        );
      };
      formatter.${system} = pkgs.nixfmt;
    };
}
