//! Query API for ElastiCube
//!
//! Provides a fluent API for building and executing analytical queries
//! against ElastiCube data using Apache DataFusion.

use crate::cache::{QueryCache, QueryCacheKey};
use crate::cube::ElastiCube;
use crate::error::{Error, Result};
use crate::optimization::OptimizationConfig;
use arrow::record_batch::RecordBatch;
use datafusion::datasource::MemTable;
use datafusion::prelude::*;
use std::sync::Arc;

/// Query builder for ElastiCube queries
///
/// Provides a fluent API for building and executing queries against a cube.
/// Supports both SQL queries and a DataFrame-style fluent API.
///
/// # Examples
///
/// ```rust,ignore
/// // SQL query
/// let results = cube.query()
///     .sql("SELECT region, SUM(sales) FROM cube GROUP BY region")
///     .execute()
///     .await?;
///
/// // Fluent API query
/// let results = cube.query()
///     .select(&["region", "SUM(sales) as total_sales"])
///     .filter("sales > 1000")
///     .group_by(&["region"])
///     .order_by(&["total_sales DESC"])
///     .limit(10)
///     .execute()
///     .await?;
/// ```
pub struct QueryBuilder {
    /// Reference to the parent cube
    cube: Arc<ElastiCube>,

    /// DataFusion SessionContext for query execution
    ctx: SessionContext,

    /// Optimization configuration
    #[allow(dead_code)] // Used for creating SessionContext, may be used in future features
    config: OptimizationConfig,

    /// Optional query cache
    cache: Option<Arc<QueryCache>>,

    /// Optional SQL query string (takes precedence over fluent API)
    sql_query: Option<String>,

    /// SELECT columns/expressions
    select_exprs: Vec<String>,

    /// WHERE filter condition
    filter_expr: Option<String>,

    /// GROUP BY columns
    group_by_exprs: Vec<String>,

    /// ORDER BY expressions
    order_by_exprs: Vec<String>,

    /// LIMIT clause
    limit_count: Option<usize>,

    /// OFFSET clause
    offset_count: Option<usize>,
}

impl QueryBuilder {
    /// Create a new query builder for the given cube
    pub(crate) fn new(cube: Arc<ElastiCube>) -> Result<Self> {
        Self::with_config(cube, OptimizationConfig::default())
    }

    /// Create a new query builder with custom optimization configuration
    pub(crate) fn with_config(cube: Arc<ElastiCube>, config: OptimizationConfig) -> Result<Self> {
        // Create SessionContext with optimization settings
        let session_config = config.to_session_config();
        let runtime_env = config.to_runtime_env();
        let ctx = SessionContext::new_with_config_rt(session_config, runtime_env);

        // Create query cache if enabled
        let cache = if config.enable_query_cache {
            Some(Arc::new(QueryCache::new(config.max_cache_entries)))
        } else {
            None
        };

        Ok(Self {
            cube,
            ctx,
            config,
            cache,
            sql_query: None,
            select_exprs: Vec::new(),
            filter_expr: None,
            group_by_exprs: Vec::new(),
            order_by_exprs: Vec::new(),
            limit_count: None,
            offset_count: None,
        })
    }

    /// Execute a raw SQL query
    ///
    /// # Arguments
    /// * `query` - SQL query string (can reference the cube as "cube")
    ///
    /// # Example
    /// ```rust,ignore
    /// let results = cube.query()
    ///     .sql("SELECT region, SUM(sales) as total FROM cube GROUP BY region")
    ///     .execute()
    ///     .await?;
    /// ```
    pub fn sql(mut self, query: impl Into<String>) -> Self {
        self.sql_query = Some(query.into());
        self
    }

    /// Select specific columns or expressions
    ///
    /// # Arguments
    /// * `columns` - Column names or SQL expressions
    ///
    /// # Example
    /// ```rust,ignore
    /// .select(&["region", "product", "SUM(sales) as total_sales"])
    /// ```
    pub fn select(mut self, columns: &[impl AsRef<str>]) -> Self {
        self.select_exprs = columns.iter().map(|c| c.as_ref().to_string()).collect();
        self
    }

