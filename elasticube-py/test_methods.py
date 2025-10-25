#!/usr/bin/env python3
"""Quick diagnostic to check what methods are available."""

from elasticube import ElastiCubeBuilder

builder = ElastiCubeBuilder("test")

print("Available methods and attributes:")
for attr in dir(builder):
    if not attr.startswith('_'):
        val = getattr(builder, attr)
        print(f"  {attr}: {type(val).__name__}")

print("\nChecking specific methods:")
print(f"  load_from_polars: {hasattr(builder, 'load_from_polars')} - {type(getattr(builder, 'load_from_polars', None))}")
print(f"  load_from_pandas: {hasattr(builder, 'load_from_pandas')} - {type(getattr(builder, 'load_from_pandas', None))}")
print(f"  load_from_arrow: {hasattr(builder, 'load_from_arrow')} - {type(getattr(builder, 'load_from_arrow', None))}")
print(f"  load_parquet: {hasattr(builder, 'load_parquet')} - {type(getattr(builder, 'load_parquet', None))}")
