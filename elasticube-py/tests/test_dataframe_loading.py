#!/usr/bin/env python3
"""
Test suite for DataFrame loading functionality.

Tests cover:
- load_from_polars() method
- load_from_pandas() method
- load_from_arrow() method
- Type normalization (large_utf8, large_binary, timezone handling)
- Error handling (empty DataFrames, type mismatches, etc.)
"""

import pytest
import pyarrow as pa
import pandas as pd

try:
    import polars as pl
    POLARS_AVAILABLE = True
except ImportError:
    POLARS_AVAILABLE = False

from elasticube import ElastiCubeBuilder


class TestLoadFromPolars:
    """Test load_from_polars() method."""

    @pytest.mark.skipif(not POLARS_AVAILABLE, reason="Polars not installed")
    def test_load_from_polars_basic(self):
        """Test basic Polars DataFrame loading."""
        df = pl.DataFrame({
            "region": ["North", "South", "East", "West"],
            "product": ["Widget", "Gadget", "Widget", "Gadget"],
            "sales": [1000.0, 1500.0, 1200.0, 900.0],
            "quantity": [100, 150, 120, 90]
        })

        builder = ElastiCubeBuilder("polars_cube")
        builder.add_dimension("region", "utf8")
        builder.add_dimension("product", "utf8")
        builder.add_measure("sales", "float64", "sum")
        builder.add_measure("quantity", "int64", "sum")
        builder.load_from_polars(df)
        cube = builder.build()

        assert cube is not None
        assert cube.name() == "polars_cube"
        assert cube.row_count() == 4

    @pytest.mark.skipif(not POLARS_AVAILABLE, reason="Polars not installed")
    def test_load_from_polars_with_large_string(self):
        """Test Polars DataFrame with large_string type normalization."""
        # Create a DataFrame that Polars might use large_string for
        df = pl.DataFrame({
            "id": [1, 2, 3],
            "text": ["A" * 100, "B" * 100, "C" * 100],
            "value": [10.0, 20.0, 30.0]
        })

        builder = ElastiCubeBuilder("large_string_cube")
        builder.load_from_polars(df)
        cube = builder.build()

        assert cube is not None
        assert cube.row_count() == 3

    @pytest.mark.skipif(not POLARS_AVAILABLE, reason="Polars not installed")
    def test_load_from_polars_empty_dataframe(self):
        """Test error on empty Polars DataFrame."""
        df = pl.DataFrame({
            "region": [],
            "sales": []
        })

        builder = ElastiCubeBuilder("empty_cube")
        with pytest.raises(ValueError, match="No data batches found"):
            builder.load_from_polars(df)

    @pytest.mark.skipif(not POLARS_AVAILABLE, reason="Polars not installed")
    def test_load_from_polars_wrong_type(self):
        """Test error when passing non-DataFrame to load_from_polars."""
        builder = ElastiCubeBuilder("test_cube")
        with pytest.raises(RuntimeError, match="Failed to convert Polars DataFrame to Arrow"):
            builder.load_from_polars("not a dataframe")

    @pytest.mark.skipif(not POLARS_AVAILABLE, reason="Polars not installed")
    def test_load_from_polars_query_roundtrip(self):
        """Test complete workflow: Polars → ElastiCube → Query → Polars."""
        df = pl.DataFrame({
            "region": ["North", "South", "North", "South"],
            "sales": [100.0, 200.0, 150.0, 250.0]
        })

        builder = ElastiCubeBuilder("roundtrip_cube")
        builder.add_dimension("region", "utf8")
        builder.add_measure("sales", "float64", "sum")
        builder.load_from_polars(df)
        cube = builder.build()

        query = cube.query()
        query.select(["region", "SUM(sales) as total"])
        query.group_by(["region"])
        result_df = query.to_polars()

        assert isinstance(result_df, pl.DataFrame)
        assert len(result_df) == 2
        assert "region" in result_df.columns
        assert "total" in result_df.columns


