#!/bin/bash

if [ -z ${AOC_SESSION+x} ];
then
    echo "AOC_SESSION is unset";
    aocd --help
    exit 1
fi

if [ "$1" ]; then
    set -e

    source .env/bin/activate

    maturin develop
    cargo clippy
    cargo fmt

    aoce --example-parser=aoce_advent_plugin -y 2022

else
    cargo watch -c -s "./watch.sh doit"
fi
