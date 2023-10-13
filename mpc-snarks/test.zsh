#!/usr/bin/env zsh
set -xe
trap "exit" INT TERM
trap "kill 0" EXIT

cargo +nightly build --bin client

BIN=./target/debug/client

# KZG commit gsz 16 coeffs
$BIN --hosts data/3 kzgcommit 16 --party 0 & ; pid0=$!
$BIN --hosts data/3 kzgcommit 16 --party 1 & ; pid1=$!
$BIN --hosts data/3 kzgcommit 16 --party 2 & ; pid2=$!
wait $pid0 $pid1 $pid2

# KZG commit spdz 16 coeffs
$BIN --spdz --hosts data/3 kzgcommit 16 --party 0 & ; pid0=$!
$BIN --spdz --hosts data/3 kzgcommit 16 --party 1 & ; pid1=$!
$BIN --spdz --hosts data/3 kzgcommit 16 --party 2 & ; pid2=$!
wait $pid0 $pid1 $pid2

# KZG commit spdz 128 coeffs
$BIN --spdz --hosts data/3 kzgcommit 128 --party 0 & ; pid0=$!
$BIN --spdz --hosts data/3 kzgcommit 128 --party 1 & ; pid1=$!
$BIN --spdz --hosts data/3 kzgcommit 128 --party 2 & ; pid2=$!
wait $pid0 $pid1 $pid2

# KZG commit gsz 128 coeefs
$BIN --hosts data/3 kzgcommit 128 --party 0 & ; pid0=$!
$BIN --hosts data/3 kzgcommit 128 --party 1 & ; pid1=$!
$BIN --hosts data/3 kzgcommit 128 --party 2 & ; pid2=$!
wait $pid0 $pid1 $pid2

# msm spdz 128 coeffs
$BIN --spdz --hosts data/3 msm 128 --party 0 & ; pid0=$!
$BIN --spdz --hosts data/3 msm 128 --party 1 & ; pid1=$!
$BIN --spdz --hosts data/3 msm 128 --party 2 & ; pid2=$!
wait $pid0 $pid1 $pid2

# msm gsz 128 coeefs
$BIN --hosts data/3 msm 128 --party 0 & ; pid0=$!
$BIN --hosts data/3 msm 128 --party 1 & ; pid1=$!
$BIN --hosts data/3 msm 128 --party 2 & ; pid2=$!
wait $pid0 $pid1 $pid2

# # KZG commit (no blind)
# $BIN --spdz --hosts data/3 kzgcommit 1 2 0 4 4 1 2 0 4 4 1 2 0 4 4 --party 0 & ; pid0=$!
# $BIN --spdz --hosts data/3 kzgcommit 3 2 0 0 1 3 2 0 0 1 3 2 0 0 1 --party 1 & ; pid1=$!
# $BIN --spdz --hosts data/3 kzgcommit 3 2 0 0 1 3 2 0 0 1 3 2 0 0 1 --party 2 & ; pid2=$!
# wait $pid0 $pid1 $pid2

# # msm gsz
# $BIN --hosts data/2 msm 4 1 2 --party 0 & ; pid0=$!
# $BIN --hosts data/2 msm 0 1 2 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# # msm spdz
# $BIN --hosts data/2 msm 4 1 2 --party 0 & ; pid0=$!
# $BIN --hosts data/2 msm 0 1 2 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# $BIN --spdz --hosts data/2 -d kzgcommit 1 2 0 4 4 --party 0 & ; pid0=$!
# $BIN --spdz --hosts data/2 -d kzgcommit 3 2 0 0 1 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# $BIN --gsz --hosts data/3 -d kzgcommit 1 2 0 4 4 --party 0 & ; pid0=$!
# $BIN --gsz --hosts data/3 -d kzgcommit 1 2 0 4 4 --party 1 & ; pid1=$!
# $BIN --gsz --hosts data/3 -d kzgcommit 3 2 0 0 1 --party 2 & ; pid2=$!

# wait $pid0 $pid1

