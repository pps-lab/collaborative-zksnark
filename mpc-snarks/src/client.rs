#![feature(trait_alias)]
//! Mostly just for testing
use ark_std::{start_timer, end_timer, test_rng, UniformRand};
use log::debug;

use ark_bls12_377::{Fr, FqParameters};
use ark_ec::{group::Group, AffineCurve, PairingEngine, ProjectiveCurve};
use ark_ff::{Field, PrimeField, Fp256, BigInteger256, BigInteger};
use ark_poly::domain::radix2::Radix2EvaluationDomain;
use ark_poly::univariate::DensePolynomial;
use ark_poly::{EvaluationDomain, Polynomial, UVPolynomial};
use ark_poly_commit::marlin_pc;
use ark_poly_commit::PolynomialCommitment;
use ark_serialize::CanonicalSerialize;
use ark_std::rand::SeedableRng;
use rand::{Rng, random};
use std::borrow::Cow;
use std::path::PathBuf;
use std::vec;
use ark_sponge::{ CryptographicSponge, FieldBasedCryptographicSponge, poseidon::PoseidonSponge, poseidon::PoseidonParameters};
use ark_sponge::poseidon_parameters_for_test;
use mpc_algebra::com::ComField;
use mpc_algebra::honest_but_curious as hbc;
use mpc_algebra::malicious_majority as mm;
use mpc_algebra::honest_majority as hm;
use mpc_algebra::*;
use mpc_trait::MpcWire;
use mpc_net::{MpcNet, MpcMultiNet};

use clap::arg_enum;
use merlin::Transcript;
use structopt::StructOpt;

mod groth;
mod marlin;
mod plonk;
mod silly;

pub type PoseidonParam<F> = PoseidonParameters<F>;
pub type SPNGFunction<F> = PoseidonSponge<F>;
pub type SPNGOutput<F> = Vec<F>;
pub type SPNGParam<F> = <SPNGFunction<F> as CryptographicSponge>::Parameters;

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum Computation {
        Fft,
        Sum,
        Product,
        PProduct,
        Commit,
        Merkle,
        Fri,
        Dh,
        NaiveMsm,
        GroupOps,
        PairingDh,
        PairingProd,
        PairingDiv,
        Groth16,
        Marlin,
        PolyEval,
        MarlinPc,
        MarlinPcBatch,
        Msm,
        Kzg,
        KzgCommit,
        KzgZk,
        KzgZkBatch,
        PcTwoCom,
        Plonk,
        PolyDiv,
        Ecdsa
    }
}

#[derive(PartialEq, Debug)]
enum ComputationDomain {
    G1,
    G2,
    Group,
    Field,
    Pairing,
    BlsPairing,
    PolyField,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "client", about = "An example MPC")]
struct Opt {
    /// Activate debug mode
    // short and long flags (-d, --debug) will be deduced from the field's name
    #[structopt(short, long)]
    debug: bool,

    /// File with list of hosts
    #[structopt(long, parse(from_os_str))]
    hosts: PathBuf,

    /// Which party are you? 0 or 1?
    #[structopt(long, default_value = "0")]
    party: u8,

    /// Computation to perform
    #[structopt()]
    computation: Computation,

    /// Computation to perform
    #[structopt(long)]
    use_g2: bool,

    /// Computation to perform
    #[structopt(long)]
    spdz: bool,

    /// Input a
    #[structopt()]
    args: u64,
}

impl Opt {
    fn domain(&self) -> ComputationDomain {
        match &self.computation {
            Computation::Dh => {
                if self.use_g2 {
                    ComputationDomain::G2
                } else {
                    ComputationDomain::G1
                }
            }
            Computation::NaiveMsm | Computation::GroupOps => {
                ComputationDomain::Group
            }
            Computation::PairingDh | Computation::PairingProd | Computation::PairingDiv => {
                ComputationDomain::Pairing
            }
            Computation::Marlin
            | Computation::Groth16
            | Computation::Plonk
            | Computation::Kzg
            | Computation::KzgCommit
            | Computation::KzgZk
            | Computation::Msm
            | Computation::KzgZkBatch
            | Computation::MarlinPc
            | Computation::Ecdsa
            | Computation::MarlinPcBatch => ComputationDomain::BlsPairing,
            Computation::PolyEval => ComputationDomain::PolyField,
            _ => ComputationDomain::Field,
        }
    }
}

