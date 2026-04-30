use std::collections::HashMap;
use std::rc::Rc;

use crate::pareto_dp::ParetoFrontSolution;

const DELTA: f64 = 1e-9;

type ScenarioVector = Box<[f64]>;
type StandData = Box<[ScenarioVector]>;

#[derive(Debug, PartialEq)]
pub struct DataTable {
    data: Box<[StandData]>, // Complete type: Box<[Box<[Box<[f64]>]>]>

    n_stands: usize,           // number of stands in the data
    n_scenarios: Box<[usize]>, // number of scenarios may be different for each stand
    n_variables: usize,        // number of variables for optimization
}

#[derive(Debug, PartialEq, Eq)]
pub enum DataTableError {
    Empty,
    OnlyOneStand,
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
            Self::OnlyOneStand => write!(f, "There's only one stand. Nothing to optimize here."),
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
    pub fn new(data: Vec<Vec<Vec<f64>>>) -> Result<Self, DataTableError> {
        // check that the data is not empty
        if data.is_empty() {
            return Err(DataTableError::Empty);
        }
        if data.len() < 2 {
            return Err(DataTableError::OnlyOneStand);
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
        // Freeze all three levels: no Vec remains after this point.
        // Boxes are immutable, like tuples vs lists in Python
        let data: Box<[StandData]> = data
            .into_iter()
            .map(|scenarios| {
                scenarios
                    .into_iter()
                    .map(std::vec::Vec::into_boxed_slice)
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let n_stands = data.len();

        Ok(Self {
            data,
            n_stands,
            n_scenarios: scenarios_cardinality.into(),
            n_variables: expected_n_variables,
        })
    }
}

struct PartialParetoPoint {
    // # v-dimensional vector in the objective space
    target_vector: Box<[f64]>,
    // # Pointer to the parent vector (the i-1 step in the algorithm).
    // # Takes None value for the first stand, which has no parents.
    parent_point: Option<Rc<Self>>,
    // # scenario choice for the ith step in the algorithm.
    // # The design space vector of the current point can be recovered
    // # by attaching this choice number to the parent point.
    current_scenario_choice: usize,
}

fn get_first_stand_scenarios(first_stand: &StandData) -> Vec<Rc<PartialParetoPoint>> {
    first_stand
        .iter()
        .enumerate()
        .map(|(index, item)| {
            Rc::new(PartialParetoPoint {
                target_vector: item.clone(),
                parent_point: None,
                current_scenario_choice: index,
            })
        })
        .collect()
}
fn add_vectors(a: &[f64], b: &[f64]) -> Vec<f64> {
    assert_eq!(a.len(), b.len(), "Vectors must be the same length");
    a.iter().zip(b.iter()).map(|(x, y)| x + y).collect()
}

fn assign_bucket_to_vector(vector: &[f64], epsilon: f64) -> Vec<usize> {
    /*
    Assign vector to a bucket in the coarse-grained space.

    A bucket is the v-dimensional pixel where the Pareto point falls
    when the space is coarse-grained using logarithmic binning.

    Args:
        vector: The objective vector of the Pareto point
        epsilon: The binning parameter controlling bucket size

    Returns:
        A tuple of integers representing the bucket coordinates
    */

    // ln_1p(x) computes ln(1+x)
    let base = epsilon.ln_1p();

    vector
        .iter()
        .map(
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::as_conversions
            )]
            |&coord| {
                let val = ((coord + DELTA).ln_1p() / base).floor();
                assert!(
                    val.is_finite() && val >= 0.0,
                    "negative coordinate value: {val}"
                );
                val as usize
            },
        )
        .collect::<Vec<usize>>()
}

fn dominates(target_vector_p: &[f64], target_vector_q: &[f64]) -> bool {
    target_vector_p
        .iter()
        .zip(target_vector_q.iter())
        .all(|(p_k, q_k)| p_k <= q_k)
        && target_vector_p
            .iter()
            .zip(target_vector_q.iter())
            .any(|(p_k, q_k)| p_k < q_k)
}

fn compress_into_buckets(points: Vec<Rc<PartialParetoPoint>>, epsilon: f64) -> Vec<Rc<PartialParetoPoint>> {
    let mut buckets: HashMap<Vec<usize>, Rc<PartialParetoPoint>> = HashMap::new();
    for point in points {
        let bucket_key = assign_bucket_to_vector(&point.target_vector, epsilon);
        match buckets.get(&bucket_key) {
            Some(existing) if dominates(&point.target_vector, &existing.target_vector) => {
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

fn pareto_prune(points: Vec<Rc<PartialParetoPoint>>) -> Vec<Rc<PartialParetoPoint>> {
    let mut pareto = vec![];

    for point in points {
        // If any existing pareto front point dominates `point`, discard it entirely
        if pareto
            .iter()
            .any(|q: &Rc<PartialParetoPoint>| dominates(&q.target_vector, &point.target_vector))
        {
            continue;
        }

        // Otherwise, remove all points that `point` dominates, then add it
        pareto.retain(|q: &Rc<PartialParetoPoint>| !dominates(&point.target_vector, &q.target_vector));
        pareto.push(point);
    }
    pareto
}

fn pareto_epsilon_prune(points: Vec<Rc<PartialParetoPoint>>, epsilon: f64) -> Vec<Rc<PartialParetoPoint>> {
    pareto_prune(compress_into_buckets(points, epsilon))
}

fn recover_scenario_choices(point: &Rc<PartialParetoPoint>) -> Vec<usize> {
    let mut choices = Vec::new();
    let mut current: &Rc<PartialParetoPoint> = point;

    loop {
        choices.push(current.current_scenario_choice);
        match &current.parent_point {
            Some(parent) => current = parent,
            None => break,
        }
    }

    choices.reverse();
    choices
}

/// Indexing is safe because `DataTable::new()` validates that all stands and
/// scenarios exist and have consistent dimensions, and `design_vector.len() ==
/// data_table.n_stands` is checked by the assert below.
#[allow(clippy::indexing_slicing)]
fn compute_objective(design_vector: &[usize], data_table: &DataTable) -> Vec<f64> {
    assert_eq!(design_vector.len(), data_table.n_stands);
    let mut objective: Vec<f64> = vec![0.0; data_table.n_variables];
    for (stand_ix, scenario_ix) in design_vector.iter().enumerate() {
        objective = add_vectors(&objective, &data_table.data[stand_ix][*scenario_ix]);
    }
    objective
}

fn reconstruct_solution_pareto_front(
    pareto_front: Vec<Rc<PartialParetoPoint>>,
    data_table: &DataTable,
) -> Vec<ParetoFrontSolution> {
    //- Recovers scenario choices from nested structure
    //- Shifts the target vectors back from positive space
    //
    let mut solution = vec![];
    for point in pareto_front {
        let design_vector = recover_scenario_choices(&point);
        let target_vector = compute_objective(&design_vector, data_table);
        solution.push(ParetoFrontSolution {
            design_vector,
            target_vector,
        });
    }
    solution
}

#[allow(clippy::indexing_slicing)]
pub fn build_pareto_front(data_table: &DataTable, epsilon: f64) -> Vec<ParetoFrontSolution> {
    let mut pareto_front = get_first_stand_scenarios(&data_table.data[0]);

    // Remove stand 0 from loop: already considered in the initialization
    #[allow(clippy::indexing_slicing)]
    for stand_ix in 1..data_table.n_stands {
        let mut pareto_front_new: Vec<Rc<PartialParetoPoint>> = vec![];
        for pareto_point in &pareto_front {
            for (scenario_ix, scenario_vector) in data_table.data[stand_ix].iter().enumerate() {
                let new_target_vector = add_vectors(&pareto_point.target_vector, scenario_vector);

                pareto_front_new.push(Rc::new(PartialParetoPoint {
                    target_vector: new_target_vector.into(),
                    parent_point: Some(Rc::clone(pareto_point)),
                    current_scenario_choice: scenario_ix,
                }));
            }
        }
        pareto_front = pareto_epsilon_prune(pareto_front_new, epsilon);
    }

    reconstruct_solution_pareto_front(pareto_front, data_table)
}

#[cfg(test)]
mod tests {
    // imports names from outer scope.
    use super::*;

    #[test]
    fn test_empty_data_error() {
        let empty_data = DataTable::new(vec![vec![vec![]], vec![vec![1.0]]]);
        assert_eq!(empty_data, Err(DataTableError::Empty));
    }

    #[test]
    fn test_some_empty_value_error() {
        let data_without_one_value = vec![vec![vec![-1.4, 2.3], vec![]], vec![vec![1.0, 2.0]]];
        assert_eq!(
            DataTable::new(data_without_one_value),
            Err(DataTableError::Empty)
        );
    }

    #[test]
    fn test_inconsistent_number_of_variables_error() {
        let inconsistent_vars = vec![
            vec![vec![2.0, 1.0, 4.3], vec![2.3, 3.4, 4.5, 1.2]],
            vec![vec![1.0, 2.0, 3.0]],
        ];
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
}
