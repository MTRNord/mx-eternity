#!/bin/sh

rustc ./src/lib.rs -o test-plugin.wasm  --target=wasm32-unknown-unknown