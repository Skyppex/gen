{
  src,
  naerskLib,
  pkg-config,
}:
naerskLib.buildPackage {
  name = "gen";
  src = src;
  nativeBuildInputs = [pkg-config];
}
