#!/usr/bin/env python3
"""
Comprehensive test suite for ElastiCube Python bindings.

Tests cover:
- Cube building from various sources (CSV, Parquet, JSON)
- Dimension and measure definitions
- Query operations (select, filter, group_by, order_by, limit)
- Aggregation functions (SUM, AVG, COUNT, MIN, MAX)
- OLAP operations (slice, dice, drill-down, roll-up)
- PyArrow and Pandas integration
- Error handling
"""

import os
import tempfile
import pytest
import pandas as pd
import pyarrow as pa
from elasticube import ElastiCubeBuilder


# Fixtures for test data
@pytest.fixture
def sample_csv(tmp_path):
    """Create a sample CSV file for testing."""
    csv_path = tmp_path / "test_data.csv"
    csv_content = """region,product,category,sales,quantity,year,quarter
North,Widget,Electronics,1000.0,100,2024,1
South,Widget,Electronics,1500.0,150,2024,1
North,Gadget,Hardware,800.0,80,2024,1
East,Widget,Electronics,1200.0,120,2024,1
South,Gadget,Hardware,950.0,95,2024,2
West,Widget,Electronics,1100.0,110,2024,2
North,Gadget,Hardware,750.0,75,2024,2
East,Gadget,Hardware,900.0,90,2024,2"""
    csv_path.write_text(csv_content)
    return str(csv_path)


@pytest.fixture
def sample_parquet(tmp_path):
    """Create a sample Parquet file for testing."""
    parquet_path = tmp_path / "test_data.parquet"
    df = pd.DataFrame({
        'region': ['North', 'South', 'East', 'West'],
        'product': ['Widget', 'Gadget', 'Widget', 'Gadget'],
        'sales': [1000.0, 1500.0, 1200.0, 900.0],
        'quantity': [100, 150, 120, 90]
    })
    df.to_parquet(parquet_path, index=False)
    return str(parquet_path)


@pytest.fixture
def sample_json(tmp_path):
    """Create a sample JSON file for testing (newline-delimited JSON)."""
    json_path = tmp_path / "test_data.json"
    # Arrow expects newline-delimited JSON (NDJSON), not a JSON array
    json_content = """{"region": "North", "product": "Widget", "sales": 1000.0, "quantity": 100}
{"region": "South", "product": "Gadget", "sales": 1500.0, "quantity": 150}
{"region": "East", "product": "Widget", "sales": 1200.0, "quantity": 120}"""
    json_path.write_text(json_content)
    return str(json_path)


class TestCubeBuilder:
    """Test ElastiCubeBuilder functionality."""

    def test_builder_creation(self):
        """Test creating a new builder."""
        builder = ElastiCubeBuilder("test_cube")
        assert builder is not None

    def test_add_dimension(self):
        """Test adding dimensions to the builder."""
        builder = ElastiCubeBuilder("test_cube")
        builder.add_dimension("region", "utf8")
        builder.add_dimension("year", "int64")
        assert builder is not None

    def test_add_measure(self):
        """Test adding measures to the builder."""
        builder = ElastiCubeBuilder("test_cube")
        builder.add_measure("sales", "float64", "sum")
        builder.add_measure("quantity", "int64", "avg")
        assert builder is not None

    def test_load_csv(self, sample_csv):
        """Test loading data from CSV."""
        builder = ElastiCubeBuilder("csv_cube")
        builder.add_dimension("region", "utf8")
        builder.add_measure("sales", "float64", "sum")
        builder.load_csv(sample_csv)
        cube = builder.build()
        assert cube is not None
        assert cube.name() == "csv_cube"
        assert cube.row_count() > 0

    def test_load_parquet(self, sample_parquet):
        """Test loading data from Parquet."""
        builder = ElastiCubeBuilder("parquet_cube")
        builder.add_dimension("region", "utf8")
        builder.add_measure("sales", "float64", "sum")
        builder.load_parquet(sample_parquet)
        cube = builder.build()
        assert cube is not None
        assert cube.name() == "parquet_cube"
        assert cube.row_count() == 4

    def test_load_json(self, sample_json):
        """Test loading data from JSON."""
        builder = ElastiCubeBuilder("json_cube")
        builder.add_dimension("region", "utf8")
        builder.add_measure("sales", "float64", "sum")
        builder.load_json(sample_json)
        cube = builder.build()
        assert cube is not None
        assert cube.name() == "json_cube"
        assert cube.row_count() == 3

    def test_schema_inference(self, sample_csv):
        """Test building cube without explicit schema (schema inference)."""
        builder = ElastiCubeBuilder("inferred_cube")
        builder.load_csv(sample_csv)
        cube = builder.build()
        assert cube is not None
        assert cube.row_count() > 0