fn ecdsa_gsz (
    x: MFrg, k: MFrg, p: <MEg as PairingEngine>::G1Projective, m: &[u8], params: SPNGParam<Fr>
) -> MFrg {
    let R = p.scalar_mul(&k);

    let (x_coord, y_coord) = match R.val{
        MpcGroup::Public(cp) => (cp.x, cp.y),
        MpcGroup::Shared(cp) => (cp.val.x, cp.val.y),
    };
    let x_coord_bytes = x_coord.into_repr().to_bytes_le();
    let r = MFrg::from_le_bytes_mod_order(&x_coord_bytes);
    let mut sponge = PoseidonSponge::< >::new(&params);
    sponge.absorb(&m);
    let e: Fr = sponge.squeeze_native_field_elements(1)[0];
    let rng = &mut test_rng();
    let e_share = MFrg::Public(e);
    let xr = x * r;
    let mut s = e_share + xr;
    s = s * k.inv().unwrap();
    s
}

fn ecdsa_spdz (
    x: MFrs, k: MFrs, p: <MEs as PairingEngine>::G1Projective, m: &[u8], params: SPNGParam<Fr>
) -> MFrs {
    let R = p.scalar_mul(&k);

    let (x_coord, y_coord) = match R.val{
        MpcGroup::Public(cp) => (cp.x, cp.y),
        MpcGroup::Shared(cp) => (cp.sh.val.x, cp.sh.val.y),
    };
    let x_coord_bytes = x_coord.into_repr().to_bytes_le();
    let r = MFrs::from_le_bytes_mod_order(&x_coord_bytes);
    let mut sponge = PoseidonSponge::< >::new(&params);
    sponge.absorb(&m);
    let e: Fr = sponge.squeeze_native_field_elements(1)[0];
    let xr = x * r;
    let rng = &mut test_rng();
    let e_share = MFrs::Public(e);
    let mut s = e_share + xr;
    s = s * k.inv().unwrap();
    s
}

fn powers_to_mpc<'a, P: PairingShare<ark_bls12_377::Bls12_377>>(
    p: ark_poly_commit::kzg10::Powers<'a, ark_bls12_377::Bls12_377>,
) -> ark_poly_commit::kzg10::Powers<'a, MpcPairingEngine<ark_bls12_377::Bls12_377, P>>{
    ark_poly_commit::kzg10::Powers {
        powers_of_g: Cow::Owned(
            p.powers_of_g
                .iter()
                .cloned()
                .map(MpcG1Affine::from_public)
                .collect(),
        ),
        powers_of_gamma_g: Cow::Owned(
            p.powers_of_gamma_g
                .iter()
                .cloned()
                .map(MpcG1Affine::from_public)
                .collect(),
        ),
    }
}


// fn commit_from_mpc<'a>(
//     p: ark_poly_commit::kzg10::Commitment<hbc::MpcPairingEngine<ark_bls12_377::Bls12_377>>,
// ) -> ark_poly_commit::kzg10::Commitment<ark_bls12_377::Bls12_377> {
//     ark_poly_commit::kzg10::Commitment(p.0.reveal())
// }
// fn commit_from_mpc_spdz<'a>(
//     p: ark_poly_commit::kzg10::Commitment<mm::MpcPairingEngine<ark_bls12_377::Bls12_377>>,
// ) -> ark_poly_commit::kzg10::Commitment<ark_bls12_377::Bls12_377> {
//     ark_poly_commit::kzg10::Commitment(p.0.reveal())
// }
fn pf_from_mpc<'a>(
    pf: ark_poly_commit::kzg10::Proof<hbc::MpcPairingEngine<ark_bls12_377::Bls12_377>>,
) -> ark_poly_commit::kzg10::Proof<ark_bls12_377::Bls12_377> {
    ark_poly_commit::kzg10::Proof {
        w: pf.w.reveal(),
        random_v: pf.random_v.map(MpcField::reveal),
    }
}

