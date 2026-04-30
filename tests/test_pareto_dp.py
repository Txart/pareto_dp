import unittest

import pareto_dp

EPSILON = 0.001


def dominates(a, b):
    """Check if target vector a dominates b (minimization)."""
    all_le = all(x <= y for x, y in zip(a, b))
    any_lt = any(x < y for x, y in zip(a, b))
    return all_le and any_lt


class TestFindParetoFrontErrors(unittest.TestCase):
    def test_empty_data(self):
        with self.assertRaises(ValueError):
            pareto_dp.find_pareto_front(data=[], epsilon=EPSILON)

    def test_single_stand(self):
        with self.assertRaises(ValueError):
            pareto_dp.find_pareto_front(data=[[[1.0, 2.0]]], epsilon=EPSILON)

    def test_inconsistent_variable_lengths(self):
        with self.assertRaises(ValueError):
            pareto_dp.find_pareto_front(
                data=[
                    [[1.0, 2.0, 3.0]],
                    [[4.0, 5.0]],
                ],
                epsilon=EPSILON,
            )

    def test_empty_inner_vector(self):
        with self.assertRaises(ValueError):
            pareto_dp.find_pareto_front(
                data=[
                    [[1.0, 2.0], []],
                    [[3.0, 4.0]],
                ],
                epsilon=EPSILON,
            )

    def test_empty_scenario_in_second_stand(self):
        with self.assertRaises(ValueError):
            pareto_dp.find_pareto_front(
                data=[
                    [[1.0, 2.0]],
                    [[3.0, 4.0], []],
                ],
                epsilon=EPSILON,
            )


