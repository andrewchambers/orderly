{stdenv, rustPlatform, fetchFromGitHub}:
rustPlatform.buildRustPackage rec {
  name = "orderly-${version}";
  
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "andrewchambers";
    repo = "orderly";
    rev = "v${version}";
    sha256 = "163vk93dzw3bj4ccs4ggqgv7sk2plddyihgkx7vsjnnxr5m10wp9";
  };

  cargoSha256 = "17wxa3i6zza0kgh0gnldyrqm5rz38ydsdvga8dhdrdkw923b02i3";

  postInstall = ''
    mkdir -p $out/share/man/man1
    cat ./man/generated/orderly.1 | gzip > $out/share/man/man1/orderly.1.gz
  '';

  meta = with stdenv.lib; {
    description = "Ordered process (re)start, shutdown, and supervision.";
    homepage = https://github.com/andrewchambers/orderly;
  };
}