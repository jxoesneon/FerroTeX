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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_compatibility_add() {
        let scalar = Shape::Scalar;
        let vec1 = Shape::Vector(Dimension::Finite(3));
        let vec2 = Shape::Vector(Dimension::Finite(3));
        let vec3 = Shape::Vector(Dimension::Finite(4));
        let mat1 = Shape::Matrix {
            rows: Dimension::Symbolic("n".into()),
            cols: Dimension::Finite(3),
        };
        let mat2 = Shape::Matrix {
            rows: Dimension::Symbolic("n".into()),
            cols: Dimension::Finite(3),
        };

        assert!(scalar.is_compatible_add(&scalar));
        assert!(vec1.is_compatible_add(&vec2));
        assert!(!vec1.is_compatible_add(&vec3));
        assert!(mat1.is_compatible_add(&mat2));
        assert!(!mat1.is_compatible_add(&scalar));
    }

    #[test]
    fn test_shape_compatibility_mul() {
        let scalar = Shape::Scalar;
        let vec = Shape::Vector(Dimension::Finite(3));
        let mat = Shape::Matrix {
            rows: Dimension::Finite(5),
            cols: Dimension::Finite(3),
        };
        let mat_bad = Shape::Matrix {
            rows: Dimension::Finite(4),
            cols: Dimension::Finite(5),
        };

        assert!(scalar.is_compatible_mul(&mat));
        assert!(mat.is_compatible_mul(&scalar));
        assert!(mat.is_compatible_mul(&Shape::Matrix {
            rows: Dimension::Finite(3),
            cols: Dimension::Finite(2)
        }));
        assert!(!mat.is_compatible_mul(&mat_bad));
        assert!(mat.is_compatible_mul(&vec)); // Matrix(5 x 3) * Vector(3) -> compatible
        assert!(!Shape::Vector(Dimension::Finite(5)).is_compatible_mul(&mat)); // Vector(5) * Matrix(5 x 3) -> incompatible (ignoring transposition for now)
    }
}