# $BIN --spdz --hosts data/2 -d sum 1 0 --party 0 & ; pid0=$!
# $BIN --spdz --hosts data/2 -d sum 0 1 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# $BIN --spdz --hosts data/2 -d product 1 3 --party 0 & ; pid0=$!
# $BIN --spdz --hosts data/2 -d product 2 1 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# $BIN --spdz --hosts data/2 -d pproduct 1 3 --party 0 & ; pid0=$!
# $BIN --spdz --hosts data/2 -d pproduct 2 1 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# $BIN --spdz --hosts data/2 -d polydiv 1 3 1 0 --party 0 & ; pid0=$!
# $BIN --spdz --hosts data/2 -d polydiv 0 0 2 1 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# $BIN --spdz --hosts data/2 -d dh 1 3 --party 0 & ; pid0=$!
# $BIN --spdz --hosts data/2 -d dh 0 0 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# $BIN --spdz --hosts data/2 -d groupops 1 3 --party 0 & ; pid0=$!
# $BIN --spdz --hosts data/2 -d groupops 5 0 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# # msm
# $BIN --hosts data/2 msm 4 1 2 --party 0 & ; pid0=$!
# $BIN --hosts data/2 msm 0 1 2 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# $BIN --hosts data/2 -d sum 1 0 --party 0 & ; pid0=$!
# $BIN --hosts data/2 -d sum 0 1 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# $BIN --hosts data/2 -d product 1 0 --party 0 & ; pid0=$!
# $BIN --hosts data/2 -d product 0 1 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# $BIN --hosts data/2 -d pproduct 2 3 --party 0 & ; pid0=$!
# $BIN --hosts data/2 -d pproduct 1 2 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# # $BIN --hosts data/2 -d commit 1 0 --party 0 & ; pid0=$!
# # $BIN --hosts data/2 -d commit 0 1 --party 1 & ; pid1=$!
# # 
# # wait $pid0 $pid1
# # 
# # $BIN --hosts data/2 -d merkle 1 2 3 4 --party 0 & ; pid0=$!
# # $BIN --hosts data/2 -d merkle 0 0 0 0 --party 1 & ; pid1=$!
# # 
# # wait $pid0 $pid1
# # 
# # $BIN --hosts data/2 -d fri 2 2 1 7 --party 0 & ; pid0=$!
# # $BIN --hosts data/2 -d fri 0 0 0 0 --party 1 & ; pid1=$!
# # 
# # wait $pid0 $pid1

# # sum-check (G1)
# $BIN --hosts data/2 -d dh 0 4 6 --party 0 & ; pid0=$!
# $BIN --hosts data/2 -d dh 1 2 1 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# # sum-check (G2)
# $BIN --hosts data/2 -d dh 0 4 6 --party 0 --use-g2 & ; pid0=$!
# $BIN --hosts data/2 -d dh 1 2 1 --party 1 --use-g2 & ; pid1=$!

# wait $pid0 $pid1

# # DDH triple check (pairing)
# $BIN --hosts data/2 -d pairingdh 0 1 6 --party 0 & ; pid0=$!
# $BIN --hosts data/2 -d pairingdh 2 2 0 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# # lin-check (pairing)
# $BIN --hosts data/2 -d pairingprod 0 1 6 1 --party 0 & ; pid0=$!
# $BIN --hosts data/2 -d pairingprod 2 2 0 1 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# # lin-check (pairing)
# $BIN --hosts data/2 -d pairingdiv 0 1 6 1 --party 0 & ; pid0=$!
# $BIN --hosts data/2 -d pairingdiv 2 2 0 1 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# # groth16
# $BIN --hosts data/2 groth16 --party 0 & ; pid0=$!
# $BIN --hosts data/2 groth16 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# # poly eval
# $BIN --hosts data/2 polyeval 1 2 --party 0 & ; pid0=$!
# $BIN --hosts data/2 polyeval 3 2 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# # KZG commit (no blind)
# $BIN --hosts data/2 kzg 1 2 0 4 4 --party 0 & ; pid0=$!
# $BIN --hosts data/2 kzg 3 2 0 0 1 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# # KZG commit (zk)
# $BIN --hosts data/2 kzgzk 1 2 0 4 4 --party 0 & ; pid0=$!
# $BIN --hosts data/2 kzgzk 3 2 0 0 1 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# # KZG commit (zk, batch verify)
# $BIN --hosts data/2 kzgzkbatch 1 2 0 4 4 0 --party 0 & ; pid0=$!
# $BIN --hosts data/2 kzgzkbatch 3 2 0 0 1 0 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# # poly commit
# $BIN --hosts data/2 marlinpc 0 0 --party 0 & ; pid0=$!
# $BIN --hosts data/2 marlinpc 0 0 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# # marlin poly commit (zk, batch verify)
# $BIN --hosts data/2 marlinpcbatch 1 2 0 4 4 0 --party 0 & ; pid0=$!
# $BIN --hosts data/2 marlinpcbatch 3 2 0 0 1 0 --party 1 & ; pid1=$!

# wait $pid0 $pid1

# # plonk
# $BIN --hosts data/2 plonk --party 0 & ; pid0=$!
# $BIN --hosts data/2 plonk --party 1 & ; pid1=$!

# wait $pid0 $pid1

# # marlin
# $BIN --hosts data/2 marlin --party 0 & ; pid0=$!
# $BIN --hosts data/2 marlin --party 1 & ; pid1=$!

# wait $pid0 $pid1

#trap - INT TERM EXIT

#./bench_test.zsh