    /// Add a WHERE filter condition
    ///
    /// # Arguments
    /// * `condition` - SQL filter expression
    ///
    /// # Example
    /// ```rust,ignore
    /// .filter("sales > 1000 AND region = 'North'")
    /// ```
    pub fn filter(mut self, condition: impl Into<String>) -> Self {
        self.filter_expr = Some(condition.into());
        self
    }

    /// Add WHERE filter (alias for filter)
    pub fn where_clause(self, condition: impl Into<String>) -> Self {
        self.filter(condition)
    }

    /// Group by columns
    ///
    /// # Arguments
    /// * `columns` - Column names to group by
    ///
    /// # Example
    /// ```rust,ignore
    /// .group_by(&["region", "product"])
    /// ```
    pub fn group_by(mut self, columns: &[impl AsRef<str>]) -> Self {
        self.group_by_exprs = columns.iter().map(|c| c.as_ref().to_string()).collect();
        self
    }

    /// Order results by columns
    ///
    /// # Arguments
    /// * `columns` - Column names with optional ASC/DESC
    ///
    /// # Example
    /// ```rust,ignore
    /// .order_by(&["total_sales DESC", "region ASC"])
    /// ```
    pub fn order_by(mut self, columns: &[impl AsRef<str>]) -> Self {
        self.order_by_exprs = columns.iter().map(|c| c.as_ref().to_string()).collect();
        self
    }

    /// Limit the number of results
    ///
    /// # Example
    /// ```rust,ignore
    /// .limit(100)
    /// ```
    pub fn limit(mut self, count: usize) -> Self {
        self.limit_count = Some(count);
        self
    }

    /// Skip a number of results
    ///
    /// # Example
    /// ```rust,ignore
    /// .offset(50)
    /// ```
    pub fn offset(mut self, count: usize) -> Self {
        self.offset_count = Some(count);
        self
    }

    /// OLAP Operation: Slice - filter on a single dimension
    ///
    /// # Example
    /// ```rust,ignore
    /// .slice("region", "North")
    /// ```
    pub fn slice(self, dimension: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        let condition = format!("{} = '{}'", dimension.as_ref(), value.as_ref());
        self.filter(condition)
    }

    /// OLAP Operation: Dice - filter on multiple dimensions
    ///
    /// # Example
    /// ```rust,ignore
    /// .dice(&[("region", "North"), ("product", "Widget")])
    /// ```
    pub fn dice(self, filters: &[(impl AsRef<str>, impl AsRef<str>)]) -> Self {
        let conditions: Vec<String> = filters
            .iter()
            .map(|(dim, val)| format!("{} = '{}'", dim.as_ref(), val.as_ref()))
            .collect();
        let combined = conditions.join(" AND ");
        self.filter(combined)
    }

    /// OLAP Operation: Drill-down - navigate down a hierarchy
    ///
    /// This selects data at a more granular level by including a lower-level dimension.
    ///
    /// # Example
    /// ```rust,ignore
    /// // Drill down from year to month
    /// .drill_down("year", &["year", "month"])
    /// ```
    pub fn drill_down(
        mut self,
        _parent_level: impl AsRef<str>,
        child_levels: &[impl AsRef<str>],
    ) -> Self {
        // Add child levels to GROUP BY
        self.group_by_exprs
            .extend(child_levels.iter().map(|c| c.as_ref().to_string()));
        self
    }

    /// OLAP Operation: Roll-up - aggregate across dimensions
    ///
    /// This aggregates data by removing one or more dimensions from grouping.
    ///
    /// # Example
    /// ```rust,ignore
    /// .roll_up(&["region"]) // Aggregate across all regions
    /// ```
    pub fn roll_up(mut self, dimensions_to_remove: &[impl AsRef<str>]) -> Self {
        let to_remove: Vec<String> = dimensions_to_remove
            .iter()
            .map(|d| d.as_ref().to_string())
            .collect();

        self.group_by_exprs
            .retain(|col| !to_remove.contains(col));
        self
    }

