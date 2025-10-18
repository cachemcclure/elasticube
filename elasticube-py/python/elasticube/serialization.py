"""
Serialization helpers for ElastiCube.

Provides methods to save and load cubes to/from disk using Parquet format,
which is more efficient than pickle for columnar data.
"""

import json
import os
from pathlib import Path
from typing import Optional, Dict, Any


class CubeSerializer:
    """Helper class for serializing and deserializing ElastiCube objects.

    Note: Direct pickle support is not available for ElastiCube due to the Rust
    backend. Instead, we provide efficient serialization via Parquet files.

    Example:
        >>> # Save a cube
        >>> CubeSerializer.save(cube, "my_cube.cube")
        >>>
        >>> # Load a cube
        >>> cube = CubeSerializer.load("my_cube.cube")
    """

    @staticmethod
    def save(cube, path: str) -> None:
        """Save an ElastiCube to disk.

        The cube is saved as a directory containing:
        - metadata.json: Cube metadata (name, schema, etc.)
        - data.parquet: The cube's data in Parquet format

        Args:
            cube: ElastiCube instance to save
            path: Path where to save the cube (will create a directory)

        Raises:
            ImportError: If pyarrow is not installed
        """
        try:
            import pyarrow as pa
            import pyarrow.parquet as pq
        except ImportError:
            raise ImportError(
                "pyarrow is required for serialization. "
                "Install with: pip install pyarrow"
            )

        # Create directory
        cube_dir = Path(path)
        cube_dir.mkdir(parents=True, exist_ok=True)

        # Get metadata
        metadata = {
            "name": cube.name(),
            "row_count": cube.row_count(),
            "version": "0.1.0",
        }

        # Save metadata
        metadata_path = cube_dir / "metadata.json"
        with open(metadata_path, "w") as f:
            json.dump(metadata, f, indent=2)

        # Get data as PyArrow table
        query = cube.query()
        table = query.execute()

        # Save data as Parquet
        data_path = cube_dir / "data.parquet"
        pq.write_table(table, data_path, compression="snappy")

        print(f"✓ Cube saved to {path}/")
        print(f"  - Metadata: {metadata_path}")
        print(f"  - Data: {data_path}")

    @staticmethod
    def load(path: str, name: Optional[str] = None):
        """Load an ElastiCube from disk.

        Args:
            path: Path to the saved cube directory
            name: Optional new name for the cube (uses saved name if not provided)

        Returns:
            Loaded ElastiCube instance

        Raises:
            FileNotFoundError: If the cube directory or files don't exist
            ImportError: If required libraries are not installed
        """
        from elasticube import ElastiCubeBuilder

        try:
            import pyarrow.parquet as pq
        except ImportError:
            raise ImportError(
                "pyarrow is required for deserialization. "
                "Install with: pip install pyarrow"
            )

        cube_dir = Path(path)

        # Check directory exists
        if not cube_dir.exists():
            raise FileNotFoundError(f"Cube directory not found: {path}")

        # Load metadata
        metadata_path = cube_dir / "metadata.json"
        if not metadata_path.exists():
            raise FileNotFoundError(f"Metadata file not found: {metadata_path}")

        with open(metadata_path, "r") as f:
            metadata = json.load(f)

        # Load data
        data_path = cube_dir / "data.parquet"
        if not data_path.exists():
            raise FileNotFoundError(f"Data file not found: {data_path}")

        # Use the provided name or fall back to saved name
        cube_name = name or metadata["name"]

        # Create a builder and load from Parquet
        builder = ElastiCubeBuilder(cube_name)
        builder.load_parquet(str(data_path))
        cube = builder.build()

        print(f"✓ Cube loaded from {path}/")
        print(f"  - Name: {cube.name()}")
        print(f"  - Rows: {cube.row_count():,}")

        return cube

    @staticmethod
    def export_parquet(cube, path: str, compression: str = "snappy") -> None:
        """Export cube data to a single Parquet file.

        Args:
            cube: ElastiCube instance to export
            path: Output path for the Parquet file
            compression: Compression algorithm ('snappy', 'gzip', 'brotli', 'zstd', 'none')

        Raises:
            ImportError: If pyarrow is not installed
        """
        try:
            import pyarrow.parquet as pq
        except ImportError:
            raise ImportError(
                "pyarrow is required for Parquet export. "
                "Install with: pip install pyarrow"
            )

        # Get all data
        query = cube.query()
        table = query.execute()

        # Write to Parquet
        pq.write_table(table, path, compression=compression)

        print(f"✓ Cube data exported to {path}")
        print(f"  - Compression: {compression}")
        print(f"  - Rows: {cube.row_count():,}")


def add_serialization_methods(elasticube_class):
    """Add serialization methods to ElastiCube class.

    This allows calling cube.save() and ElastiCube.load() directly.
    """

    def save(self, path: str) -> None:
        """Save this cube to disk.

        Args:
            path: Directory path where to save the cube

        Example:
            >>> cube.save("my_saved_cube")
        """
        CubeSerializer.save(self, path)

    def to_parquet(self, path: str, compression: str = "snappy") -> None:
        """Export cube data to a Parquet file.

        Args:
            path: Output path for the Parquet file
            compression: Compression algorithm (default: 'snappy')

        Example:
            >>> cube.to_parquet("output.parquet")
        """
        CubeSerializer.export_parquet(self, path, compression)

    # Add instance methods
    elasticube_class.save = save
    elasticube_class.to_parquet = to_parquet

    # Note: We can't easily add a class method in this pattern,
    # but users can use CubeSerializer.load() directly

    return elasticube_class


# Informative error message for pickle attempts
def _pickle_not_supported(*args, **kwargs):
    """Raise an informative error when pickle is attempted."""
    raise NotImplementedError(
        "ElastiCube does not support pickle due to its Rust backend.\n"
        "Use cube.save(path) and CubeSerializer.load(path) instead:\n\n"
        "  # Save a cube\n"
        "  cube.save('my_cube.cube')\n\n"
        "  # Load a cube\n"
        "  from elasticube.serialization import CubeSerializer\n"
        "  cube = CubeSerializer.load('my_cube.cube')\n\n"
        "For simple data export, use:\n"
        "  cube.to_parquet('output.parquet')\n"
    )
