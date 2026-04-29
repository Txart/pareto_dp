import pareto_dp

result = pareto_dp.find_pareto_front(data=[[[3.0, 2.1], [2.0, 1.0]]])

print(result.design_vectors)
print(result.target_vectors)