    /// Execute the query and return results
    ///
    /// # Returns
    /// A QueryResult containing the data and metadata
    pub async fn execute(mut self) -> Result<QueryResult> {
        // Build the query SQL string for caching
        let query_sql = if let Some(sql) = &self.sql_query {
            sql.clone()
        } else {
            self.build_sql_query()
        };

        // Check cache if enabled
        if let Some(cache) = &self.cache {
            let cache_key = QueryCacheKey::new(&query_sql);
            if let Some(cached_result) = cache.get(&cache_key) {
                return Ok(cached_result);
            }
        }

        // Register the cube data as a MemTable
        self.register_cube_data().await?;

        // Execute the query
        let dataframe = if let Some(sql) = &self.sql_query {
            // Execute raw SQL query
            self.execute_sql(sql).await?
        } else {
            // Build and execute fluent API query
            self.execute_fluent_query().await?
        };

        // Collect results
        let batches = dataframe
            .collect()
            .await
            .map_err(|e| Error::query(format!("Failed to collect query results: {}", e)))?;

        let row_count = batches.iter().map(|b| b.num_rows()).sum();

        let result = QueryResult {
            batches,
            row_count,
        };

        // Cache the result if caching is enabled
        if let Some(cache) = &self.cache {
            let cache_key = QueryCacheKey::new(&query_sql);
            cache.put(cache_key, result.clone());
        }

        Ok(result)
    }

    /// Register cube data as a DataFusion MemTable
    async fn register_cube_data(&mut self) -> Result<()> {
        let schema = self.cube.arrow_schema().clone();
        let data = self.cube.data().to_vec();

        // MemTable expects Vec<Vec<RecordBatch>> (partitions)
        // We'll use a single partition with all our batches
        let partitions = vec![data];

        let mem_table = MemTable::try_new(schema, partitions)
            .map_err(|e| Error::query(format!("Failed to create MemTable: {}", e)))?;

        self.ctx
            .register_table("cube", Arc::new(mem_table))
            .map_err(|e| Error::query(format!("Failed to register table: {}", e)))?;

        Ok(())
    }

    /// Execute a raw SQL query
    async fn execute_sql(&self, query: &str) -> Result<DataFrame> {
        self.ctx
            .sql(query)
            .await
            .map_err(|e| Error::query(format!("SQL execution failed: {}", e)))
    }

    /// Expand calculated fields in an expression
    ///
    /// Replaces references to calculated measures and virtual dimensions
    /// with their underlying expressions. Performs recursive expansion
    /// to handle nested calculated fields.
    fn expand_calculated_fields(&self, expr: &str) -> String {
        let mut expanded = expr.to_string();
        let schema = self.cube.schema();

        // Keep expanding until no more changes occur (handles nested calculated fields)
        // Use a maximum iteration count to prevent infinite loops
        const MAX_ITERATIONS: usize = 10;
        for _ in 0..MAX_ITERATIONS {
            let before = expanded.clone();

            // Expand virtual dimensions first (they can be used in calculated measures)
            for vdim in schema.virtual_dimensions() {
                let pattern = vdim.name();
                // Use word boundaries to avoid partial matches
                // e.g., don't replace "year" in "yearly_sales"
                let regex_pattern = format!(r"\b{}\b", regex::escape(pattern));
                if let Ok(re) = regex::Regex::new(&regex_pattern) {
                    let replacement = format!("({})", vdim.expression());
                    expanded = re.replace_all(&expanded, replacement.as_str()).to_string();
                }
            }

            // Expand calculated measures
            for calc_measure in schema.calculated_measures() {
                let pattern = calc_measure.name();
                let regex_pattern = format!(r"\b{}\b", regex::escape(pattern));
                if let Ok(re) = regex::Regex::new(&regex_pattern) {
                    let replacement = format!("({})", calc_measure.expression());
                    expanded = re.replace_all(&expanded, replacement.as_str()).to_string();
                }
            }

            // If no changes were made, we're done
            if expanded == before {
                break;
            }
        }

        expanded
    }