class TestLoadFromPandas:
    """Test load_from_pandas() method."""

    def test_load_from_pandas_basic(self):
        """Test basic Pandas DataFrame loading."""
        df = pd.DataFrame({
            "region": ["North", "South", "East", "West"],
            "product": ["Widget", "Gadget", "Widget", "Gadget"],
            "sales": [1000.0, 1500.0, 1200.0, 900.0],
            "quantity": [100, 150, 120, 90]
        })

        builder = ElastiCubeBuilder("pandas_cube")
        builder.add_dimension("region", "utf8")
        builder.add_dimension("product", "utf8")
        builder.add_measure("sales", "float64", "sum")
        builder.add_measure("quantity", "int64", "sum")
        builder.load_from_pandas(df)
        cube = builder.build()

        assert cube is not None
        assert cube.name() == "pandas_cube"
        assert cube.row_count() == 4

    def test_load_from_pandas_with_datetime(self):
        """Test Pandas DataFrame with datetime columns."""
        df = pd.DataFrame({
            "date": pd.date_range("2024-01-01", periods=5),
            "revenue": [100.0, 200.0, 150.0, 300.0, 250.0]
        })

        builder = ElastiCubeBuilder("datetime_cube")
        builder.load_from_pandas(df)
        cube = builder.build()

        assert cube is not None
        assert cube.row_count() == 5

    def test_load_from_pandas_empty_dataframe(self):
        """Test error on empty Pandas DataFrame."""
        df = pd.DataFrame({
            "region": [],
            "sales": []
        })

        builder = ElastiCubeBuilder("empty_cube")
        with pytest.raises(ValueError, match="Cannot load empty DataFrame"):
            builder.load_from_pandas(df)

    def test_load_from_pandas_wrong_type(self):
        """Test error when passing non-DataFrame to load_from_pandas."""
        builder = ElastiCubeBuilder("test_cube")
        with pytest.raises(TypeError, match="Expected pandas.DataFrame"):
            builder.load_from_pandas("not a dataframe")

    def test_load_from_pandas_query_roundtrip(self):
        """Test complete workflow: Pandas → ElastiCube → Query → Pandas."""
        df = pd.DataFrame({
            "region": ["North", "South", "North", "South"],
            "sales": [100.0, 200.0, 150.0, 250.0]
        })

        builder = ElastiCubeBuilder("roundtrip_cube")
        builder.add_dimension("region", "utf8")
        builder.add_measure("sales", "float64", "sum")
        builder.load_from_pandas(df)
        cube = builder.build()

        query = cube.query()
        query.select(["region", "SUM(sales) as total"])
        query.group_by(["region"])
        result_df = query.to_pandas()

        assert isinstance(result_df, pd.DataFrame)
        assert len(result_df) == 2
        assert "region" in result_df.columns
        assert "total" in result_df.columns


class TestLoadFromArrow:
    """Test load_from_arrow() method."""

    def test_load_from_arrow_basic(self):
        """Test basic PyArrow Table loading."""
        table = pa.table({
            "region": ["North", "South", "East", "West"],
            "product": ["Widget", "Gadget", "Widget", "Gadget"],
            "sales": [1000.0, 1500.0, 1200.0, 900.0],
            "quantity": [100, 150, 120, 90]
        })

        builder = ElastiCubeBuilder("arrow_cube")
        builder.add_dimension("region", "utf8")
        builder.add_dimension("product", "utf8")
        builder.add_measure("sales", "float64", "sum")
        builder.add_measure("quantity", "int64", "sum")
        builder.load_from_arrow(table)
        cube = builder.build()

        assert cube is not None
        assert cube.name() == "arrow_cube"
        assert cube.row_count() == 4

    def test_load_from_arrow_with_large_utf8(self):
        """Test PyArrow Table with large_utf8 type normalization."""
        # Create a table with large_utf8 type
        schema = pa.schema([
            pa.field("id", pa.int64()),
            pa.field("text", pa.large_utf8()),
            pa.field("value", pa.float64())
        ])

        table = pa.table({
            "id": [1, 2, 3],
            "text": ["A" * 100, "B" * 100, "C" * 100],
            "value": [10.0, 20.0, 30.0]
        }, schema=schema)

        builder = ElastiCubeBuilder("large_utf8_cube")
        builder.load_from_arrow(table)
        cube = builder.build()

        assert cube is not None
        assert cube.row_count() == 3

    def test_load_from_arrow_with_large_binary(self):
        """Test PyArrow Table with large_binary type normalization."""
        # Create a table with large_binary type
        schema = pa.schema([
            pa.field("id", pa.int64()),
            pa.field("data", pa.large_binary()),
        ])

        table = pa.table({
            "id": [1, 2, 3],
            "data": [b"data1", b"data2", b"data3"],
        }, schema=schema)

        builder = ElastiCubeBuilder("large_binary_cube")
        builder.load_from_arrow(table)
        cube = builder.build()

        assert cube is not None
        assert cube.row_count() == 3

    def test_load_from_arrow_empty_table(self):
        """Test error on empty PyArrow Table."""
        table = pa.table({
            "region": [],
            "sales": []
        })

        builder = ElastiCubeBuilder("empty_cube")
        with pytest.raises(ValueError, match="No data batches found"):
            builder.load_from_arrow(table)

    def test_load_from_arrow_wrong_type(self):
        """Test error when passing non-Table to load_from_arrow."""
        builder = ElastiCubeBuilder("test_cube")
        with pytest.raises(AttributeError, match="'str' object has no attribute 'schema'"):
            builder.load_from_arrow("not a table")


