"""Tests for streaming DataFrame loading functionality."""

import pytest
import polars as pl
import pandas as pd
import pyarrow as pa

from elasticube import ElastiCubeBuilder, ElastiCube
from elasticube.streaming import (
    load_polars_chunked,
    load_pandas_chunked,
    estimate_chunk_size,
    stream_from_parquet,
)


class TestAppendMethods:
    """Test append_from_* methods on ElastiCube."""

    def test_append_from_polars_basic(self):
        """Test basic append_from_polars functionality."""
        # Create initial cube with 3 rows
        initial_df = pl.DataFrame({
            "region": ["North", "South", "East"],
            "sales": [100.0, 200.0, 150.0]
        })

        builder = ElastiCubeBuilder("sales")
        builder.add_dimension("region", "utf8")
        builder.add_measure("sales", "float64", "sum")
        builder.load_from_polars(initial_df)
        cube = builder.build()

        assert cube.row_count() == 3

        # Append 2 more rows
        new_df = pl.DataFrame({
            "region": ["West", "Central"],
            "sales": [175.0, 225.0]
        })

        rows_added = cube.append_from_polars(new_df)
        assert rows_added == 2
        assert cube.row_count() == 5

    def test_append_from_polars_type_normalization(self):
        """Test that append_from_polars handles type normalization."""
        # Initial data
        initial_df = pl.DataFrame({
            "name": ["Alice", "Bob"],
            "score": [95.5, 87.3]
        })

        builder = ElastiCubeBuilder("scores")
        builder.add_dimension("name", "utf8")
        builder.add_measure("score", "float64", "avg")
        builder.load_from_polars(initial_df)
        cube = builder.build()

        # Append with potential large_utf8 (depends on Polars version)
        new_df = pl.DataFrame({
            "name": ["Charlie"],
            "score": [92.1]
        })

        rows_added = cube.append_from_polars(new_df)
        assert rows_added == 1
        assert cube.row_count() == 3

    def test_append_from_pandas_basic(self):
        """Test basic append_from_pandas functionality."""
        # Create initial cube
        initial_df = pd.DataFrame({
            "category": ["A", "B", "C"],
            "amount": [10.0, 20.0, 15.0]
        })

        builder = ElastiCubeBuilder("amounts")
        builder.add_dimension("category", "utf8")
        builder.add_measure("amount", "float64", "sum")
        builder.load_from_pandas(initial_df)
        cube = builder.build()

        assert cube.row_count() == 3

        # Append more data
        new_df = pd.DataFrame({
            "category": ["D", "E"],
            "amount": [25.0, 30.0]
        })

        rows_added = cube.append_from_pandas(new_df)
        assert rows_added == 2
        assert cube.row_count() == 5

    def test_append_from_arrow_basic(self):
        """Test basic append_from_arrow functionality."""
        # Create initial cube
        initial_table = pa.table({
            "product": ["Widget", "Gadget"],
            "quantity": [100, 200]
        })

        builder = ElastiCubeBuilder("inventory")
        builder.add_dimension("product", "utf8")
        builder.add_measure("quantity", "int64", "sum")
        builder.load_from_arrow(initial_table)
        cube = builder.build()

        assert cube.row_count() == 2

        # Append more data
        new_table = pa.table({
            "product": ["Doohickey"],
            "quantity": [150]
        })

        rows_added = cube.append_from_arrow(new_table)
        assert rows_added == 1
        assert cube.row_count() == 3

    def test_append_methods_preserve_data(self):
        """Test that appending preserves all data correctly."""
        initial_df = pl.DataFrame({
            "id": [1, 2, 3],
            "value": [10.0, 20.0, 30.0]
        })

        builder = ElastiCubeBuilder("test")
        builder.add_dimension("id", "int64")
        builder.add_measure("value", "float64", "sum")
        builder.load_from_polars(initial_df)
        cube = builder.build()

        # Append more data
        new_df = pl.DataFrame({
            "id": [4, 5],
            "value": [40.0, 50.0]
        })
        cube.append_from_polars(new_df)

        # Query to verify all data is present
        query = cube.query()
        query.select(["SUM(value) as total"])
        result = query.to_polars()
        assert result["total"][0] == 150.0  # 10 + 20 + 30 + 40 + 50