impl Computation {
    fn run_gsz(&self, inputs: Vec<MFrg>) -> Vec<MFrg> {
        let outputs: Vec<MFrg> = match self {
            Computation::KzgCommit => {
                //let timer = start_timer!(|| "KZG Commitment");
                //let ipt_reveal: Vec<Fr> = inputs.iter().map(|x| x.reveal()).collect();
                //println!("REVEALED VALUES{:?}", ipt_reveal);
                let poly = MPg::from_coefficients_slice(&inputs);
                let rng = &mut ark_std::test_rng();
                let pp = ark_poly_commit::kzg10::KZG10::<
                    ark_bls12_377::Bls12_377,
                    ark_poly::univariate::DensePolynomial<ark_bls12_377::Fr>,
                >::setup(inputs.len() - 1, true, rng)
                .unwrap();
                let powers_of_gamma_g = (0..inputs.len())
                    .map(|i| pp.powers_of_gamma_g[&i])
                    .collect::<Vec<_>>();
                let powers = ark_poly_commit::kzg10::Powers::<ark_bls12_377::Bls12_377> {
                    powers_of_g: Cow::Borrowed(&pp.powers_of_g),
                    powers_of_gamma_g: Cow::Owned(powers_of_gamma_g),
                };
                let mpc_powers = powers_to_mpc::<GszPairingShare<ark_bls12_377::Bls12_377>>(powers);
                let (commit, rand) =
                    ark_poly_commit::kzg10::KZG10::commit(&mpc_powers, &poly, None, None).unwrap();
                //println!("{:?}", commit);
                // let commit = commit_from_mpc_spdz(commit);
                // println!("{:?}", commit);
                //end_timer!(timer);
                vec![]
            }

            Computation::Msm => {
                let rng = &mut rand::rngs::StdRng::from_seed([0u8; 32]);
                let ps: Vec<MFrg> = (0..inputs.len()).map(|_| MFrg::public_rand(rng)).collect();
                //let sum: MFrg = inputs.iter().zip(ps.iter()).map(|(a, b)| *a * b).sum();
                let mut public_gens =
                    vec![<MEg as PairingEngine>::G1Affine::prime_subgroup_generator(); inputs.len()];
                for (g, c) in public_gens.iter_mut().zip(ps.iter()) {
                    *g = g.scalar_mul(*c).into();
                }
                let mut msm =
                    <MEg as PairingEngine>::G1Affine::multi_scalar_mul(&public_gens, &inputs);
                // let mut expected = <MEg as PairingEngine>::G1Projective::prime_subgroup_generator()
                //     .scalar_mul(&sum);
                //msm.publicize();
                // expected.publicize();

                //println!("{}", msm);
                // assert_eq!(msm, expected);
                vec![]
            }

            Computation::Ecdsa => {
                let rng = &mut test_rng();

                let random_bytes: Vec<Vec<u8>> = (0..inputs.len()).map(|_| (0..16).map(|_| rng.gen()).collect()).collect();
                let parameters : SPNGParam<Fr> = poseidon_parameters_for_test();
                for bytes in random_bytes {
                    ecdsa_gsz(MFrg::king_share(Fr::rand(rng), rng), 
                                MFrg::king_share(Fr::rand(rng), rng), 
                                <MEg as PairingEngine>::G1Projective::prime_subgroup_generator(),
                                 &bytes, parameters.clone());
                }
                vec![]
            }

            c => unimplemented!("Cannot run_dh {:?}", c),
        };
        // println!("Outputs:");
        // for (i, v) in outputs.iter().enumerate() {
        //     println!("  {}: {}", i, v);
        // }
        outputs
    }
    fn run_spdz(&self, inputs: Vec<MFrs>) -> Vec<MFrs> {
        let outputs: Vec<MFrs> = match self {
            Computation::KzgCommit => {
                //let timer = start_timer!(|| "KZG Commitment");
                //let ipt_reveal: Vec<Fr> = inputs.iter().map(|x| x.reveal()).collect();
                //println!("REVEALED VALUES{:?}", ipt_reveal);
                let poly = MPs::from_coefficients_slice(&inputs);
                let rng = &mut ark_std::test_rng();
                let pp = ark_poly_commit::kzg10::KZG10::<
                    ark_bls12_377::Bls12_377,
                    ark_poly::univariate::DensePolynomial<ark_bls12_377::Fr>,
                >::setup(inputs.len() - 1, true, rng)
                .unwrap();
                let powers_of_gamma_g = (0..inputs.len())
                    .map(|i| pp.powers_of_gamma_g[&i])
                    .collect::<Vec<_>>();
                let powers = ark_poly_commit::kzg10::Powers::<ark_bls12_377::Bls12_377> {
                    powers_of_g: Cow::Borrowed(&pp.powers_of_g),
                    powers_of_gamma_g: Cow::Owned(powers_of_gamma_g),
                };
                let mpc_powers = powers_to_mpc::<SpdzPairingShare<ark_bls12_377::Bls12_377>>(powers);
                let (commit, rand) =
                    ark_poly_commit::kzg10::KZG10::commit(&mpc_powers, &poly, None, None).unwrap();
                //println!("{:?}", commit);
                // let commit = commit_from_mpc_spdz(commit);
                // println!("{:?}", commit);
                //end_timer!(timer);
                vec![]
            }

            Computation::Msm => {
                let rng = &mut rand::rngs::StdRng::from_seed([0u8; 32]);
                let ps: Vec<MFrs> = (0..inputs.len()).map(|_| MFrs::public_rand(rng)).collect();
                // let sum: MFrs = inputs.iter().zip(ps.iter()).map(|(a, b)| *a * b).sum();
                let mut public_gens =
                    vec![<MEs as PairingEngine>::G1Affine::prime_subgroup_generator(); inputs.len()];
                for (g, c) in public_gens.iter_mut().zip(ps.iter()) {
                    *g = g.scalar_mul(*c).into();
                }
                let mut msm =
                    <MEs as PairingEngine>::G1Affine::multi_scalar_mul(&public_gens, &inputs);
                // let mut expected = <MEs as PairingEngine>::G1Projective::prime_subgroup_generator()
                //     .scalar_mul(&sum);
                // msm.publicize();
                // expected.publicize();

                // //println!("{}", msm);
                // assert_eq!(msm, expected);
                vec![]
            }

            Computation::Ecdsa => {
                let rng = &mut test_rng();
                let random_bytes: Vec<Vec<u8>> = (0..inputs.len()).map(|_| (0..16).map(|_| rng.gen()).collect()).collect();
                let parameters : SPNGParam<Fr> = poseidon_parameters_for_test();
                for bytes in random_bytes {
                    ecdsa_spdz(MFrs::king_share(Fr::rand(rng), rng), 
                                MFrs::king_share(Fr::rand(rng), rng), 
                                <MEs as PairingEngine>::G1Projective::prime_subgroup_generator(),
                                 &bytes, parameters.clone());
                }
                vec![]
            }


            c => unimplemented!("Cannot run_dh {:?}", c),
        };
        // println!("Outputs:");
        // for (i, v) in outputs.iter().enumerate() {
        //     println!("  {}: {}", i, v);
        // }
        outputs
    }
}