    /// Build SQL query string from fluent API parameters
    fn build_sql_query(&self) -> String {
        let mut query_str = String::from("SELECT ");

        // SELECT clause - expand calculated fields
        if self.select_exprs.is_empty() {
            query_str.push('*');
        } else {
            let expanded_selects: Vec<String> = self
                .select_exprs
                .iter()
                .map(|expr| self.expand_calculated_fields(expr))
                .collect();
            query_str.push_str(&expanded_selects.join(", "));
        }

        query_str.push_str(" FROM cube");

        // WHERE clause - expand calculated fields
        if let Some(filter) = &self.filter_expr {
            query_str.push_str(" WHERE ");
            let expanded_filter = self.expand_calculated_fields(filter);
            query_str.push_str(&expanded_filter);
        }

        // GROUP BY clause - expand calculated fields
        if !self.group_by_exprs.is_empty() {
            query_str.push_str(" GROUP BY ");
            let expanded_groups: Vec<String> = self
                .group_by_exprs
                .iter()
                .map(|expr| self.expand_calculated_fields(expr))
                .collect();
            query_str.push_str(&expanded_groups.join(", "));
        }

        // ORDER BY clause - expand calculated fields
        if !self.order_by_exprs.is_empty() {
            query_str.push_str(" ORDER BY ");
            let expanded_orders: Vec<String> = self
                .order_by_exprs
                .iter()
                .map(|expr| self.expand_calculated_fields(expr))
                .collect();
            query_str.push_str(&expanded_orders.join(", "));
        }

        // LIMIT clause
        if let Some(limit) = self.limit_count {
            query_str.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET clause
        if let Some(offset) = self.offset_count {
            query_str.push_str(&format!(" OFFSET {}", offset));
        }

        query_str
    }

    /// Build and execute a fluent API query
    async fn execute_fluent_query(&self) -> Result<DataFrame> {
        let query_str = self.build_sql_query();
        self.execute_sql(&query_str).await
    }
}

/// Query result containing the executed query data
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Result data as Arrow RecordBatches
    batches: Vec<RecordBatch>,

    /// Total number of rows in the result
    row_count: usize,
}

impl QueryResult {
    /// Create a new QueryResult (for testing purposes)
    #[cfg(test)]
    pub(crate) fn new_for_testing(batches: Vec<RecordBatch>, row_count: usize) -> Self {
        Self {
            batches,
            row_count,
        }
    }

    /// Get the result batches
    pub fn batches(&self) -> &[RecordBatch] {
        &self.batches
    }

    /// Get the total number of rows
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    /// Check if the result is empty
    pub fn is_empty(&self) -> bool {
        self.row_count == 0
    }

    /// Get a pretty-printed string representation of the results
    ///
    /// Useful for debugging and testing
    pub fn pretty_print(&self) -> Result<String> {
        use arrow::util::pretty::pretty_format_batches;

        pretty_format_batches(&self.batches)
            .map(|display| display.to_string())
            .map_err(|e| Error::query(format!("Failed to format results: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::ElastiCubeBuilder;
    use crate::cube::AggFunc;
    use arrow::array::{Float64Array, Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema as ArrowSchema};

    fn create_test_cube() -> Result<ElastiCube> {
        // Create test data
        let schema = Arc::new(ArrowSchema::new(vec![
            Field::new("region", DataType::Utf8, false),
            Field::new("product", DataType::Utf8, false),
            Field::new("sales", DataType::Float64, false),
            Field::new("quantity", DataType::Int32, false),
        ]));

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec![
                    "North", "South", "North", "East", "South",
                ])),
                Arc::new(StringArray::from(vec![
                    "Widget", "Widget", "Gadget", "Widget", "Gadget",
                ])),
                Arc::new(Float64Array::from(vec![100.0, 200.0, 150.0, 175.0, 225.0])),
                Arc::new(Int32Array::from(vec![10, 20, 15, 17, 22])),
            ],
        )
        .unwrap();

        ElastiCubeBuilder::new("test_cube")
            .add_dimension("region", DataType::Utf8)?
            .add_dimension("product", DataType::Utf8)?
            .add_measure("sales", DataType::Float64, AggFunc::Sum)?
            .add_measure("quantity", DataType::Int32, AggFunc::Sum)?
            .load_record_batches(schema, vec![batch])?
            .build()
    }

