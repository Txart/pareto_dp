import pareto_dp

results = pareto_dp.find_pareto_front(data=[[[3.0, 2.1], [2.0, 1.0]], [[1.0, 1.0], [2.0, 0.5]]], epsilon=0.001)

for result in results:
    print(
        f"design vector = {result.design_vector}\ntarget vector = {result.target_vector}\n"
    )
