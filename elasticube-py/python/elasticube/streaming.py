"""Streaming utilities for loading large DataFrames into ElastiCube.

This module provides helper functions for efficiently loading very large DataFrames
(>10M rows) by chunking them into smaller batches. This approach:
- Reduces peak memory usage
- Allows progress tracking
- Enables processing of datasets larger than available RAM

Recommended chunk size: 1,000,000 rows (balances memory vs overhead)
"""

from typing import Optional, Callable, Any
import warnings


def load_polars_chunked(
    builder_or_cube: Any,
    df: "polars.DataFrame",
    chunk_size: int = 1_000_000,
    progress_callback: Optional[Callable[[int, int, int], None]] = None,
) -> Any:
    """Load a large Polars DataFrame in chunks to manage memory usage.

    This function splits a large DataFrame into chunks and loads them incrementally,
    either building a new cube (if builder provided) or appending to an existing
    cube (if cube provided).

    Args:
        builder_or_cube: ElastiCubeBuilder (for initial load) or ElastiCube (for append)
        df: Large Polars DataFrame to load
        chunk_size: Number of rows per chunk (default: 1,000,000)
        progress_callback: Optional function(chunk_num, total_chunks, rows_loaded)
                          called after each chunk

    Returns:
        ElastiCube: The built or updated cube

    Raises:
        ValueError: If chunk_size < 1 or df is empty
        TypeError: If builder_or_cube is neither ElastiCubeBuilder nor ElastiCube

    Example:
        >>> import polars as pl
        >>> from elasticube import ElastiCubeBuilder
        >>> from elasticube.streaming import load_polars_chunked
        >>>
        >>> # Create a large DataFrame (10M rows)
        >>> large_df = pl.DataFrame({
        ...     "id": range(10_000_000),
        ...     "value": range(10_000_000)
        ... })
        >>>
        >>> # Load in chunks with progress tracking
        >>> def progress(chunk, total, rows):
        ...     print(f"Loaded chunk {chunk}/{total} ({rows:,} rows)")
        >>>
        >>> builder = ElastiCubeBuilder("large_cube")
        >>> cube = load_polars_chunked(
        ...     builder,
        ...     large_df,
        ...     chunk_size=1_000_000,
        ...     progress_callback=progress
        ... )
        Loaded chunk 1/10 (1,000,000 rows)
        Loaded chunk 2/10 (2,000,000 rows)
        ...
    """
    if chunk_size < 1:
        raise ValueError(f"chunk_size must be >= 1, got {chunk_size}")

    total_rows = len(df)
    if total_rows == 0:
        raise ValueError("Cannot load empty DataFrame")

    # Calculate number of chunks
    total_chunks = (total_rows + chunk_size - 1) // chunk_size

    # Check what type we're working with
    from elasticube import ElastiCubeBuilder

    is_builder = hasattr(builder_or_cube, "load_from_polars") and hasattr(
        builder_or_cube, "build"
    )
    is_cube = hasattr(builder_or_cube, "append_from_polars") and not hasattr(
        builder_or_cube, "build"
    )

    if not is_builder and not is_cube:
        raise TypeError(
            "builder_or_cube must be ElastiCubeBuilder or ElastiCube, "
            f"got {type(builder_or_cube).__name__}"
        )

    cube = None
    total_loaded = 0

    for chunk_idx in range(total_chunks):
        start_idx = chunk_idx * chunk_size
        end_idx = min(start_idx + chunk_size, total_rows)
        chunk_df = df[start_idx:end_idx]

        if chunk_idx == 0 and is_builder:
            # First chunk with builder: load and build
            builder_or_cube.load_from_polars(chunk_df)
            cube = builder_or_cube.build()
        elif is_cube:
            # Appending to existing cube
            builder_or_cube.append_from_polars(chunk_df)
            cube = builder_or_cube
        else:
            # Subsequent chunks: append
            cube.append_from_polars(chunk_df)

        total_loaded += len(chunk_df)

        # Call progress callback if provided
        if progress_callback:
            progress_callback(chunk_idx + 1, total_chunks, total_loaded)

    return cube if cube is not None else builder_or_cube