class TestChunkedLoading:
    """Test chunked loading utilities."""

    def test_load_polars_chunked_with_builder(self):
        """Test loading a large Polars DataFrame in chunks."""
        # Create a DataFrame with 10,000 rows
        large_df = pl.DataFrame({
            "id": range(10_000),
            "value": [i * 1.5 for i in range(10_000)]
        })

        # Track progress
        progress_calls = []

        def track_progress(chunk, total, rows):
            progress_calls.append((chunk, total, rows))

        # Load in chunks of 2,500 rows
        builder = ElastiCubeBuilder("large")
        builder.add_dimension("id", "int64")
        builder.add_measure("value", "float64", "sum")

        cube = load_polars_chunked(
            builder,
            large_df,
            chunk_size=2_500,
            progress_callback=track_progress
        )

        # Verify cube was created correctly
        assert cube.row_count() == 10_000

        # Verify progress was tracked (4 chunks)
        assert len(progress_calls) == 4
        assert progress_calls[0] == (1, 4, 2_500)
        assert progress_calls[3] == (4, 4, 10_000)

        # Verify data integrity
        query = cube.query()
        query.select(["SUM(value) as total"])
        result = query.to_polars()
        expected_sum = sum(i * 1.5 for i in range(10_000))
        assert abs(result["total"][0] - expected_sum) < 0.01

    def test_load_polars_chunked_append_to_existing(self):
        """Test appending large DataFrame in chunks to existing cube."""
        # Create initial cube with 1,000 rows
        initial_df = pl.DataFrame({
            "id": range(1_000),
            "value": [float(i) for i in range(1_000)]
        })

        builder = ElastiCubeBuilder("test")
        builder.add_dimension("id", "int64")
        builder.add_measure("value", "float64", "sum")
        builder.load_from_polars(initial_df)
        cube = builder.build()

        assert cube.row_count() == 1_000

        # Create large DataFrame to append (5,000 rows)
        append_df = pl.DataFrame({
            "id": range(1_000, 6_000),
            "value": [float(i) for i in range(1_000, 6_000)]
        })

        # Append in chunks
        result_cube = load_polars_chunked(
            cube,
            append_df,
            chunk_size=1_000
        )

        # Verify total rows
        assert result_cube.row_count() == 6_000

    def test_load_pandas_chunked_basic(self):
        """Test loading a Pandas DataFrame in chunks."""
        # Create a DataFrame with 5,001 rows
        large_df = pd.DataFrame({
            "category": (["A", "B", "C"] * 1667)[:5001],  # Exactly 5,001 rows
            "amount": list(range(5_001))
        })

        builder = ElastiCubeBuilder("pandas_test")
        builder.add_dimension("category", "utf8")
        builder.add_measure("amount", "int64", "sum")

        cube = load_pandas_chunked(
            builder,
            large_df,
            chunk_size=1_000
        )

        assert cube.row_count() == 5_001

    def test_load_polars_chunked_small_chunks(self):
        """Test chunked loading with very small chunk size."""
        df = pl.DataFrame({
            "x": range(100),
            "y": range(100)
        })

        builder = ElastiCubeBuilder("small_chunks")
        builder.add_dimension("x", "int64")
        builder.add_measure("y", "int64", "sum")

        # Load with chunk size of 10
        cube = load_polars_chunked(builder, df, chunk_size=10)

        assert cube.row_count() == 100
        assert cube.batch_count() == 10  # Should have 10 batches

    def test_chunked_loading_error_cases(self):
        """Test error handling in chunked loading."""
        df = pl.DataFrame({"x": [1, 2, 3]})

        # Invalid chunk size
        with pytest.raises(ValueError, match="chunk_size must be >= 1"):
            load_polars_chunked(None, df, chunk_size=0)

        # Empty DataFrame
        empty_df = pl.DataFrame({"x": []})
        with pytest.raises(ValueError, match="Cannot load empty DataFrame"):
            load_polars_chunked(None, empty_df, chunk_size=100)

        # Invalid builder_or_cube type
        with pytest.raises(TypeError, match="must be ElastiCubeBuilder or ElastiCube"):
            load_polars_chunked("not a builder", df, chunk_size=100)


class TestEstimateChunkSize:
    """Test chunk size estimation utility."""

    def test_estimate_chunk_size_basic(self):
        """Test basic chunk size estimation."""
        # With 1GB memory and 100 byte rows
        chunk_size = estimate_chunk_size(
            total_rows=10_000_000,
            available_memory_mb=1024,
            row_size_bytes=100
        )

        # Should return a reasonable size between min and max
        assert 10_000 <= chunk_size <= 10_000_000

    def test_estimate_chunk_size_small_memory(self):
        """Test estimation with limited memory."""
        chunk_size = estimate_chunk_size(
            total_rows=10_000_000,
            available_memory_mb=128,  # Only 128MB
            row_size_bytes=200
        )

        # Should still be at least minimum
        assert chunk_size >= 10_000

    def test_estimate_chunk_size_large_memory(self):
        """Test estimation with plenty of memory."""
        chunk_size = estimate_chunk_size(
            total_rows=10_000_000,
            available_memory_mb=8192,  # 8GB
            row_size_bytes=50
        )

        # Should be capped at maximum
        assert chunk_size <= 10_000_000


