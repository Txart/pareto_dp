#[derive(Debug, PartialEq)]
pub struct DataTable {
    data: Vec<Vec<Vec<f64>>>,

    n_stands: usize,         // number of stands in the data
    n_scenarios: Vec<usize>, // number of scenarios may be different for each stand
    n_variables: usize,      // number of variables for optimization
}

#[derive(Debug, PartialEq)]
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
    pub fn new(data: Vec<Vec<Vec<f64>>>) -> Result<Self, DataTableError> {
        // check that the data is not empty
        if data.is_empty() {
            return Err(DataTableError::Empty);
        }
        let expected_n_variables = data[0][0].len();

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
            data: data.clone(),
            n_stands: data.len(),
            n_scenarios: scenarios_cardinality,
            n_variables: expected_n_variables,
        })
    }
}

#[cfg(test)]
mod tests {
    // imports names from outer scope.
    use super::*;

    #[test]
    fn test_empty_data_error() {
        let empty_data = DataTable::new(vec![vec![vec![]]]);
        assert_eq!(empty_data, Err(DataTableError::Empty));
    }

    #[test]
    fn test_some_empty_value_error() {
        let data_without_one_value = vec![vec![vec![-1.4, 2.3], vec![]]];
        assert_eq!(
            DataTable::new(data_without_one_value),
            Err(DataTableError::Empty)
        )
    }

    #[test]
    fn test_inconsistent_number_of_variables_error() {
        let inconsistent_vars = vec![vec![vec![2.0, 1.0, 4.3], vec![2.3, 3.4, 4.5, 1.2]]];
        assert_eq!(
            DataTable::new(inconsistent_vars),
            Err(DataTableError::InconsistentNumberOfVariables {
                expected: 3,
                got: 4,
                stand_index: 0,
                scenario_index: 1
            })
        )
    }

    #[test]
    fn test_correct_data_creation() {
        let data = vec![vec![vec![2.0, 1.0, 4.3], vec![2.3, 3.4, 4.5]]];
        assert_eq!(
            DataTable::new(data.clone()),
            Ok(DataTable {
                data: data,
                n_stands: 1,
                n_scenarios: vec![2],
                n_variables: 3
            })
        )
    }
}
