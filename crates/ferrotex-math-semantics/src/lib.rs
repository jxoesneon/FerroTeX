use serde::{Deserialize, Serialize};

pub mod analysis;
pub mod delimiters;

/// Represents the dimensionality and size of a mathematical object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shape {
    /// A scalar value (0-dimensional).
    Scalar,
    /// A column vector of size `n`.
    Vector(Dimension),
    /// A matrix of size `rows x cols`.
    Matrix { rows: Dimension, cols: Dimension },
    /// A higher-order tensor with specified dimensions.
    Tensor(Vec<Dimension>),
    /// The shape is unknown or could not be inferred.
    Unknown,
    /// The object has an inconsistent shape (e.g., a matrix with rows of defined but differing lengths).
    Invalid(String),
}

/// Represents a single dimension size, which can be concrete or symbolic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dimension {
    /// A known integer size (e.g., 3).
    Finite(usize),
    /// A symbolic size (e.g., "n").
    Symbolic(String),
    /// An unknown size.
    Unknown,
}

impl Shape {
    /// Returns true if this shape expects to be compatible with another for addition.
    pub fn is_compatible_add(&self, other: &Shape) -> bool {
        match (self, other) {
            (Shape::Scalar, Shape::Scalar) => true,
            (Shape::Vector(d1), Shape::Vector(d2)) => d1 == d2,
            (Shape::Matrix { rows: r1, cols: c1 }, Shape::Matrix { rows: r2, cols: c2 }) => {
                r1 == r2 && c1 == c2
            }
            // Scalars can sometimes be broadcast, but strict math usually forbids "Matrix + Scalar"
            // For now, let's assume strictness.
            _ => false,
        }
    }

    /// Returns true if this shape is compatible with another for multiplication (A * B).
    pub fn is_compatible_mul(&self, other: &Shape) -> bool {
        match (self, other) {
            (Shape::Scalar, _) => true,
            (_, Shape::Scalar) => true,
            (Shape::Matrix { cols, .. }, Shape::Matrix { rows, .. }) => cols == rows,
            (Shape::Matrix { cols, .. }, Shape::Vector(rows)) => cols == rows,
            // Vector * Matrix is usually row-vector * matrix, which implies transposition.
            // Strict interpretation: Vector is column vector (N x 1).
            // So Vector * Matrix is (N x 1) * (R x C) -> mismatch unless 1 == R (row vector).
            _ => false,
        }
    }
}
