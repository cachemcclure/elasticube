"""
ElastiCube - High-performance OLAP Cube Library

A Python library for building and querying OLAP cubes with a Rust backend,
leveraging Apache Arrow and DataFusion for maximum performance.

Example:
    >>> from elasticube import ElastiCubeBuilder
    >>>
    >>> builder = ElastiCubeBuilder("my_cube")
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

# Add visualization support
from .viz import add_viz_methods, CubeVisualizer
from .display import add_jupyter_repr, add_querybuilder_repr, enable_jupyter_integration
from .serialization import add_serialization_methods, CubeSerializer

# Enhance classes with Jupyter display and visualization support
add_jupyter_repr(ElastiCube)
add_querybuilder_repr(QueryBuilder)
add_viz_methods(QueryBuilder)
add_serialization_methods(ElastiCube)

# Enable Jupyter integration if available
_jupyter_enabled = enable_jupyter_integration()

__version__ = "1.0.2"
__all__ = [
    "ElastiCubeBuilder",
    "ElastiCube",
    "QueryBuilder",
    "CubeVisualizer",
    "CubeSerializer",
]
