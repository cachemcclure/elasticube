"""Type stubs for elasticube"""

from typing import List, Optional, Dict, Tuple, Any
import pyarrow as pa
import pandas as pd

class ElastiCubeBuilder:
    """Builder for creating ElastiCube instances."""

    def __init__(self, name: str) -> None:
        """Create a new cube builder with a name."""
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

    def add_hierarchy(self, name: str, levels: List[str]) -> None:
        """
        Add a hierarchy to the cube.

        Args:
            name: Name of the hierarchy
            levels: List of dimension names forming the hierarchy (coarse to fine)
        """
        ...

    def add_calculated_measure(
        self, name: str, expression: str, data_type: str, agg_func: str
    ) -> None:
        """
        Add a calculated measure derived from an expression.

        Args:
            name: Name for the calculated measure
            expression: SQL expression (e.g., "revenue - cost")
            data_type: Expected result data type
            agg_func: Aggregation function
        """
        ...

    def add_virtual_dimension(
        self, name: str, expression: str, data_type: str
    ) -> None:
        """
        Add a virtual dimension computed from an expression.

        Args:
            name: Name for the virtual dimension
            expression: SQL expression (e.g., "EXTRACT(YEAR FROM sale_date)")
            data_type: Expected result data type
        """
        ...

    def with_description(self, description: str) -> None:
        """
        Set the cube description.

        Args:
            description: Human-readable description of the cube
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

    def batch_count(self) -> int:
        """Get the number of batches in the cube."""
        ...

    def dimensions(self) -> List[Dict[str, Any]]:
        """
        Get all dimensions.

        Returns:
            List of dimension dictionaries with keys: name, data_type, cardinality
        """
        ...

    def measures(self) -> List[Dict[str, Any]]:
        """
        Get all measures.

        Returns:
            List of measure dictionaries with keys: name, data_type, agg_func
        """
        ...

    def hierarchies(self) -> List[Dict[str, Any]]:
        """
        Get all hierarchies.

        Returns:
            List of hierarchy dictionaries with keys: name, levels
        """
        ...

    def get_dimension(self, name: str) -> Optional[Dict[str, Any]]:
        """
        Get a specific dimension by name.

        Args:
            name: Name of the dimension

        Returns:
            Dictionary with dimension metadata or None if not found
        """
        ...

    def get_measure(self, name: str) -> Optional[Dict[str, Any]]:
        """
        Get a specific measure by name.

        Args:
            name: Name of the measure

        Returns:
            Dictionary with measure metadata or None if not found
        """
        ...

    def get_hierarchy(self, name: str) -> Optional[Dict[str, Any]]:
        """
        Get a specific hierarchy by name.

        Args:
            name: Name of the hierarchy

        Returns:
            Dictionary with hierarchy metadata or None if not found
        """
        ...

    def description(self) -> Optional[str]:
        """
        Get the cube description.

        Returns:
            Description string or None if not set
        """
        ...

    def statistics(self) -> Dict[str, Any]:
        """
        Get cube statistics.

        Returns:
            Dictionary with statistics including row_count, partition_count,
            memory_bytes, memory_mb, and column_stats
        """
        ...

    def append_rows(self, data: pa.Table) -> int:
        """
        Append rows from a PyArrow Table.

        Args:
            data: PyArrow Table or RecordBatch

        Returns:
            Number of rows added
        """
        ...

    def append_batches(self, batches: List[pa.Table]) -> int:
        """
        Append multiple batches.

        Args:
            batches: List of PyArrow Tables/RecordBatches

        Returns:
            Total number of rows added
        """
        ...

    def delete_rows(self, filter_expr: str) -> int:
        """
        Delete rows matching a filter expression.

        Args:
            filter_expr: SQL WHERE clause

        Returns:
            Number of rows deleted
        """
        ...

    def update_rows(self, filter_expr: str, replacement_data: pa.Table) -> Tuple[int, int]:
        """
        Update rows matching a filter with replacement data.

        Args:
            filter_expr: SQL WHERE clause
            replacement_data: PyArrow Table with updated rows

        Returns:
            Tuple of (rows_deleted, rows_added)
        """
        ...

    def consolidate_batches(self) -> int:
        """
        Consolidate all batches into a single batch.

        Returns:
            Number of batches before consolidation
        """
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

    def offset(self, count: int) -> None:
        """
        Skip a number of rows (offset).

        Args:
            count: Number of rows to skip
        """
        ...

    def slice(self, dimension: str, value: str) -> None:
        """
        OLAP Operation: Slice - filter on a single dimension.

        Args:
            dimension: Dimension name to filter on
            value: Value to filter for
        """
        ...

    def dice(self, filters: List[Tuple[str, str]]) -> None:
        """
        OLAP Operation: Dice - filter on multiple dimensions.

        Args:
            filters: List of (dimension, value) tuples to filter on
        """
        ...

    def drill_down(self, parent_level: str, child_levels: List[str]) -> None:
        """
        OLAP Operation: Drill-down - navigate down a hierarchy.

        Args:
            parent_level: Parent level name (for reference)
            child_levels: List of child level names to drill down to
        """
        ...

    def roll_up(self, dimensions_to_remove: List[str]) -> None:
        """
        OLAP Operation: Roll-up - aggregate across dimensions.

        Args:
            dimensions_to_remove: List of dimension names to remove from grouping
        """
        ...

    def execute(self) -> pa.Table:
        """
        Execute the query and return results as PyArrow Table.

        Returns:
            PyArrow Table containing query results
        """
        ...

    def to_pandas(self) -> pd.DataFrame:
        """
        Execute the query and return results as Pandas DataFrame.

        Returns:
            Pandas DataFrame containing query results
        """
        ...

    def to_polars(self) -> Any:
        """
        Execute the query and return results as Polars DataFrame.

        Returns:
            Polars DataFrame containing query results
        """
        ...

__version__: str
__all__: List[str]
