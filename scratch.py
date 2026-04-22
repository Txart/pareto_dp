import pareto_dp

result = pareto_dp.find_pareto_front(data=[3])

print(result.design_vectors)
print(result.target_vectors)
