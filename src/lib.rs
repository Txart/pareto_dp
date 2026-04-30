mod pareto;
use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
mod pareto_dp {
    use pyo3::exceptions::PyValueError;
    use pyo3::prelude::*;

    use crate::pareto;

    #[pyclass]
    pub struct ParetoFrontSolution {
        #[pyo3(get)]
        pub design_vector: Vec<usize>,
        #[pyo3(get)]
        pub target_vector: Vec<f64>,
    }

    impl From<pareto::DataTableError> for PyErr {
        fn from(err: pareto::DataTableError) -> Self {
            PyValueError::new_err(err.to_string())
        }
    }

    #[pyfunction]
    fn find_pareto_front(data: Vec<Vec<Vec<f64>>>, epsilon: f64) -> PyResult<Vec<ParetoFrontSolution>> {
        let data_table = pareto::DataTable::new(data)?;
        Ok(pareto::build_pareto_front(&data_table, epsilon))
    }
}
