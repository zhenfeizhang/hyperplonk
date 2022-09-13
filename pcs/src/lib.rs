mod errors;
mod multilinear_kzg;
mod structs;
mod univariate_kzg;

pub mod prelude;

use ark_ec::PairingEngine;
use ark_ff::Field;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::{CryptoRng, RngCore};
use errors::PCSError;
use std::{borrow::Borrow, fmt::Debug, hash::Hash};

/// This trait defines APIs for polynomial commitment schemes.
/// Note that for our usage of PCS, we do not require the hiding property.
pub trait PolynomialCommitmentScheme<E: PairingEngine> {
    /// Prover parameters
    type ProverParam: Clone;
    /// Verifier parameters
    type VerifierParam: Clone + CanonicalSerialize + CanonicalDeserialize;
    /// Structured reference string
    type SRS: Clone + Debug;
    /// Polynomial and its associated types
    type Polynomial: Clone
        + Debug
        + Hash
        + PartialEq
        + Eq
        + CanonicalSerialize
        + CanonicalDeserialize;
    /// Polynomial input domain
    type Point: Clone + Ord + Debug + Sync + Hash + PartialEq + Eq;
    /// Polynomial Evaluation
    type Evaluation: Field;
    /// Commitments
    type Commitment: Clone + CanonicalSerialize + CanonicalDeserialize + Debug + PartialEq + Eq;
    /// Batch commitments
    type BatchCommitment: Clone + CanonicalSerialize + CanonicalDeserialize + Debug + PartialEq + Eq;
    /// Proofs
    type Proof: Clone + CanonicalSerialize + CanonicalDeserialize + Debug + PartialEq + Eq;
    /// Batch proofs
    type BatchProof: Clone + CanonicalSerialize + CanonicalDeserialize + Debug + PartialEq + Eq;

    /// Build SRS for testing.
    ///
    /// - For univariate polynomials, `supported_size` is the maximum degree.
    /// - For multilinear polynomials, `supported_size` is the number of
    ///   variables.
    ///
    /// WARNING: THIS FUNCTION IS FOR TESTING PURPOSE ONLY.
    /// THE OUTPUT SRS SHOULD NOT BE USED IN PRODUCTION.
    fn gen_srs_for_testing<R: RngCore + CryptoRng>(
        rng: &mut R,
        supported_size: usize,
    ) -> Result<Self::SRS, PCSError>;

    /// Trim the universal parameters to specialize the public parameters.
    /// Input both `supported_degree` for univariate and
    /// `supported_num_vars` for multilinear.
    /// ## Note on function signature
    /// Usually, data structure like SRS and ProverParam are huge and users
    /// might wish to keep them in heap using different kinds of smart pointers
    /// (instead of only in stack) therefore our `impl Borrow<_>` interface
    /// allows for passing in any pointer type, e.g.: `trim(srs: &Self::SRS,
    /// ..)` or `trim(srs: Box<Self::SRS>, ..)` or `trim(srs: Arc<Self::SRS>,
    /// ..)` etc.
    fn trim(
        srs: impl Borrow<Self::SRS>,
        supported_degree: usize,
        supported_num_vars: Option<usize>,
    ) -> Result<(Self::ProverParam, Self::VerifierParam), PCSError>;

    /// Generate a commitment for a polynomial
    /// ## Note on function signature
    /// Usually, data structure like SRS and ProverParam are huge and users
    /// might wish to keep them in heap using different kinds of smart pointers
    /// (instead of only in stack) therefore our `impl Borrow<_>` interface
    /// allows for passing in any pointer type, e.g.: `commit(prover_param:
    /// &Self::ProverParam, ..)` or `commit(prover_param:
    /// Box<Self::ProverParam>, ..)` or `commit(prover_param:
    /// Arc<Self::ProverParam>, ..)` etc.
    fn commit(
        prover_param: impl Borrow<Self::ProverParam>,
        poly: &Self::Polynomial,
    ) -> Result<Self::Commitment, PCSError>;

    /// Generate a commitment for a list of polynomials
    fn multi_commit(
        prover_param: impl Borrow<Self::ProverParam>,
        polys: &[Self::Polynomial],
    ) -> Result<Self::BatchCommitment, PCSError>;

