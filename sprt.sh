#!/usr/bin/env sh

if [ -z "$1" ]; then
    echo "Branch name required"
    exit 1
fi

git checkout "$1"
cargo build --release
cp target/release/minuette "minuette-$1"

git checkout main
cargo build --release
cp target/release/minuette "minuette-main"

cutechess-cli \
    -each proto=uci tc=8+0.08 dir=. option.Hash=16 option.Threads=1 \
    -openings file=4moves_noob.epd format=epd order=random -repeat \
    -games 2 -concurrency 4 \
    -rounds 20000 -sprt alpha=0.05 beta=0.1 elo0=0 elo1=10 \
    -ratinginterval 0 \
    -engine name="minuette-$1" cmd="./minuette-$1" \
    -engine name=minuette-main cmd=./minuette-main

