use std::collections::HashMap;

use crate::pareto_dp::ParetoFrontSolution;

const DELTA: f64 = 1e-9;

#[derive(Debug, PartialEq)]
pub struct DataTable {
    data: Vec<Vec<Vec<f64>>>,

    n_stands: usize,         // number of stands in the data
    n_scenarios: Vec<usize>, // number of scenarios may be different for each stand
    n_variables: usize,      // number of variables for optimization
}

#[derive(Debug, PartialEq, Eq)]
pub enum DataTableError {
    Empty,
    InconsistentNumberOfVariables {
        expected: usize,
        got: usize,
        stand_index: usize,
        scenario_index: usize,
    },
}
impl std::fmt::Display for DataTableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Data table must not have empty entries."),
            Self::InconsistentNumberOfVariables {
                expected,
                got,
                stand_index,
                scenario_index,
            } => write!(
                f,
                "There are {expected} variables in the first entry, but got {got} at index [{stand_index}][{scenario_index}]. Need same number of variables in all data table entries!"
            ),
        }
    }
}

impl DataTable {
    pub fn new(data: &[Vec<Vec<f64>>]) -> Result<Self, DataTableError> {
        // check that the data is not empty
        if data.is_empty() {
            return Err(DataTableError::Empty);
        }
        let expected_n_variables = data
            .first()
            .and_then(|outer| outer.first())
            .map(Vec::len)
            .ok_or(DataTableError::Empty)?;

        let mut scenarios_cardinality: Vec<usize> = vec![];

        // Check non-empty values
        // and same number of variables in all entries
        for (stand_index, scenarios) in data.iter().enumerate() {
            scenarios_cardinality.push(scenarios.len());
            for (scenario_index, variables) in scenarios.iter().enumerate() {
                if variables.is_empty() {
                    return Err(DataTableError::Empty);
                } else if variables.len() != expected_n_variables {
                    return Err(DataTableError::InconsistentNumberOfVariables {
                        expected: expected_n_variables,
                        got: variables.len(),
                        stand_index,
                        scenario_index,
                    });
                }
            }
        }

        Ok(Self {
            data: data.to_owned(),
            n_stands: data.len(),
            n_scenarios: scenarios_cardinality,
            n_variables: expected_n_variables,
        })
    }
}

// Info about partial pareto fronts considered in the algorithm.
#[derive(Clone)]
struct PartialParetoPoint {
    // # v-dimensional vector in the objective space
    target_vector: Vec<f64>,
    // # Pointer to the parent vector (the i-1 step in the algorithm).
    // # Takes None value for the first stand, which has no parents.
    parent_point: Option<Box<Self>>,
    // # scenario choice for the ith step in the algorithm.
    // # The design space vector of the current point can be recovered
    // # by attaching this choice number to the parent point.
    current_scenario_choice: usize,
    // # step in the algorithm, a.k.a. i. Storing just in case.
    stand_index: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParetoFrontError {
    EmptyInput,
    NoParetoPointsFound,
}

impl std::fmt::Display for ParetoFrontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "Cannot build Pareto front: empty input data."),
            Self::NoParetoPointsFound => write!(f, "No Pareto points found in the data."),
        }
    }
}

fn get_first_stand_scenarios(
    data_table: &DataTable,
) -> Result<Vec<PartialParetoPoint>, ParetoFrontError> {
    // Initializes the partial Pareto  fronts with
    // the scenarios in the first stand
    let first_stand_vectors = data_table
        .data
        .first()
        .ok_or(ParetoFrontError::EmptyInput)?;

    let pareto_front: Vec<PartialParetoPoint> = first_stand_vectors
        .iter()
        .enumerate()
        .map(|(index, item)| PartialParetoPoint {
            target_vector: item.clone(),
            parent_point: None,
            current_scenario_choice: index,
            stand_index: 0,
        })
        .collect();
    Ok(pareto_front)
}
fn add_vectors(a: &[f64], b: &[f64]) -> Vec<f64> {
    assert_eq!(a.len(), b.len(), "Vectors must be the same length");
    a.iter().zip(b.iter()).map(|(x, y)| x + y).collect()
}

fn assign_bucket_to_point(point: &PartialParetoPoint, epsilon: f64) -> Vec<usize> {
    /*
    Assign a point to a bucket in the coarse-grained space.

    A bucket is the v-dimensional pixel where the Pareto point falls
    when the space is coarse-grained using logarithmic binning.

    Args:
        point: The Pareto point to assign to a bucket
        epsilon: The binning parameter controlling bucket size

    Returns:
        A tuple of integers representing the bucket coordinates
    */

    // ln_1p(x) computes ln(1+x)
    let base = epsilon.ln_1p();

    point
        .target_vector
        .iter()
        .map(
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::as_conversions
            )]
            |&coord| {
                let val = ((coord + DELTA).ln() / base).floor();
                assert!(
                    val.is_finite() && val >= 0.0,
                    "negative coordinate value: {val}"
                );
                val as usize
            },
        )
        .collect::<Vec<usize>>()
}