class TestQueryOperations:
    """Test query operations and methods."""

    @pytest.fixture
    def test_cube(self, sample_csv):
        """Create a test cube for query operations."""
        builder = ElastiCubeBuilder("query_test_cube")
        builder.add_dimension("region", "utf8")
        builder.add_dimension("product", "utf8")
        builder.add_dimension("category", "utf8")
        builder.add_dimension("year", "int64")
        builder.add_dimension("quarter", "int64")
        builder.add_measure("sales", "float64", "sum")
        builder.add_measure("quantity", "int64", "sum")
        builder.load_csv(sample_csv)
        return builder.build()

    def test_basic_select(self, test_cube):
        """Test basic SELECT query."""
        query = test_cube.query()
        query.select(["region", "sales"])
        query.limit(5)
        result = query.execute()
        assert result is not None
        assert isinstance(result, pa.Table)

    def test_select_with_aggregation(self, test_cube):
        """Test SELECT with aggregation functions."""
        query = test_cube.query()
        query.select(["region", "SUM(sales) as total_sales"])
        query.group_by(["region"])
        df = query.to_pandas()
        assert df is not None
        assert isinstance(df, pd.DataFrame)
        assert 'region' in df.columns
        assert 'total_sales' in df.columns

    def test_filter_operation(self, test_cube):
        """Test WHERE clause filtering."""
        query = test_cube.query()
        query.select(["region", "product", "sales"])
        query.filter("sales > 900")
        df = query.to_pandas()
        assert df is not None
        assert all(df['sales'] > 900)

    def test_group_by(self, test_cube):
        """Test GROUP BY aggregation."""
        query = test_cube.query()
        query.select(["region", "SUM(sales) as total", "COUNT(*) as count"])
        query.group_by(["region"])
        df = query.to_pandas()
        assert df is not None
        assert len(df) <= test_cube.row_count()

    def test_order_by(self, test_cube):
        """Test ORDER BY sorting."""
        query = test_cube.query()
        query.select(["region", "sales"])
        query.order_by(["sales DESC"])
        query.limit(3)
        df = query.to_pandas()
        assert df is not None
        # Check that sales are in descending order
        sales_values = df['sales'].tolist()
        assert sales_values == sorted(sales_values, reverse=True)

    def test_limit(self, test_cube):
        """Test LIMIT clause."""
        query = test_cube.query()
        query.select(["region", "sales"])
        query.limit(3)
        df = query.to_pandas()
        assert df is not None
        assert len(df) == 3

    def test_complex_query(self, test_cube):
        """Test complex query with multiple operations."""
        query = test_cube.query()
        query.select([
            "region",
            "category",
            "SUM(sales) as total_sales",
            "AVG(quantity) as avg_qty",
            "COUNT(*) as count"
        ])
        query.filter("sales > 700")
        query.group_by(["region", "category"])
        query.order_by(["total_sales DESC"])
        query.limit(5)
        df = query.to_pandas()
        assert df is not None
        assert len(df) <= 5
        assert 'total_sales' in df.columns


class TestAggregations:
    """Test aggregation functions."""

    @pytest.fixture
    def agg_cube(self, sample_csv):
        """Create a cube for aggregation testing."""
        builder = ElastiCubeBuilder("agg_cube")
        builder.add_measure("sales", "float64", "sum")
        builder.add_measure("quantity", "int64", "sum")
        builder.load_csv(sample_csv)
        return builder.build()

    def test_sum_aggregation(self, agg_cube):
        """Test SUM aggregation."""
        query = agg_cube.query()
        query.select(["SUM(sales) as total"])
        df = query.to_pandas()
        assert df is not None
        assert 'total' in df.columns
        assert df['total'].iloc[0] > 0

    def test_avg_aggregation(self, agg_cube):
        """Test AVG aggregation."""
        query = agg_cube.query()
        query.select(["AVG(sales) as average"])
        df = query.to_pandas()
        assert df is not None
        assert 'average' in df.columns

    def test_count_aggregation(self, agg_cube):
        """Test COUNT aggregation."""
        query = agg_cube.query()
        query.select(["COUNT(*) as count"])
        df = query.to_pandas()
        assert df is not None
        assert df['count'].iloc[0] == agg_cube.row_count()

    def test_min_max_aggregation(self, agg_cube):
        """Test MIN and MAX aggregations."""
        query = agg_cube.query()
        query.select([
            "MIN(sales) as min_sales",
            "MAX(sales) as max_sales"
        ])
        df = query.to_pandas()
        assert df is not None
        assert df['min_sales'].iloc[0] <= df['max_sales'].iloc[0]


