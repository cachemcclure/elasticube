"""
ElastiCube - High-performance OLAP Cube Library

A Python library for building and querying OLAP cubes with a Rust backend,
leveraging Apache Arrow and DataFusion for maximum performance.

Example:
    >>> from elasticube import ElastiCubeBuilder
    >>>
    >>> builder = ElastiCubeBuilder()
    >>> builder.add_dimension("region", "string")
    >>> builder.add_measure("sales", "float64", "sum")
    >>> builder.load_csv("data.csv")
    >>> cube = builder.build()
    >>>
    >>> # Query the cube
    >>> query = cube.query()
    >>> query.select(["region", "SUM(sales)"])
    >>> query.group_by(["region"])
    >>> df = query.to_pandas()
    >>> print(df)
"""

from ._elasticube import (
    PyElastiCubeBuilder as ElastiCubeBuilder,
    PyElastiCube as ElastiCube,
    PyQueryBuilder as QueryBuilder,
)

__version__ = "0.1.0"
__all__ = ["ElastiCubeBuilder", "ElastiCube", "QueryBuilder"]
