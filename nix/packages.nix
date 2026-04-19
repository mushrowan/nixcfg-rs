# crane outputs: package, clippy, test, deny, doctest + schema drift check
{
  pkgs,
  craneLib,
  src,
}: let
  # workspace metadata; can't use `crateNameFromCargoToml` directly here
  # because the member Cargo.toml uses workspace inheritance which crane
  # doesn't resolve
  commonArgs = {
    inherit src;
    pname = "nixcfg-rs";
    version = "0.3.0";
    strictDeps = true;
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  # build the example binary on its own so we can run it in the schema check
  example = craneLib.buildPackage (commonArgs
    // {
      inherit cargoArtifacts;
      pname = "example-mycel";
      cargoExtraArgs = "--bin example-mycel";
      doCheck = false;
    });

  referenceSchema = ./../example-mycel/schema.json;
in {
  inherit cargoArtifacts example;

  package = craneLib.buildPackage (commonArgs
    // {
      inherit cargoArtifacts;
      doCheck = false;
    });

  clippy = craneLib.cargoClippy (commonArgs
    // {
      inherit cargoArtifacts;
      cargoClippyExtraArgs = "--all-targets -- --deny warnings";
    });

  test = craneLib.cargoNextest (commonArgs
    // {
      inherit cargoArtifacts;
    });

  deny = craneLib.cargoDeny (commonArgs
    // {
      inherit cargoArtifacts;
    });

  doctest = craneLib.cargoDocTest (commonArgs
    // {
      inherit cargoArtifacts;
    });

  # catches drift between the example Config struct and the checked-in
  # schema.json. if the struct changes, this check fails until someone
  # re-runs `cargo x update-schema`
  schemaCheck = pkgs.runCommand "nixcfg-rs-schema-check" {} ''
    ${pkgs.diffutils}/bin/diff -u ${referenceSchema} <(${example}/bin/example-mycel)
    touch $out
  '';
}