class TestOLAPOperations:
    """Test OLAP-specific operations."""

    @pytest.fixture
    def olap_cube(self, sample_csv):
        """Create a cube for OLAP testing."""
        builder = ElastiCubeBuilder("olap_cube")
        builder.add_dimension("region", "utf8")
        builder.add_dimension("product", "utf8")
        builder.add_dimension("category", "utf8")
        builder.add_measure("sales", "float64", "sum")
        builder.load_csv(sample_csv)
        return builder.build()

    def test_slice_operation(self, olap_cube):
        """Test slice (filter on single dimension)."""
        query = olap_cube.query()
        query.filter("region = 'North'")
        query.select(["product", "SUM(sales) as total"])
        query.group_by(["product"])
        df = query.to_pandas()
        assert df is not None
        # All results should be from North region
        query_all = olap_cube.query()
        query_all.filter("region = 'North'")
        query_all.select(["region"])
        df_all = query_all.to_pandas()
        assert all(df_all['region'] == 'North')

    def test_dice_operation(self, olap_cube):
        """Test dice (filter on multiple dimensions)."""
        query = olap_cube.query()
        query.filter("region = 'North' AND product = 'Widget'")
        query.select(["SUM(sales) as total"])
        df = query.to_pandas()
        assert df is not None
        assert len(df) > 0

    def test_drill_down(self, olap_cube):
        """Test drill-down (from region to product)."""
        # First get regional totals
        query1 = olap_cube.query()
        query1.select(["region", "SUM(sales) as total"])
        query1.group_by(["region"])
        df1 = query1.to_pandas()

        # Then drill down to product level
        query2 = olap_cube.query()
        query2.select(["region", "product", "SUM(sales) as total"])
        query2.group_by(["region", "product"])
        df2 = query2.to_pandas()

        assert df1 is not None
        assert df2 is not None
        # Drill-down should have more rows (more granular)
        assert len(df2) >= len(df1)

    def test_roll_up(self, olap_cube):
        """Test roll-up (from product to region)."""
        # Start with detailed product-level data
        query1 = olap_cube.query()
        query1.select(["region", "product", "SUM(sales) as total"])
        query1.group_by(["region", "product"])
        df1 = query1.to_pandas()

        # Roll up to region level
        query2 = olap_cube.query()
        query2.select(["region", "SUM(sales) as total"])
        query2.group_by(["region"])
        df2 = query2.to_pandas()

        assert df1 is not None
        assert df2 is not None
        # Roll-up should have fewer rows (less granular)
        assert len(df2) <= len(df1)


class TestDataFrameIntegration:
    """Test integration with Pandas and PyArrow."""

    @pytest.fixture
    def df_cube(self, sample_csv):
        """Create a cube for DataFrame testing."""
        builder = ElastiCubeBuilder("df_cube")
        builder.load_csv(sample_csv)
        return builder.build()

    def test_to_pandas(self, df_cube):
        """Test conversion to Pandas DataFrame."""
        query = df_cube.query()
        query.select(["region", "sales"])
        query.limit(5)
        df = query.to_pandas()
        assert isinstance(df, pd.DataFrame)
        assert len(df) > 0
        assert 'region' in df.columns
        assert 'sales' in df.columns

    def test_execute_pyarrow(self, df_cube):
        """Test execution returning PyArrow Table."""
        query = df_cube.query()
        query.select(["region", "sales"])
        query.limit(5)
        table = query.execute()
        assert isinstance(table, pa.Table)
        assert table.num_rows > 0
        assert 'region' in table.column_names
        assert 'sales' in table.column_names

    def test_pandas_operations(self, df_cube):
        """Test Pandas operations on query results."""
        query = df_cube.query()
        query.select(["region", "sales", "quantity"])
        df = query.to_pandas()

        # Perform Pandas operations
        grouped = df.groupby('region')['sales'].sum()
        assert grouped is not None
        assert len(grouped) > 0

        # Statistical operations
        stats = df['sales'].describe()
        assert stats is not None
        assert 'mean' in stats.index


class TestErrorHandling:
    """Test error handling and edge cases."""

    def test_invalid_file_path(self):
        """Test loading from non-existent file."""
        builder = ElastiCubeBuilder("error_cube")
        builder.load_csv("/nonexistent/path/to/file.csv")
        # Error is raised when building, not when loading
        with pytest.raises(Exception):
            builder.build()

    def test_empty_select(self, sample_csv):
        """Test query with empty select clause."""
        builder = ElastiCubeBuilder("test_cube")
        builder.load_csv(sample_csv)
        cube = builder.build()

        query = cube.query()
        # Empty select should still work (selects all columns)
        df = query.to_pandas()
        assert df is not None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