def load_pandas_chunked(
    builder_or_cube: Any,
    df: "pandas.DataFrame",
    chunk_size: int = 1_000_000,
    progress_callback: Optional[Callable[[int, int, int], None]] = None,
) -> Any:
    """Load a large Pandas DataFrame in chunks to manage memory usage.

    This function splits a large DataFrame into chunks and loads them incrementally,
    either building a new cube (if builder provided) or appending to an existing
    cube (if cube provided).

    Note: For very large datasets (>1GB), consider using Polars instead of Pandas
    for better memory efficiency and performance.

    Args:
        builder_or_cube: ElastiCubeBuilder (for initial load) or ElastiCube (for append)
        df: Large Pandas DataFrame to load
        chunk_size: Number of rows per chunk (default: 1,000,000)
        progress_callback: Optional function(chunk_num, total_chunks, rows_loaded)
                          called after each chunk

    Returns:
        ElastiCube: The built or updated cube

    Raises:
        ValueError: If chunk_size < 1 or df is empty
        TypeError: If builder_or_cube is neither ElastiCubeBuilder nor ElastiCube

    Example:
        >>> import pandas as pd
        >>> from elasticube import ElastiCubeBuilder
        >>> from elasticube.streaming import load_pandas_chunked
        >>>
        >>> # Create a large DataFrame
        >>> large_df = pd.DataFrame({
        ...     "id": range(5_000_000),
        ...     "value": range(5_000_000)
        ... })
        >>>
        >>> builder = ElastiCubeBuilder("large_cube")
        >>> cube = load_pandas_chunked(builder, large_df, chunk_size=500_000)
    """
    if chunk_size < 1:
        raise ValueError(f"chunk_size must be >= 1, got {chunk_size}")

    total_rows = len(df)
    if total_rows == 0:
        raise ValueError("Cannot load empty DataFrame")

    # Warn if using Pandas for very large datasets
    if total_rows > 5_000_000:
        warnings.warn(
            f"Loading {total_rows:,} rows from Pandas DataFrame. "
            "For better performance with large datasets, consider using Polars: "
            "df_polars = pl.from_pandas(df)",
            PerformanceWarning,
            stacklevel=2,
        )

    # Calculate number of chunks
    total_chunks = (total_rows + chunk_size - 1) // chunk_size

    # Check what type we're working with
    is_builder = hasattr(builder_or_cube, "load_from_pandas") and hasattr(
        builder_or_cube, "build"
    )
    is_cube = hasattr(builder_or_cube, "append_from_pandas") and not hasattr(
        builder_or_cube, "build"
    )

    if not is_builder and not is_cube:
        raise TypeError(
            "builder_or_cube must be ElastiCubeBuilder or ElastiCube, "
            f"got {type(builder_or_cube).__name__}"
        )

    cube = None
    total_loaded = 0

    for chunk_idx in range(total_chunks):
        start_idx = chunk_idx * chunk_size
        end_idx = min(start_idx + chunk_size, total_rows)
        chunk_df = df.iloc[start_idx:end_idx]

        if chunk_idx == 0 and is_builder:
            # First chunk with builder: load and build
            builder_or_cube.load_from_pandas(chunk_df)
            cube = builder_or_cube.build()
        elif is_cube:
            # Appending to existing cube
            builder_or_cube.append_from_pandas(chunk_df)
            cube = builder_or_cube
        else:
            # Subsequent chunks: append
            cube.append_from_pandas(chunk_df)

        total_loaded += len(chunk_df)

        # Call progress callback if provided
        if progress_callback:
            progress_callback(chunk_idx + 1, total_chunks, total_loaded)

    return cube if cube is not None else builder_or_cube


def estimate_chunk_size(
    total_rows: int, available_memory_mb: float = 1024, row_size_bytes: int = 100
) -> int:
    """Estimate optimal chunk size based on available memory.

    This helper function calculates a recommended chunk size based on your
    available memory and estimated row size.

    Args:
        total_rows: Total number of rows in the DataFrame
        available_memory_mb: Available memory for loading in MB (default: 1GB)
        row_size_bytes: Estimated average row size in bytes (default: 100)

    Returns:
        int: Recommended chunk size (minimum 10,000, maximum 10,000,000)

    Example:
        >>> # Estimate for 50M rows with 2GB available memory
        >>> chunk_size = estimate_chunk_size(50_000_000, available_memory_mb=2048)
        >>> print(f"Recommended chunk size: {chunk_size:,}")
        Recommended chunk size: 2,000,000
    """
    # Convert available memory to bytes
    available_bytes = available_memory_mb * 1_048_576

    # Calculate max rows that fit in available memory
    # Account for 2x overhead (DataFrame + Arrow conversion)
    max_rows = int(available_bytes / (row_size_bytes * 2))

    # Clamp to reasonable range
    chunk_size = max(10_000, min(max_rows, 10_000_000))

    return chunk_size