class TestFindParetoFrontBasicFunctionality(unittest.TestCase):
    def test_returns_non_empty_results(self):
        data = [
            [[3.0, 2.1], [2.0, 1.0]],
            [[1.0, 1.0], [2.0, 0.5]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        self.assertGreater(len(results), 0)

    def test_design_vector_length_equals_stands(self):
        data = [
            [[3.0, 2.1], [2.0, 1.0]],
            [[1.0, 1.0], [2.0, 0.5]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        for result in results:
            self.assertEqual(len(result.design_vector), 2)

    def test_target_vector_length_equals_variables(self):
        data = [
            [[3.0, 2.1], [2.0, 1.0]],
            [[1.0, 1.0], [2.0, 0.5]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        for result in results:
            self.assertEqual(len(result.target_vector), 2)

    def test_target_vector_is_sum_of_selected_scenarios(self):
        data = [
            [[3.0, 2.1], [2.0, 1.0]],
            [[1.0, 1.0], [2.0, 0.5]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        for result in results:
            expected = [
                data[0][result.design_vector[0]][i]
                + data[1][result.design_vector[1]][i]
                for i in range(2)
            ]
            for i in range(2):
                self.assertAlmostEqual(result.target_vector[i], expected[i])

    def test_single_scenario_per_stand(self):
        data = [
            [[1.0, 2.0]],
            [[3.0, 4.0]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].design_vector, [0, 0])
        self.assertAlmostEqual(results[0].target_vector[0], 4.0)
        self.assertAlmostEqual(results[0].target_vector[1], 6.0)

    def test_three_stands(self):
        data = [
            [[1.0, 2.0], [3.0, 1.0]],
            [[0.5, 1.0], [2.0, 0.5]],
            [[1.0, 0.5], [0.5, 1.5]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        self.assertGreater(len(results), 0)
        for result in results:
            self.assertEqual(len(result.design_vector), 3)
            self.assertEqual(len(result.target_vector), 2)


class ParetoFrontTestMixin:
    def assert_pareto_front_is_valid(self, results):
        for i, r_i in enumerate(results):
            for j, r_j in enumerate(results):
                if i == j:
                    continue
                self.assertFalse(
                    dominates(r_j.target_vector, r_i.target_vector),
                    f"Result {j} dominates result {i}:\n"
                    f"  {r_j.target_vector} dominates {r_i.target_vector}",
                )


class TestFindParetoFrontParetoOptimality(unittest.TestCase, ParetoFrontTestMixin):
    def test_no_dominance_among_results_2d(self):
        data = [
            [[3.0, 2.1], [2.0, 1.0]],
            [[1.0, 1.0], [2.0, 0.5]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        self.assert_pareto_front_is_valid(results)

    def test_no_dominance_among_results_3d(self):
        data = [
            [[1.0, 2.0, 3.0], [3.0, 1.0, 2.0]],
            [[0.5, 1.0, 1.5], [2.0, 0.5, 1.0]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        self.assert_pareto_front_is_valid(results)

    def test_scratch_py_data(self):
        data = [
            [[3.0, 2.1], [2.0, 1.0]],
        ]
        with self.assertRaises(ValueError):
            pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)

    def test_all_scenarios_identical(self):
        data = [
            [[1.0, 1.0], [1.0, 1.0]],
            [[2.0, 2.0], [2.0, 2.0]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        self.assertGreater(len(results), 0)
        self.assert_pareto_front_is_valid(results)


class TestFindParetoFrontEdgeCases(unittest.TestCase, ParetoFrontTestMixin):
    def test_many_stands_varying_scenarios(self):
        data = [
            [[1.0, 2.0], [2.0, 1.0], [1.5, 1.5]],
            [[0.5, 0.5], [1.0, 0.3]],
            [[0.3, 1.0], [1.0, 0.3], [0.5, 0.5]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        self.assertGreater(len(results), 0)
        for result in results:
            self.assertEqual(len(result.design_vector), 3)
            self.assertEqual(len(result.target_vector), 2)

    def test_single_scenario_dominates(self):
        data = [
            [[5.0, 5.0], [1.0, 1.0]],
            [[1.0, 1.0], [10.0, 10.0]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        self.assert_pareto_front_is_valid(results)

    def test_design_vector_values_in_range(self):
        data = [
            [[1.0, 2.0], [2.0, 1.0], [3.0, 3.0]],
            [[0.5, 0.5], [1.0, 1.0]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        for result in results:
            self.assertIn(result.design_vector[0], [0, 1, 2])
            self.assertIn(result.design_vector[1], [0, 1])


class TestFindParetoFrontResultStructure(unittest.TestCase):
    def test_result_has_design_vector(self):
        data = [
            [[1.0, 2.0], [2.0, 1.0]],
            [[0.5, 0.5], [1.0, 1.0]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        for result in results:
            self.assertTrue(hasattr(result, "design_vector"))
            self.assertIsInstance(result.design_vector, list)

    def test_result_has_target_vector(self):
        data = [
            [[1.0, 2.0], [2.0, 1.0]],
            [[0.5, 0.5], [1.0, 1.0]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        for result in results:
            self.assertTrue(hasattr(result, "target_vector"))
            self.assertIsInstance(result.target_vector, list)

    def test_design_vector_contains_integers(self):
        data = [
            [[1.0, 2.0], [2.0, 1.0]],
            [[0.5, 0.5], [1.0, 1.0]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        for result in results:
            for val in result.design_vector:
                self.assertIsInstance(val, int)

    def test_target_vector_contains_floats(self):
        data = [
            [[1.0, 2.0], [2.0, 1.0]],
            [[0.5, 0.5], [1.0, 1.0]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        for result in results:
            for val in result.target_vector:
                self.assertIsInstance(val, float)

    def test_dominated_combinations_excluded_single_pareto_point(self):
        """One combination strictly dominates all others."""
        data = [
            [[5.0, 5.0], [1.0, 1.0]],
            [[1.0, 1.0], [10.0, 10.0]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].design_vector, [1, 0])
        self.assertAlmostEqual(results[0].target_vector[0], 2.0)
        self.assertAlmostEqual(results[0].target_vector[1], 2.0)

    def test_dominated_combinations_excluded_two_pareto_points(self):
        """Two Pareto-optimal points, two dominated combinations."""
        data = [
            [[1.0, 5.0], [5.0, 1.0]],
            [[1.0, 1.0], [10.0, 10.0]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        design_vectors = sorted(r.design_vector for r in results)
        self.assertEqual(len(results), 2)
        self.assertEqual(design_vectors, [[0, 0], [1, 0]])
        for result in results:
            self.assertNotEqual(result.design_vector, [0, 1])
            self.assertNotEqual(result.design_vector, [1, 1])

    def test_dominated_combinations_excluded_three_stands(self):
        """Three stands: one Pareto-optimal point, seven dominated."""
        data = [
            [[1.0, 2.0], [5.0, 5.0]],
            [[1.0, 1.0], [5.0, 5.0]],
            [[1.0, 1.0], [5.0, 5.0]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].design_vector, [0, 0, 0])
        self.assertAlmostEqual(results[0].target_vector[0], 3.0)
        self.assertAlmostEqual(results[0].target_vector[1], 4.0)

    def test_dominated_combinations_excluded_three_stands_multiple_pareto(self):
        """Three stands: four Pareto-optimal points, four dominated."""
        data = [
            [[1.0, 5.0], [5.0, 1.0]],
            [[2.0, 3.0], [3.0, 2.0]],
            [[1.0, 1.0], [10.0, 10.0]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        design_vectors = sorted(r.design_vector for r in results)
        self.assertEqual(len(results), 4)
        self.assertEqual(design_vectors, [[0, 0, 0], [0, 1, 0], [1, 0, 0], [1, 1, 0]])
        for result in results:
            self.assertNotEqual(result.design_vector, [0, 0, 1])
            self.assertNotEqual(result.design_vector, [0, 1, 1])
            self.assertNotEqual(result.design_vector, [1, 0, 1])
            self.assertNotEqual(result.design_vector, [1, 1, 1])

    def test_dominated_combinations_excluded_four_stands(self):
        """Four stands: dominated combos with large penalty in last stand."""
        data = [
            [[1.0, 4.0], [4.0, 1.0]],
            [[1.0, 4.0], [4.0, 1.0]],
            [[1.0, 4.0], [4.0, 1.0]],
            [[1.0, 1.0], [100.0, 100.0]],
        ]
        results = pareto_dp.find_pareto_front(data=data, epsilon=EPSILON)
        for result in results:
            self.assertEqual(result.design_vector[3], 0)
            self.assertNotEqual(result.design_vector[3], 1)


if __name__ == "__main__":
    unittest.main()
