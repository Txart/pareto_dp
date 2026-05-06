# pareto_dp

Simple multi-objective optimization using dynamic programming to approximate the Pareto frontier.

Written in Rust, published as a Python PyPI package with `PyO3` and `maturin`.

## Problem specification

This library solves a multi-objective multiple-choice combinatorial optimization problem with **additive separable objectives**, designed for scenarios with precomputed data for independent decision points.

### Core user-facing properties
- **Independent groups**: The problem consists of `n` independent groups (e.g., decision stages, components) with no interdependencies.
- **Additive objective aggregation**: The total objective vector for a configuration is the sum of per-group contributions. For a configuration `x` (one scenario chosen per group), the `v`-dimensional objective is:
  $$
  F(x) = \sum_{i=1}^n a_{i,x_i} \in \mathbb{R}^v
  $$
  where `a_{i,j}` is the precomputed objective vector for group `i`, scenario `j`.
- **Precomputed lookup table**: All per-group, per-scenario objective values are precomputed (table-lookup/black-box). The library takes this 3D lookup table as direct input.

### Formal structure
- `n` independent groups (typical `n ~ 300`)
- Each group `i` has `m_i ∈ [3,12]` scenarios (branching options)
- Multiple-choice constraint: A configuration selects exactly one scenario per group
- `v` objectives (typical `v ~ 6`), precomputed as `a_{i,j} ∈ ℝᵛ`
- Goal: Compute the Pareto-optimal frontier in the `v`-dimensional objective space

### Scale challenge
The configuration space size is `|X| = ∏_{i=1}^n m_i`, which grows exponentially with `n`. Exhaustive search is infeasible for large `n`, so the library uses dynamic programming with epsilon-dominance pruning to efficiently approximate the Pareto front.

## Installation

```bash
pip install pareto-dp
```

## Usage

```python
from pareto_dp import find_pareto_front

# Define your multi-objective optimization problem
# Each stand has multiple scenarios, each scenario has objective values
data = [
    [[3.0, 2.1], [2.0, 1.0]],  # Stand 0: 2 scenarios, 2 objectives
    [[1.0, 1.0], [2.0, 0.5]],  # Stand 1: 2 scenarios, 2 objectives
]

# Find Pareto-optimal solutions with epsilon precision
solutions = find_pareto_front(data, epsilon=0.01)

for sol in solutions:
    print(f"Design: {sol.design_vector}, Objectives: {sol.target_vector}")
```

## API

### `find_pareto_front(data, epsilon)`

Finds all Pareto-optimal solutions for a multi-objective optimization problem.

**Parameters:**
- `data` (List[List[List[float]]]): A 3D list where:
  - First dimension: decision points
  - Second dimension: options for each decision point
  - Third dimension: objective variable values (e.g., [cost, time, quality])
- `epsilon` (float): Precision parameter for epsilon-dominance pruning

(Note: the context in which this problem originated is forestry land-use, so decision points are called `stands` in the code, and options are called `scenarios`).

**Returns:**
- List of `ParetoFrontSolution` objects, each with:
  - `design_vector` (List[int]): Which scenario to pick for each stand
  - `target_vector` (List[float]): The summed objective values for that design

**Raises:**
- `ValueError`: If data is empty, has only one stand, or has inconsistent dimensions

## Algorithm

The algorithm uses dynamic programming to efficiently find the Pareto front:

1. **Data Validation**: Validates input structure and dimensions
2. **Shift to Positive Space**: Shifts all objective values to positive space for numerical stability
3. **DP Tree Construction**: Builds partial Pareto fronts iteratively for each stand
4. **Epsilon-Dominance Pruning**: Uses logarithmic binning to prune dominated solutions
5. **Solution Reconstruction**: Traces back through parent pointers to reconstruct full solutions

## Requirements

- Python >= 3.9
- Compatible with CPython and PyPy

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Repository

https://github.com/Txart/pareto_dp