    #[tokio::test]
    async fn test_query_select_all() {
        let cube = create_test_cube().unwrap();
        let arc_cube = Arc::new(cube);

        let result = arc_cube.query().unwrap().execute().await.unwrap();

        assert_eq!(result.row_count(), 5);
        assert_eq!(result.batches().len(), 1);
    }

    #[tokio::test]
    async fn test_query_select_columns() {
        let cube = create_test_cube().unwrap();
        let arc_cube = Arc::new(cube);

        let result = arc_cube
            .query()
            .unwrap()
            .select(&["region", "sales"])
            .execute()
            .await
            .unwrap();

        assert_eq!(result.row_count(), 5);
        // Check that we only got 2 columns
        assert_eq!(result.batches()[0].num_columns(), 2);
    }

    #[tokio::test]
    async fn test_query_filter() {
        let cube = create_test_cube().unwrap();
        let arc_cube = Arc::new(cube);

        let result = arc_cube
            .query()
            .unwrap()
            .filter("sales > 150")
            .execute()
            .await
            .unwrap();

        assert_eq!(result.row_count(), 3); // 200, 175, 225
    }

    #[tokio::test]
    async fn test_query_group_by() {
        let cube = create_test_cube().unwrap();
        let arc_cube = Arc::new(cube);

        let result = arc_cube
            .query()
            .unwrap()
            .select(&["region", "SUM(sales) as total_sales"])
            .group_by(&["region"])
            .execute()
            .await
            .unwrap();

        assert_eq!(result.row_count(), 3); // North, South, East
    }

    #[tokio::test]
    async fn test_query_order_by() {
        let cube = create_test_cube().unwrap();
        let arc_cube = Arc::new(cube);

        let result = arc_cube
            .query()
            .unwrap()
            .select(&["region", "sales"])
            .order_by(&["sales DESC"])
            .execute()
            .await
            .unwrap();

        assert_eq!(result.row_count(), 5);
        // First row should have highest sales (225)
    }

    #[tokio::test]
    async fn test_query_limit() {
        let cube = create_test_cube().unwrap();
        let arc_cube = Arc::new(cube);

        let result = arc_cube
            .query()
            .unwrap()
            .limit(3)
            .execute()
            .await
            .unwrap();

        assert_eq!(result.row_count(), 3);
    }

    #[tokio::test]
    async fn test_query_sql() {
        let cube = create_test_cube().unwrap();
        let arc_cube = Arc::new(cube);

        let result = arc_cube
            .query()
            .unwrap()
            .sql("SELECT region, SUM(sales) as total FROM cube GROUP BY region ORDER BY total DESC")
            .execute()
            .await
            .unwrap();

        assert_eq!(result.row_count(), 3);
    }

    #[tokio::test]
    async fn test_olap_slice() {
        let cube = create_test_cube().unwrap();
        let arc_cube = Arc::new(cube);

        let result = arc_cube
            .query()
            .unwrap()
            .slice("region", "North")
            .execute()
            .await
            .unwrap();

        assert_eq!(result.row_count(), 2); // 2 North entries
    }

    #[tokio::test]
    async fn test_olap_dice() {
        let cube = create_test_cube().unwrap();
        let arc_cube = Arc::new(cube);

        let result = arc_cube
            .query()
            .unwrap()
            .dice(&[("region", "North"), ("product", "Widget")])
            .execute()
            .await
            .unwrap();

        assert_eq!(result.row_count(), 1); // 1 North Widget
    }

    #[tokio::test]
    async fn test_complex_query() {
        let cube = create_test_cube().unwrap();
        let arc_cube = Arc::new(cube);

        let result = arc_cube
            .query()
            .unwrap()
            .select(&["region", "product", "SUM(sales) as total_sales", "AVG(quantity) as avg_qty"])
            .filter("sales > 100")
            .group_by(&["region", "product"])
            .order_by(&["total_sales DESC"])
            .limit(5)
            .execute()
            .await
            .unwrap();

        assert!(result.row_count() > 0);
    }
}
