#!/usr/bin/env python3
"""
Serialization Demo for ElastiCube

Demonstrates how to save and load cubes to/from disk.
Since ElastiCube uses a Rust backend, it doesn't support pickle.
Instead, we use efficient Parquet-based serialization.

Requirements:
    pip install elasticube pyarrow
"""

import os
import tempfile
from elasticube import ElastiCubeBuilder
from elasticube.serialization import CubeSerializer


def main():
    print("=== ElastiCube Serialization Demo ===\n")

    # Get the path to the CSV file
    script_dir = os.path.dirname(os.path.abspath(__file__))
    csv_path = os.path.join(script_dir, "sales_data.csv")

    # Build a cube
    print("1. Building a cube from CSV data...")
    builder = ElastiCubeBuilder("sales_cube")
    builder.add_dimension("region", "utf8")
    builder.add_dimension("product", "utf8")
    builder.add_dimension("category", "utf8")
    builder.add_measure("sales", "float64", "sum")
    builder.add_measure("quantity", "int64", "sum")
    builder.load_csv(csv_path)
    cube = builder.build()
    print(f"✓ Created cube '{cube.name()}' with {cube.row_count()} rows\n")

    # Create a temporary directory for saved cubes
    with tempfile.TemporaryDirectory() as temp_dir:
        # Example 1: Save using the instance method
        print("2. Saving cube using cube.save() method...")
        save_path = os.path.join(temp_dir, "my_cube.cube")
        cube.save(save_path)
        print()

        # Example 2: Load using CubeSerializer
        print("3. Loading cube using CubeSerializer.load()...")
        loaded_cube = CubeSerializer.load(save_path)
        print()

        # Verify the loaded cube
        print("4. Verifying loaded cube...")
        print(f"   Original rows: {cube.row_count()}")
        print(f"   Loaded rows: {loaded_cube.row_count()}")
        print(f"   ✓ Match: {cube.row_count() == loaded_cube.row_count()}\n")

        # Query the loaded cube
        print("5. Querying the loaded cube...")
        query = loaded_cube.query()
        query.select([
            "region",
            "SUM(sales) as total_sales",
            "COUNT(*) as transactions"
        ])
        query.group_by(["region"])
        query.order_by(["total_sales DESC"])
        df = query.to_pandas()
        print(df)
        print()

        # Example 3: Export just the data to Parquet
        print("6. Exporting cube data to Parquet file...")
        parquet_path = os.path.join(temp_dir, "cube_export.parquet")
        cube.to_parquet(parquet_path, compression="snappy")
        print()

        # Example 4: Load cube from exported Parquet
        print("7. Loading cube from exported Parquet...")
        builder2 = ElastiCubeBuilder("imported_cube")
        builder2.load_parquet(parquet_path)
        imported_cube = builder2.build()
        print(f"✓ Imported cube '{imported_cube.name()}' with {imported_cube.row_count()} rows\n")

        # Example 5: Save with different compression
        print("8. Testing different compression formats...")
        for compression in ["snappy", "gzip", "zstd"]:
            export_path = os.path.join(temp_dir, f"cube_{compression}.parquet")
            cube.to_parquet(export_path, compression=compression)

            # Check file size
            file_size = os.path.getsize(export_path)
            print(f"   {compression:8s}: {file_size:,} bytes")
        print()

    # Summary
    print("\n=== Serialization Summary ===")
    print("✓ Cubes can be saved using: cube.save(path)")
    print("✓ Cubes can be loaded using: CubeSerializer.load(path)")
    print("✓ Data can be exported to Parquet: cube.to_parquet(path)")
    print("✓ Supports multiple compression formats: snappy, gzip, zstd, brotli")
    print("\n📁 Saved cubes contain:")
    print("   - metadata.json (cube name, schema, row count)")
    print("   - data.parquet (compressed columnar data)")
    print("\n💡 Tip: Parquet format is more efficient than pickle for OLAP data!")
    print("         It preserves schema, supports compression, and is language-agnostic.")

    # Demonstrate what happens if someone tries to pickle
    print("\n=== About Pickle Support ===")
    print("⚠️  ElastiCube does NOT support pickle due to its Rust backend.")
    print("    Use cube.save() and CubeSerializer.load() instead.")
    print("    This is actually better because:")
    print("    • Parquet is more efficient for columnar data")
    print("    • Cross-language compatibility (Rust, Python, R, etc.)")
    print("    • Built-in compression support")
    print("    • Schema preservation")


if __name__ == "__main__":
    main()