def stream_from_parquet(
    builder_or_cube: Any,
    parquet_path: str,
    chunk_size: int = 1_000_000,
    progress_callback: Optional[Callable[[int, int], None]] = None,
    use_polars: bool = True,
) -> Any:
    """Stream a large Parquet file in chunks without loading it entirely into memory.

    This function reads a Parquet file in batches, which is more memory-efficient
    than loading the entire file and then chunking it.

    Args:
        builder_or_cube: ElastiCubeBuilder or ElastiCube
        parquet_path: Path to the Parquet file
        chunk_size: Number of rows per batch (default: 1,000,000)
        progress_callback: Optional function(rows_loaded, estimated_total) called per batch
        use_polars: Use Polars for reading (default: True, faster than Pandas)

    Returns:
        ElastiCube: The built or updated cube

    Example:
        >>> from elasticube import ElastiCubeBuilder
        >>> from elasticube.streaming import stream_from_parquet
        >>>
        >>> builder = ElastiCubeBuilder("large_cube")
        >>> cube = stream_from_parquet(
        ...     builder,
        ...     "large_file.parquet",
        ...     chunk_size=1_000_000
        ... )
    """
    if use_polars:
        try:
            import polars as pl

            # Read in batches using Polars LazyFrame
            lazy_df = pl.scan_parquet(parquet_path)

            # Get total row count if possible (may require scanning)
            try:
                total_rows = lazy_df.select(pl.len()).collect().item()
            except Exception:
                total_rows = None

            # Process in chunks
            cube = None
            rows_loaded = 0
            chunk_idx = 0

            # Polars doesn't have native chunked reading for LazyFrame,
            # so we'll slice it
            if total_rows:
                total_chunks = (total_rows + chunk_size - 1) // chunk_size
            else:
                total_chunks = None

            offset = 0
            is_builder = hasattr(builder_or_cube, "build")

            while True:
                chunk_df = (
                    lazy_df.slice(offset, chunk_size).collect()
                )  # Collect only this chunk

                if len(chunk_df) == 0:
                    break

                if chunk_idx == 0 and is_builder:
                    builder_or_cube.load_from_polars(chunk_df)
                    cube = builder_or_cube.build()
                elif cube is None:
                    # Appending to passed cube
                    builder_or_cube.append_from_polars(chunk_df)
                    cube = builder_or_cube
                else:
                    cube.append_from_polars(chunk_df)

                rows_loaded += len(chunk_df)
                chunk_idx += 1

                if progress_callback:
                    progress_callback(rows_loaded, total_rows)

                if len(chunk_df) < chunk_size:
                    break  # Last chunk

                offset += chunk_size

            return cube if cube else builder_or_cube

        except ImportError:
            warnings.warn(
                "Polars not available, falling back to Pandas. "
                "Install polars for better performance: pip install polars",
                ImportWarning,
            )
            use_polars = False

    if not use_polars:
        import pandas as pd
        import pyarrow.parquet as pq

        # Use PyArrow for chunked reading
        parquet_file = pq.ParquetFile(parquet_path)
        total_rows = parquet_file.metadata.num_rows

        cube = None
        rows_loaded = 0
        chunk_idx = 0
        is_builder = hasattr(builder_or_cube, "build")

        for batch in parquet_file.iter_batches(batch_size=chunk_size):
            # Convert to Pandas DataFrame
            chunk_df = batch.to_pandas()

            if chunk_idx == 0 and is_builder:
                builder_or_cube.load_from_pandas(chunk_df)
                cube = builder_or_cube.build()
            elif cube is None:
                builder_or_cube.append_from_pandas(chunk_df)
                cube = builder_or_cube
            else:
                cube.append_from_pandas(chunk_df)

            rows_loaded += len(chunk_df)
            chunk_idx += 1

            if progress_callback:
                progress_callback(rows_loaded, total_rows)

        return cube if cube else builder_or_cube


__all__ = [
    "load_polars_chunked",
    "load_pandas_chunked",
    "estimate_chunk_size",
    "stream_from_parquet",
]
