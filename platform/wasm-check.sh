#!/usr/bin/env bash
# E7: does each candidate compile for wasm32-unknown-unknown? Runs in a
# throwaway container (adds the target there, never on the host).
#   docker run --rm -v $PWD:/p:ro ghcr.io/hypertidy/gdal-r-python-extras:latest bash /p/wasm-check.sh
set -u
export RUSTUP_HOME=/opt/rustup CARGO_HOME=/tmp/cargo PATH=/opt/cargo/bin:$PATH
rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || { echo "cannot add target"; exit 1; }
check() { # name, dependency line
  d=/tmp/w/$1; mkdir -p $d/src
  printf '[package]\nname="w"\nversion="0.0.0"\nedition="2021"\n[lib]\ncrate-type=["cdylib","rlib"]\n[dependencies]\n%s\n' "$2" > $d/Cargo.toml
  echo "" > $d/src/lib.rs
  if out=$(cd $d && cargo build --release --target wasm32-unknown-unknown 2>&1); then
    echo "OK   $1 :: $2"
  else
    echo "FAIL $1 :: $2 :: $(echo "$out" | grep -E '^error|error:' | head -2 | tr '\n' ' ' | cut -c1-300)"
  fi
}
check proj4rs-default        'proj4rs = "=0.2.1"'
check proj4rs-nodefault      'proj4rs = { version = "=0.2.1", default-features = false }'
check proj4wkt               'proj4wkt = "=0.1.1"'
check crs-definitions        'crs-definitions = "=0.6.0"'
check geodesy-default        'geodesy = "=0.15.0"'
check geodesy-js             'geodesy = { version = "=0.15.0", default-features = false, features = ["js"] }'
check miniproj               'miniproj = "=0.10.4"'
check proj-core-default      'proj-core = "=0.11.0"'
check proj-core-nodefault    'proj-core = { version = "=0.11.0", default-features = false }'
check proj-wkt               'proj-wkt = "=0.11.0"'
check wbprojection-default   'wbprojection = "=0.3.3"'
check wbprojection-nodefault 'wbprojection = { version = "=0.3.3", default-features = false }'
check proj-georust           'proj = { version = "=0.31.0", default-features = false }'
