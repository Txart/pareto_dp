use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
mod pareto_dp {
    use pyo3::prelude::*;

    // The way to get result.design_vectors and result.target_vectors in Python
    #[pyclass]
    pub struct ParetoFrontSolution {
        #[pyo3(get)]
        pub design_vectors: Vec<u32>,
        #[pyo3(get)]
        pub target_vectors: Vec<f64>,
    }

    /// Formats the sum of two numbers as string.
    #[pyfunction]
    fn find_pareto_front(data: Vec<f64>) -> PyResult<ParetoFrontSolution> {
        Ok(ParetoFrontSolution {
            design_vectors: vec![10, 2],
            target_vectors: vec![0.5],
        })
    }
}
