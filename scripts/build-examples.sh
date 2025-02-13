#!/usr/bin/env bash

mkdir dist
for example in examples/*; do
  example_name=$(basename "$example")
  mkdir -p dist/"$example_name"
  pushd "$example" || exit
  if [ "$example_name" == "website" ]; then
    trunk build --release --public-url https://orhun.dev/ratzilla/
    cp -r dist/* ../../dist/
  elif [[ "$example_name" == "tauri"* ]]; then
    echo "Skipping Tauri example"
  else
    trunk build --release --public-url https://orhun.dev/ratzilla/"$example_name"
    cp -r dist/* ../../dist/"$example_name"
  fi
  popd || exit
done
