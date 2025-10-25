#!/usr/bin/env python3
"""Test what Polars to_arrow() returns."""

import polars as pl
import pyarrow as pa

print("Creating Polars DataFrame...")
df = pl.DataFrame({
    "region": ["North", "South"],
    "sales": [100.0, 200.0]
})

print(f"DataFrame:\n{df}\n")
print(f"DataFrame type: {type(df)}")
print(f"DataFrame has 'to_arrow' method: {hasattr(df, 'to_arrow')}")

print("\nCalling to_arrow()...")
arrow_table = df.to_arrow()

print(f"Result type: {type(arrow_table)}")
print(f"Result: {arrow_table}")
print(f"Schema: {arrow_table.schema}")

print(f"\nIs it a PyArrow Table? {isinstance(arrow_table, pa.Table)}")
print(f"Has 'schema' attribute? {hasattr(arrow_table, 'schema')}")

# Try calling methods on it
try:
    schema = arrow_table.schema
    print(f"Schema names: {schema.names}")
    print(f"Num rows: {arrow_table.num_rows}")
except Exception as e:
    print(f"Error accessing table: {e}")