fn dominates(p: &PartialParetoPoint, q: &PartialParetoPoint) -> bool {
    p.target_vector
        .iter()
        .zip(q.target_vector.iter())
        .all(|(p_k, q_k)| p_k <= q_k)
        && p.target_vector
            .iter()
            .zip(q.target_vector.iter())
            .any(|(p_k, q_k)| p_k < q_k)
}

fn compress_into_buckets(points: Vec<PartialParetoPoint>, epsilon: f64) -> Vec<PartialParetoPoint> {
    let mut buckets: HashMap<Vec<usize>, PartialParetoPoint> = HashMap::new();
    for point in points {
        let bucket_key = assign_bucket_to_point(&point, epsilon);
        match buckets.get(&bucket_key) {
            Some(existing) if dominates(&point, existing) => {
                buckets.insert(bucket_key, point);
            }
            None => {
                buckets.insert(bucket_key, point);
            }
            _ => {} // bucket occupied by a dominant point, skip
        }
    }
    buckets.values().cloned().collect()
}

fn pareto_prune(points: Vec<PartialParetoPoint>) -> Vec<PartialParetoPoint> {
    let mut pareto = vec![];

    for point in points {
        // If any existing pareto front point dominates `point`, discard it entirely
        if pareto.iter().any(|q| dominates(q, &point)) {
            continue;
        }

        // Otherwise, remove all points that `point` dominates, then add it
        pareto.retain(|q| !dominates(&point, q));
        pareto.push(point);
    }
    pareto
}

fn pareto_epsilon_prune(points: Vec<PartialParetoPoint>, epsilon: f64) -> Vec<PartialParetoPoint> {
    pareto_prune(compress_into_buckets(points, epsilon))
}

#[allow(clippy::unnecessary_wraps)]
pub fn build_pareto_front(data_table: &DataTable) -> Result<ParetoFrontSolution, ParetoFrontError> {
    let mut pareto_front = get_first_stand_scenarios(data_table)?;

    // Remove stand 0 from loop: already considered in the initialization
    #[allow(clippy::indexing_slicing)]
    for stand_ix in 1..data_table.n_stands {
        let mut pareto_front_new: Vec<PartialParetoPoint> = vec![];
        for pareto_point in &pareto_front {
            for (scenario_ix, scenario_data) in data_table.data[stand_ix].iter().enumerate() {
                let new_target_vector = add_vectors(&pareto_point.target_vector, scenario_data);

                pareto_front_new.push(PartialParetoPoint {
                    target_vector: new_target_vector,
                    parent_point: Some(Box::new((*pareto_point).clone())),
                    current_scenario_choice: scenario_ix,
                    stand_index: stand_ix,
                });
            }
        }
        pareto_front = pareto_epsilon_prune(pareto_front_new, 1e-7);
    }

    // TODO: continue here!
    Ok(ParetoFrontSolution {
        design_vectors: vec![vec![10, 2]],
        target_vectors: vec![vec![0.5]],
    })
}

#[cfg(test)]
mod tests {
    // imports names from outer scope.
    use super::*;

    #[test]
    fn test_empty_data_error() {
        let empty_data = DataTable::new(&[vec![vec![]]]);
        assert_eq!(empty_data, Err(DataTableError::Empty));
    }

    #[test]
    fn test_some_empty_value_error() {
        let data_without_one_value = &[vec![vec![-1.4, 2.3], vec![]]];
        assert_eq!(
            DataTable::new(data_without_one_value),
            Err(DataTableError::Empty)
        );
    }

    #[test]
    fn test_inconsistent_number_of_variables_error() {
        let inconsistent_vars = &[vec![vec![2.0, 1.0, 4.3], vec![2.3, 3.4, 4.5, 1.2]]];
        assert_eq!(
            DataTable::new(inconsistent_vars),
            Err(DataTableError::InconsistentNumberOfVariables {
                expected: 3,
                got: 4,
                stand_index: 0,
                scenario_index: 1
            })
        );
    }

    #[test]
    fn test_correct_data_creation() {
        let data = vec![vec![vec![2.0, 1.0, 4.3], vec![2.3, 3.4, 4.5]]];
        assert_eq!(
            DataTable::new(&data),
            Ok(DataTable {
                data,
                n_stands: 1,
                n_scenarios: vec![2],
                n_variables: 3
            })
        );
    }
}
