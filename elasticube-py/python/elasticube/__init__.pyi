"""Type stubs for elasticube"""

from typing import List, Optional
import pyarrow as pa
import pandas as pd

class ElastiCubeBuilder:
    """Builder for creating ElastiCube instances."""

    def __init__(self) -> None:
        """Create a new cube builder."""
        ...

    def add_dimension(self, name: str, data_type: str) -> None:
        """
        Add a dimension to the cube.

        Args:
            name: Name of the dimension
            data_type: Data type (e.g., 'string', 'int32', 'float64', 'date')
        """
        ...

    def add_measure(self, name: str, data_type: str, agg_func: str) -> None:
        """
        Add a measure to the cube.

        Args:
            name: Name of the measure
            data_type: Data type (e.g., 'int32', 'float64')
            agg_func: Aggregation function ('sum', 'avg', 'min', 'max', 'count')
        """
        ...

    def load_csv(self, path: str) -> None:
        """
        Load data from a CSV file.

        Args:
            path: Path to the CSV file
        """
        ...

    def load_parquet(self, path: str) -> None:
        """
        Load data from a Parquet file.

        Args:
            path: Path to the Parquet file
        """
        ...

    def load_json(self, path: str) -> None:
        """
        Load data from a JSON file.

        Args:
            path: Path to the JSON file
        """
        ...

    def build(self) -> ElastiCube:
        """
        Build the cube with loaded data.

        Returns:
            ElastiCube instance ready for querying
        """
        ...

class ElastiCube:
    """OLAP Cube for multidimensional analysis."""

    def query(self) -> QueryBuilder:
        """
        Create a new query builder.

        Returns:
            QueryBuilder instance for constructing queries
        """
        ...

    def name(self) -> str:
        """Get the cube name."""
        ...

    def row_count(self) -> int:
        """Get the number of rows in the cube."""
        ...

class QueryBuilder:
    """Builder for constructing cube queries."""

    def select(self, columns: List[str]) -> None:
        """
        Select columns to include in the result.

        Args:
            columns: List of column names or expressions (e.g., ['region', 'SUM(sales)'])
        """
        ...

    def filter(self, condition: str) -> None:
        """
        Add a filter condition.

        Args:
            condition: SQL WHERE clause condition (e.g., "sales > 1000")
        """
        ...

    def group_by(self, columns: List[str]) -> None:
        """
        Group results by columns.

        Args:
            columns: List of column names to group by
        """
        ...

    def order_by(self, columns: List[str]) -> None:
        """
        Order results by columns.

        Args:
            columns: List of column names to order by
        """
        ...

    def limit(self, n: int) -> None:
        """
        Limit the number of results.

        Args:
            n: Maximum number of rows to return
        """
        ...

    def execute(self) -> pa.RecordBatch:
        """
        Execute the query and return results as PyArrow RecordBatch.

        Returns:
            PyArrow RecordBatch containing query results
        """
        ...

    def to_pandas(self) -> pd.DataFrame:
        """
        Execute the query and return results as Pandas DataFrame.

        Returns:
            Pandas DataFrame containing query results
        """
        ...

__version__: str
__all__: List[str]