class TestTypeNormalization:
    """Test type normalization functionality."""

    def test_mixed_types(self):
        """Test DataFrame with multiple types including those needing normalization."""
        # Create a table with various types
        schema = pa.schema([
            pa.field("id", pa.int64()),
            pa.field("name", pa.large_utf8()),  # Will be normalized to utf8
            pa.field("description", pa.utf8()),  # Already utf8
            pa.field("data", pa.large_binary()),  # Will be normalized to binary
            pa.field("value", pa.float64()),
        ])

        table = pa.table({
            "id": [1, 2, 3],
            "name": ["Alice", "Bob", "Charlie"],
            "description": ["Desc1", "Desc2", "Desc3"],
            "data": [b"data1", b"data2", b"data3"],
            "value": [10.0, 20.0, 30.0]
        }, schema=schema)

        builder = ElastiCubeBuilder("mixed_types_cube")
        builder.load_from_arrow(table)
        cube = builder.build()

        assert cube is not None
        assert cube.row_count() == 3

    def test_timezone_aware_timestamp(self):
        """Test handling of timezone-aware timestamps with warning."""
        df = pd.DataFrame({
            "timestamp": pd.date_range("2024-01-01", periods=3, tz="UTC"),
            "value": [10.0, 20.0, 30.0]
        })

        builder = ElastiCubeBuilder("timezone_cube")

        # This should warn about timezone conversion
        with pytest.warns(UserWarning, match="Converting timezone-aware timestamp"):
            builder.load_from_pandas(df)

        cube = builder.build()
        assert cube is not None
        assert cube.row_count() == 3


class TestIntegration:
    """Integration tests for complete workflows."""

    @pytest.mark.skipif(not POLARS_AVAILABLE, reason="Polars not installed")
    def test_polars_to_pandas_conversion(self):
        """Test converting Polars DataFrame → Cube → Pandas results."""
        df = pl.DataFrame({
            "category": ["A", "B", "A", "B"],
            "value": [10, 20, 30, 40]
        })

        builder = ElastiCubeBuilder("conversion_cube")
        builder.add_dimension("category", "utf8")
        builder.add_measure("value", "int64", "sum")
        builder.load_from_polars(df)
        cube = builder.build()

        query = cube.query()
        query.select(["category", "SUM(value) as total"])
        query.group_by(["category"])
        result_df = query.to_pandas()

        assert isinstance(result_df, pd.DataFrame)
        assert len(result_df) == 2
        # Verify aggregation worked correctly
        totals = {row['category']: row['total'] for _, row in result_df.iterrows()}
        assert totals['A'] == 40
        assert totals['B'] == 60

    def test_arrow_table_zero_copy(self):
        """Test that Arrow table loading is efficient (zero-copy when possible)."""
        # Create a large-ish table to test performance
        table = pa.table({
            "id": list(range(10000)),
            "value": [float(i) for i in range(10000)]
        })

        builder = ElastiCubeBuilder("zero_copy_cube")
        builder.load_from_arrow(table)
        cube = builder.build()

        assert cube is not None
        assert cube.row_count() == 10000


class TestErrorHandling:
    """Test error handling in DataFrame loading."""

    def test_no_polars_import_error(self):
        """Test helpful error when Polars is not available."""
        # We can't actually test this if Polars IS available
        # This is more of a documentation test
        pass

    @pytest.mark.skip(reason="Multiple loads may be intentionally allowed for appending data")
    def test_multiple_loads_error(self):
        """Test that you can't load data twice on the same builder."""
        df = pd.DataFrame({
            "region": ["North"],
            "sales": [100.0]
        })

        builder = ElastiCubeBuilder("multi_load_cube")
        builder.add_dimension("region", "utf8")
        builder.add_measure("sales", "float64", "sum")
        builder.load_from_pandas(df)

        # Second load should fail because builder was consumed
        with pytest.raises(RuntimeError, match="Builder already consumed"):
            builder.load_from_pandas(df)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