    /// On input a polynomial `p` and a point `point`, outputs a proof for the
    /// same.
    fn open(
        prover_param: impl Borrow<Self::ProverParam>,
        polynomial: &Self::Polynomial,
        point: &Self::Point,
    ) -> Result<(Self::Proof, Self::Evaluation), PCSError>;

    /// Input a list of multilinear extensions, and a same number of points, and
    /// a transcript, compute a multi-opening for all the polynomials.
    fn multi_open(
        prover_param: impl Borrow<Self::ProverParam>,
        multi_commitment: &Self::BatchCommitment,
        polynomials: &[Self::Polynomial],
        points: &[Self::Point],
    ) -> Result<(Self::BatchProof, Vec<Self::Evaluation>), PCSError>;

    /// Input a multilinear extension, and a number of points, and
    /// a transcript, compute a multi-opening for all the polynomials.
    fn multi_open_single_poly(
        _prover_param: impl Borrow<Self::ProverParam>,
        _commitment: &Self::Commitment,
        _polynomials: &Self::Polynomial,
        _points: &[Self::Point],
    ) -> Result<(Self::BatchProof, Vec<Self::Evaluation>), PCSError> {
        unimplemented!()
    }

    /// Verifies that `value` is the evaluation at `x` of the polynomial
    /// committed inside `comm`.
    fn verify(
        verifier_param: &Self::VerifierParam,
        commitment: &Self::Commitment,
        point: &Self::Point,
        value: &E::Fr,
        proof: &Self::Proof,
    ) -> Result<bool, PCSError>;

    /// Verifies that `value_i` is the evaluation at `x_i` of the polynomial
    /// `poly_i` committed inside `comm`.
    fn batch_verify<R: RngCore + CryptoRng>(
        verifier_param: &Self::VerifierParam,
        multi_commitment: &Self::BatchCommitment,
        points: &[Self::Point],
        values: &[E::Fr],
        batch_proof: &Self::BatchProof,
        rng: &mut R,
    ) -> Result<bool, PCSError>;

    /// Verifies that `value_i` is the evaluation at `x_i` of the polynomial
    /// `poly` committed inside `comm`.
    fn batch_verify_single_poly<R: RngCore + CryptoRng>(
        _verifier_param: &Self::VerifierParam,
        _commitment: &Self::Commitment,
        _points: &[Self::Point],
        _values: &[E::Fr],
        _batch_proof: &Self::BatchProof,
        _rng: &mut R,
    ) -> Result<bool, PCSError> {
        unimplemented!()
    }
}

/// API definitions for structured reference string
pub trait StructuredReferenceString<E: PairingEngine>: Sized {
    /// Prover parameters
    type ProverParam;
    /// Verifier parameters
    type VerifierParam;

    /// Extract the prover parameters from the public parameters.
    fn extract_prover_param(&self, supported_size: usize) -> Self::ProverParam;
    /// Extract the verifier parameters from the public parameters.
    fn extract_verifier_param(&self, supported_size: usize) -> Self::VerifierParam;

    /// Trim the universal parameters to specialize the public parameters
    /// for polynomials to the given `supported_size`, and
    /// returns committer key and verifier key.
    ///
    /// - For univariate polynomials, `supported_size` is the maximum degree.
    /// - For multilinear polynomials, `supported_size` is 2 to the number of
    ///   variables.
    ///
    /// `supported_log_size` should be in range `1..=params.log_size`
    fn trim(
        &self,
        supported_size: usize,
    ) -> Result<(Self::ProverParam, Self::VerifierParam), PCSError>;

    /// Build SRS for testing.
    ///
    /// - For univariate polynomials, `supported_size` is the maximum degree.
    /// - For multilinear polynomials, `supported_size` is the number of
    ///   variables.
    ///
    /// WARNING: THIS FUNCTION IS FOR TESTING PURPOSE ONLY.
    /// THE OUTPUT SRS SHOULD NOT BE USED IN PRODUCTION.
    fn gen_srs_for_testing<R: RngCore + CryptoRng>(
        rng: &mut R,
        supported_size: usize,
    ) -> Result<Self, PCSError>;
}