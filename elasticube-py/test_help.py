#!/usr/bin/env python3
"""Check method signatures."""

from elasticube import ElastiCubeBuilder

builder = ElastiCubeBuilder("test")

print("=== load_parquet (working method) ===")
help(builder.load_parquet)

print("\n=== load_from_pandas (working method) ===")
help(builder.load_from_pandas)

print("\n=== load_from_polars (broken method) ===")
help(builder.load_from_polars)

print("\n=== load_from_arrow (probably broken) ===")
help(builder.load_from_arrow)