type E = ark_bls12_377::Bls12_377;
type MEs = mm::MpcPairingEngine<E>;
type MEg = hm::MpcPairingEngine<E>;
type MFrs = mm::MpcField<Fr>;
type MFrg = hm::MpcField<Fr>;
type P = ark_poly::univariate::DensePolynomial<Fr>;
type MPs = ark_poly::univariate::DensePolynomial<MFrs>;
type MPg = ark_poly::univariate::DensePolynomial<MFrg>;
trait Pc = ark_poly_commit::PolynomialCommitment<Fr, DensePolynomial<Fr>>;
type MarlinPc = marlin_pc::MarlinKZG10<E, P>;

fn main() -> () {
    let opt = Opt::from_args();
    if opt.debug {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::init();
    }
    let domain = opt.domain();
    MpcMultiNet::init_from_file(opt.hosts.to_str().unwrap(), opt.party as usize);
    debug!("Start");
    if opt.spdz {
        let mut rng = test_rng();
        let inputs: Vec<MFrs> = (0..opt.args).map(|_| MFrs::king_share(Fr::rand(&mut rng), &mut rng)).collect();
        // let inputs = opt
        //     .args
        //     .iter()
        //     .map(|i| mm::MpcField::<Fr>::from_add_shared(Fr::from(*i)))
        //     .collect::<Vec<_>>();
        // println!("Inputs:");
        // for (i, v) in inputs.iter().enumerate() {
        //     println!("  {}: {}", i, v);
        // }
        match domain {
            ComputationDomain::BlsPairing => {
                //let timer = start_timer!(|| "spdz for kzg");
                let mut outputs = opt.computation.run_spdz(inputs);
                //end_timer!(timer);
                outputs.iter_mut().for_each(|c| c.publicize());
                // println!("Public Outputs:");
                // for (i, v) in outputs.iter().enumerate() {
                //     println!("  {}: {}", i, v);
                // }
            }
            // ComputationDomain::Group | ComputationDomain::G1 => {
            //     let generator = mm::MpcGroup::<ark_bls12_377::G1Projective>::from_public(ark_bls12_377::G1Projective::prime_subgroup_generator());
            //     opt.computation.run_group::<mm::MpcGroup<ark_bls12_377::G1Projective>>(inputs, generator);
            // }
            // ComputationDomain::BlsPairing => {
            //     let mut outputs = opt.computation.run_spdz(inputs);
            //     outputs.iter_mut().for_each(|c| c.publicize());
            //     println!("Public Outputs:");
            //     for (i, v) in outputs.iter().enumerate() {
            //         println!("  {}: {}", i, v);
            //     }
            // }
            d => panic!("Bad domain: {:?}", d),
        }
    } else {
        let mut rng = test_rng();
        let inputs: Vec<MFrg> = (0..opt.args).map(|_| MFrg::king_share(Fr::rand(&mut rng), &mut rng)).collect();
        // println!("Inputs:");
        // for (i, v) in inputs.iter().enumerate() {
        //     println!("  {}: {}", i, v);
        // }
        match domain {   
            ComputationDomain::BlsPairing => {
                //let timer = start_timer!(|| "gsz for kzg");
                let mut outputs = opt.computation.run_gsz(inputs);
                //end_timer!(timer);
                outputs.iter_mut().for_each(|c| c.publicize());
                // println!("Public Outputs:");
                // for (i, v) in outputs.iter().enumerate() {
                //     println!("  {}: {}", i, v);
                // }
            }
            d => panic!("Bad domain: {:?}", d),
        }
    }

    debug!("Stats: {:#?}", MpcMultiNet::stats());
    MpcMultiNet::deinit();
    debug!("Done");
}
