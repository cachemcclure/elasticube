#!/usr/bin/env python3
"""Simple test without pre-defining schema"""

import os
from elasticube import ElastiCubeBuilder

# Get the path to the CSV file
script_dir = os.path.dirname(os.path.abspath(__file__))
csv_path = os.path.join(script_dir, "sales_data.csv")

# Build cube WITHOUT defining dimensions/measures (let it infer)
print("Building cube with schema inference...")
builder = ElastiCubeBuilder("sales_cube")
builder.load_csv(csv_path)
cube = builder.build()

print(f"✓ Cube '{cube.name()}' created with {cube.row_count()} rows\n")

# Try a simple query
print("=== Simple Query Test ===")
query = cube.query()
query.select(["region", "sales"])
query.limit(5)
df = query.to_pandas()
print(df)
