use crate::poseidon::{PoseidonParameters, PoseidonSponge};
use crate::{absorb, collect_sponge_bytes, collect_sponge_field_elements, poseidon_parameters_for_test};
use crate::{Absorb, AbsorbWithLength, CryptographicSponge, FieldBasedCryptographicSponge};
use ark_ff::{One, PrimeField, UniformRand};
use ark_std::test_rng;
use ark_test_curves::bls12_381::Fr;
fn assert_different_encodings<F: PrimeField, A: Absorb>(a: &A, b: &A) {
    let bytes1 = a.to_sponge_bytes_as_vec();
    let bytes2 = b.to_sponge_bytes_as_vec();
    assert_ne!(bytes1, bytes2);

    let sponge_param = poseidon_parameters_for_test();
    let mut sponge1 = PoseidonSponge::<F>::new(&sponge_param);
    let mut sponge2 = PoseidonSponge::<F>::new(&sponge_param);

    sponge1.absorb(&a);
    sponge2.absorb(&b);

    assert_ne!(
        sponge1.squeeze_native_field_elements(3),
        sponge2.squeeze_native_field_elements(3)
    );
}

#[test]
fn single_field_element() {
    let mut rng = test_rng();
    let elem1 = Fr::rand(&mut rng);
    let elem2 = elem1 + Fr::one();

    assert_different_encodings::<Fr, _>(&elem1, &elem2)
}

#[test]
fn list_with_constant_size_element() {
    let mut rng = test_rng();
    let lst1: Vec<_> = (0..1024 * 8).map(|_| Fr::rand(&mut rng)).collect();
    let mut lst2 = lst1.to_vec();
    lst2[3] += Fr::one();

    assert_different_encodings::<Fr, _>(&lst1, &lst2)
}

struct VariableSizeList(Vec<u8>);

impl Absorb for VariableSizeList {
    fn to_sponge_bytes(&self, dest: &mut Vec<u8>) {
        self.0.to_sponge_bytes_with_length(dest)
    }

    fn to_sponge_field_elements<F: PrimeField>(&self, dest: &mut Vec<F>) {
        self.0.to_sponge_field_elements_with_length(dest)
    }
}

#[test]
fn list_with_nonconstant_size_element() {
    let lst1 = vec![
        VariableSizeList(vec![1u8, 2, 3, 4]),
        VariableSizeList(vec![5, 6]),
    ];
    let lst2 = vec![
        VariableSizeList(vec![1u8, 2]),
        VariableSizeList(vec![3, 4, 5, 6]),
    ];

    assert_different_encodings::<Fr, _>(&lst1, &lst2);
}

#[test]
fn test_squeeze_cast_native() {
    let mut rng = test_rng();
    let sponge_param = poseidon_parameters_for_test();
    let elem = Fr::rand(&mut rng);
    let mut sponge1 = PoseidonSponge::<Fr>::new(&sponge_param);
    sponge1.absorb(&elem);
    let mut sponge2 = sponge1.clone();

    // those two should return same result
    let squeezed1 = sponge1.squeeze_native_field_elements(5);
    let squeezed2 = sponge2.squeeze_field_elements::<Fr>(5);

    assert_eq!(squeezed1, squeezed2);
}

#[test]
fn test_macros() {
    let sponge_param = poseidon_parameters_for_test();
    let mut sponge1 = PoseidonSponge::<Fr>::new(&sponge_param);
    sponge1.absorb(&vec![1, 2, 3, 4, 5, 6]);
    sponge1.absorb(&Fr::from(114514u128));

    let mut sponge2 = PoseidonSponge::<Fr>::new(&sponge_param);
    absorb!(&mut sponge2, vec![1, 2, 3, 4, 5, 6], Fr::from(114514u128));

    let expected = sponge1.squeeze_native_field_elements(3);
    let actual = sponge2.squeeze_native_field_elements(3);

    assert_eq!(actual, expected);

    let mut expected = Vec::new();
    vec![6, 5, 4, 3, 2, 1].to_sponge_bytes(&mut expected);
    Fr::from(42u8).to_sponge_bytes(&mut expected);

    let actual = collect_sponge_bytes!(vec![6, 5, 4, 3, 2, 1], Fr::from(42u8));

    assert_eq!(actual, expected);

    let mut expected: Vec<Fr> = Vec::new();
    vec![6, 5, 4, 3, 2, 1].to_sponge_field_elements(&mut expected);
    Fr::from(42u8).to_sponge_field_elements(&mut expected);

    let actual: Vec<Fr> = collect_sponge_field_elements!(vec![6, 5, 4, 3, 2, 1], Fr::from(42u8));

    assert_eq!(actual, expected);
}

