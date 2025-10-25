#!/usr/bin/env python3
"""Direct test of load_from_polars."""

import polars as pl
from elasticube import ElastiCubeBuilder

print("Creating DataFrame...")
df = pl.DataFrame({
    "region": ["North", "South", "East", "West"],
    "product": ["Widget", "Gadget", "Widget", "Gadget"],
    "sales": [1000.0, 1500.0, 1200.0, 900.0],
    "quantity": [100, 150, 120, 90]
})

print(f"DataFrame created: {type(df)}")
print(f"DataFrame:\n{df}\n")

print("Creating builder...")
builder = ElastiCubeBuilder("polars_cube")
print(f"Builder created: {type(builder)}")

print("Adding dimensions and measures...")
builder.add_dimension("region", "utf8")
builder.add_dimension("product", "utf8")
builder.add_measure("sales", "float64", "sum")
builder.add_measure("quantity", "int64", "sum")

print("About to call load_from_polars...")
print(f"Method type: {type(builder.load_from_polars)}")
print(f"Method: {builder.load_from_polars}")

try:
    result = builder.load_from_polars(df)
    print(f"Success! Result: {result}")
except Exception as e:
    print(f"Error: {type(e).__name__}: {e}")
    import traceback
    traceback.print_exc()
