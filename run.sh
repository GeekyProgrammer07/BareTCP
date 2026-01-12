#!/bin/sh
cargo b --release
sudo setcap cap_net_admin=eip $CARGO_TARGET_DIR/release/BareTCP

$CARGO_TARGET_DIR/release/BareTCP &
pid=$! #! PID of last background process

# trap lets you: “Before exiting, run this.”
# 2>/dev/null: ignore error messages puts in void
# INT: Ctrl+C -> Interruption
# TERM: Termination signal (kill)
# EXIT: script ends for any reason
trap "kill $pid 2>/dev/null" INT TERM EXIT

sudo ip addr add 10.200.0.1/24 dev tun0
sudo ip link set up dev tun0

wait $pid # Don't end until BareTCP finishes
