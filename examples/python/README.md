# ElastiCube Python Examples

This directory contains comprehensive examples demonstrating ElastiCube's capabilities.

## Setup

Install ElastiCube with optional dependencies:

```bash
# Basic installation
pip install elasticube

# With visualization support
pip install elasticube[viz]

# With Jupyter notebook support
pip install elasticube[jupyter]

# With Polars support (high performance)
pip install elasticube[polars]

# Install everything
pip install elasticube[all]
```

## Examples

### 1. Basic Usage

**simple_test.py** - Getting started with ElastiCube
- Building a cube from CSV data
- Basic queries
- Converting results to Pandas DataFrames

```bash
python simple_test.py
```

### 2. Query Operations

**query_demo.py** - Comprehensive query examples
- SELECT, WHERE, GROUP BY, ORDER BY
- Aggregation functions (SUM, AVG, COUNT, MIN, MAX)
- Complex multi-dimensional queries

```bash
python query_demo.py
```

### 3. Polars Integration

**polars_demo.py** - High-performance DataFrame operations
- Zero-copy conversion from Arrow to Polars
- Lazy evaluation with Polars
- Performance comparisons (Polars vs Pandas)
- Advanced aggregations and transformations

**Requirements**: `pip install polars`

```bash
python polars_demo.py
```

### 4. Visualization

**visualization_demo.py** - Create charts from cube data
- Bar charts (vertical and horizontal)
- Line charts with multiple series
- Heatmaps for multi-dimensional analysis
- Scatter plots
- Pie charts
- Multi-chart dashboards

**Requirements**: `pip install matplotlib seaborn`

```bash
python visualization_demo.py
```

Outputs PNG files:
- `sales_by_region.png`
- `quarterly_trend.png`
- `product_performance.png`
- `sales_heatmap.png`
- `sales_vs_quantity.png`
- `market_share.png`
- `sales_dashboard.png`

### 5. Serialization

**serialization_demo.py** - Save and load cubes
- Saving cubes to disk (Parquet format)
- Loading cubes from saved state
- Exporting data with different compression formats
- Why Parquet is better than pickle for OLAP data

```bash
python serialization_demo.py
```

### 6. Jupyter Tutorial

**elasticube_tutorial.ipynb** - Interactive notebook tutorial
- Complete walkthrough of ElastiCube features
- OLAP operations (slice, dice, drill-down, roll-up)
- Interactive visualizations
- Integration with Pandas and Polars

**Requirements**: `pip install jupyter matplotlib`

```bash
jupyter notebook elasticube_tutorial.ipynb
```

## Sample Data

All examples use `sales_data.csv`, which contains sample sales transactions with the following structure:

| Column   | Type    | Description           |
|----------|---------|------------------------|
| region   | string  | Sales region          |
| product  | string  | Product name          |
| category | string  | Product category      |
| sales    | float   | Sales amount ($)      |
| quantity | integer | Quantity sold         |
| year     | integer | Year                  |
| quarter  | integer | Quarter (1-4)         |

## Key Features Demonstrated

### OLAP Operations
- **Slice**: Filter on a single dimension
- **Dice**: Filter on multiple dimensions
- **Drill-down**: Increase granularity (region → product)
- **Roll-up**: Decrease granularity (product → region)

### Query Capabilities
- Column selection with aggregations
- Filtering with SQL-style WHERE clauses
- Multi-dimensional grouping
- Sorting with custom ordering
- Result limiting and pagination

### Performance Features
- Apache Arrow columnar storage
- DataFusion query optimization
- Zero-copy data transfers
- Parallel query execution
- Polars integration for maximum performance

### Integration
- **Pandas**: Traditional DataFrame operations
- **Polars**: High-performance alternative (642x faster conversions)
- **PyArrow**: Direct Arrow table access
- **Matplotlib/Seaborn**: Publication-quality visualizations
- **Jupyter**: Rich HTML display in notebooks

## Performance Tips

1. **Use Polars for large datasets**:
   ```python
   df = query.to_polars()  # Much faster than to_pandas()
   ```

2. **Filter early**:
   ```python
   query.filter("sales > 1000")  # Before grouping
   query.group_by(["region"])
   ```

3. **Select only needed columns**:
   ```python
   query.select(["region", "SUM(sales)"])  # Don't select *
   ```

4. **Use PyArrow Tables directly when possible**:
   ```python
   table = query.execute()  # Fastest, no conversion
   ```

5. **Save cubes for reuse**:
   ```python
   cube.save("my_cube.cube")  # Faster than rebuilding
   ```

## Troubleshooting

### Import Error: pyarrow not found
```bash
pip install pyarrow
```

### Import Error: polars not found
```bash
pip install polars
```

### Import Error: matplotlib not found
```bash
pip install matplotlib seaborn
```

### Jupyter kernel not found
```bash
pip install jupyter ipykernel
python -m ipykernel install --user
```

## Next Steps

1. Try the examples with your own data
2. Experiment with different aggregation functions
3. Create custom visualizations
4. Build hierarchies for time-based analysis
5. Combine multiple cubes for advanced analytics

## Resources

- [ElastiCube Documentation](../../README.md)
- [Apache Arrow](https://arrow.apache.org/)
- [DataFusion](https://arrow.apache.org/datafusion/)
- [Polars](https://www.pola.rs/)
- [Matplotlib](https://matplotlib.org/)

## Feedback

Found an issue or have a suggestion? Please open an issue on GitHub!