class TestStreamFromParquet:
    """Test streaming from Parquet files."""

    def test_stream_from_parquet_polars(self, tmp_path):
        """Test streaming Parquet file using Polars."""
        # Create a Parquet file with test data
        df = pl.DataFrame({
            "id": range(5_000),
            "value": [i * 2.0 for i in range(5_000)]
        })

        parquet_path = tmp_path / "test_data.parquet"
        df.write_parquet(parquet_path)

        # Stream it in chunks
        progress_calls = []

        def track_progress(rows, total):
            progress_calls.append((rows, total))

        builder = ElastiCubeBuilder("streamed")
        builder.add_dimension("id", "int64")
        builder.add_measure("value", "float64", "sum")

        cube = stream_from_parquet(
            builder,
            str(parquet_path),
            chunk_size=1_000,
            progress_callback=track_progress,
            use_polars=True
        )

        assert cube.row_count() == 5_000
        assert len(progress_calls) == 5  # 5 chunks

    def test_stream_from_parquet_pandas(self, tmp_path):
        """Test streaming Parquet file using Pandas/PyArrow."""
        # Create a Parquet file with test data
        df = pd.DataFrame({
            "category": ["A", "B", "C"] * 1000,
            "amount": range(3_000)
        })

        parquet_path = tmp_path / "test_pandas.parquet"
        df.to_parquet(parquet_path, engine="pyarrow", index=False)

        # Stream it in chunks using Pandas
        builder = ElastiCubeBuilder("pandas_streamed")
        builder.add_dimension("category", "utf8")
        builder.add_measure("amount", "int64", "sum")

        cube = stream_from_parquet(
            builder,
            str(parquet_path),
            chunk_size=500,
            use_polars=False  # Force Pandas
        )

        assert cube.row_count() == 3_000


class TestIntegration:
    """Integration tests for streaming functionality."""

    def test_full_workflow_polars(self):
        """Test complete workflow: build with chunks, append with chunks."""
        # Initial data: 3,000 rows
        initial_df = pl.DataFrame({
            "date": ["2024-01-01"] * 3_000,
            "region": ["North", "South", "East"] * 1_000,
            "sales": range(3_000)
        })

        # Build cube in chunks
        builder = ElastiCubeBuilder("workflow")
        builder.add_dimension("date", "utf8")
        builder.add_dimension("region", "utf8")
        builder.add_measure("sales", "int64", "sum")

        cube = load_polars_chunked(builder, initial_df, chunk_size=1_000)
        assert cube.row_count() == 3_000

        # Append more data in chunks: 2,000 rows
        append_df = pl.DataFrame({
            "date": ["2024-01-02"] * 2_000,
            "region": ["West", "Central"] * 1_000,
            "sales": range(3_000, 5_000)
        })

        load_polars_chunked(cube, append_df, chunk_size=500)
        assert cube.row_count() == 5_000

        # Query to verify data
        query = cube.query()
        query.select(["SUM(sales) as total"])
        result = query.to_polars()
        expected = sum(range(5_000))
        assert result["total"][0] == expected

    def test_mixed_append_methods(self):
        """Test mixing different append methods (using Polars only to avoid schema mismatches)."""
        # Build with Polars
        initial_polars = pl.DataFrame({
            "id": [1, 2, 3],
            "value": [10.0, 20.0, 30.0]
        })

        builder = ElastiCubeBuilder("mixed")
        builder.add_dimension("id", "int64")
        builder.add_measure("value", "float64", "sum")
        builder.load_from_polars(initial_polars)
        cube = builder.build()

        # Append more with Polars (consistent types)
        polars_data2 = pl.DataFrame({
            "id": [4, 5],
            "value": [40.0, 50.0]
        })
        cube.append_from_polars(polars_data2)

        # Append more with Arrow (consistent types)
        polars_data3 = pl.DataFrame({
            "id": [6, 7],
            "value": [60.0, 70.0]
        })
        cube.append_from_arrow(polars_data3.to_arrow())

        assert cube.row_count() == 7

        # Verify sum
        query = cube.query()
        query.select(["SUM(value) as total"])
        result = query.to_polars()
        assert result["total"][0] == 280.0  # 10+20+30+40+50+60+70
