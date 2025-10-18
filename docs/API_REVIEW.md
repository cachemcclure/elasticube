# ElastiCube API Review & Stabilization Plan

**Version**: 0.1.0 → 0.2.0 (Pre-1.0 Development)
**Date**: 2025-10-18
**Status**: Phase 8.1 - API Stabilization

## Executive Summary

This document provides a comprehensive review of the ElastiCube public API against the official [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/). The review identifies strengths, weaknesses, and recommendations for stabilizing the API before a 1.0 release.

**Overall Assessment**: The API is well-designed with good ergonomics, comprehensive documentation, and proper use of Rust idioms. Several minor improvements are recommended to ensure long-term stability and compatibility.

---

## Table of Contents

1. [API Surface Overview](#api-surface-overview)
2. [Rust API Guidelines Compliance](#rust-api-guidelines-compliance)
3. [Identified Issues](#identified-issues)
4. [Recommendations](#recommendations)
5. [Breaking vs. Non-Breaking Changes](#breaking-vs-non-breaking-changes)
6. [Migration Path](#migration-path)

---

## API Surface Overview

### Public Modules (from `lib.rs`)

```rust
pub mod builder;      // ElastiCubeBuilder
pub mod cache;        // QueryCache, CacheStats
pub mod cube;         // ElastiCube, Dimension, Measure, etc.
pub mod error;        // Error, Result
pub mod optimization; // OptimizationConfig, CubeStatistics
pub mod query;        // QueryBuilder, QueryResult
pub mod storage;      // (minimal public surface)
pub mod sources;      // DataSource trait, CsvSource, etc.
```

### Re-exported Types

**Core Types**:
- `ElastiCubeBuilder` - Main builder for constructing cubes
- `ElastiCube` - The multidimensional data structure
- `Dimension`, `Measure`, `Hierarchy` - Schema components
- `CalculatedMeasure`, `VirtualDimension` - Computed fields
- `CubeSchema` - Schema metadata

**Query Types**:
- `QueryBuilder` - Fluent query API
- `QueryResult` - Query execution results

**Performance Types**:
- `OptimizationConfig` - Query optimization settings
- `CubeStatistics`, `ColumnStatistics` - Performance metrics
- `QueryCache`, `CacheStats`, `QueryCacheKey` - Result caching

**Data Source Types**:
- `DataSource` (trait) - Extensible data loading
- `CsvSource`, `ParquetSource`, `JsonSource`, `RecordBatchSource` - File sources
- `PostgresSource`, `MySqlSource`, `OdbcSource` (feature-gated)
- `RestApiSource` (feature-gated)
- `S3Source`, `GcsSource`, `AzureSource` (feature-gated)

**Error Handling**:
- `Error` - Comprehensive error enum
- `Result<T>` - Convenience type alias

---

## Rust API Guidelines Compliance

### ✅ Strengths

#### 1. **Naming (C-CASE)**
- ✅ All types use `PascalCase` (e.g., `ElastiCube`, `QueryBuilder`)
- ✅ All methods use `snake_case` (e.g., `add_dimension`, `load_csv`)
- ✅ Type conversions follow `to_*`, `as_*`, `into_*` patterns
- ✅ Getter methods use bare names (e.g., `schema()`, not `get_schema()`)

#### 2. **Interoperability (C-COMMON-TRAITS)**
- ✅ Error type implements `Error`, `Debug`, `Display` via `thiserror`
- ✅ Most types implement `Debug`
- ✅ `Clone` implemented where appropriate (`ElastiCube`, config types)
- ⚠️ Missing `Clone` on some types that might benefit from it

#### 3. **Predictability (C-SMART-PTR)**
- ✅ Uses `Arc<ElastiCube>` for shared ownership
- ✅ Uses `Arc<ArrowSchema>` for schema sharing
- ✅ Proper use of `&self` vs `&mut self` vs `self`

#### 4. **Flexibility (C-INTERMEDIATE)**
- ✅ Builder pattern for complex construction
- ✅ Fluent API for queries
- ✅ Trait-based extensibility (`DataSource`)
- ✅ Feature flags for optional dependencies

#### 5. **Type Safety (C-NEWTYPE)**
- ✅ `QueryCacheKey` is a newtype wrapper
- ✅ Strong typing for dimensions, measures, hierarchies
- ⚠️ Consider newtype for batch_size, memory_limit (currently raw `usize`)

#### 6. **Dependability (C-FAILURE)**
- ✅ Uses `Result` types consistently
- ✅ Error types are descriptive with context
- ✅ Uses `thiserror` for ergonomic error handling
- ⚠️ Some public methods use `unwrap()` internally (needs review)

#### 7. **Debuggability (C-DEBUG)**
- ✅ All public types derive `Debug`
- ✅ Error messages are descriptive
- ✅ `Display` implementations for stats types

#### 8. **Documentation (C-DOCS)**
- ✅ Comprehensive rustdoc comments on public items
- ✅ Examples in doc comments
- ✅ Module-level documentation
- ✅ Top-level crate documentation
- ⚠️ Some error variants lack specific documentation

---

### ⚠️ Areas for Improvement

#### 1. **Future-Proofing (C-STRUCT-PRIVATE)**

**Issue**: Error enum is not marked `#[non_exhaustive]`

```rust
// Current
pub enum Error {
    Arrow(#[from] arrow::error::ArrowError),
    DataFusion(#[from] datafusion::error::DataFusionError),
    // ... 12+ variants
}
```

**Impact**: Adding new error variants would be a breaking change

**Recommendation**: Add `#[non_exhaustive]` attribute

```rust
#[non_exhaustive]
pub enum Error {
    // ... existing variants
}
```

#### 2. **Builder Return Types (C-BUILDER-FALLIBLE)**

**Issue**: Some builder methods return `Result<Self>` while others return `Self`

```rust
// Inconsistent:
pub fn add_dimension(self, ...) -> Result<Self>  // Returns Result
pub fn load_csv(self, ...) -> Self               // Returns Self
```

**Impact**: Inconsistent error handling expectations

**Recommendation**: Standardize on one approach or document the distinction clearly

#### 3. **Public Constructors (C-CTOR)**

**Issue**: `QueryBuilder::new()` is `pub(crate)` but `QueryBuilder` is public

```rust
// QueryBuilder is public but can't be constructed directly
pub struct QueryBuilder { ... }

impl QueryBuilder {
    pub(crate) fn new(cube: Arc<ElastiCube>) -> Result<Self>
}
```

**Impact**: Users can't directly construct `QueryBuilder` (must go through `cube.query()`)

**Status**: This is intentional (controlled construction), but should be documented

#### 4. **Arc-Based API (C-SELF-ARC)**

**Issue**: `ElastiCube::query()` requires `Arc<Self>`

```rust
pub fn query(self: Arc<Self>) -> Result<QueryBuilder>
```

**Impact**: Users must wrap their cube in `Arc` to query it

**Current Workaround**: Documented in examples

**Consider**: Alternative API that works with `&self` for convenience

#### 5. **Trait Implementation Gaps**

**Missing trait implementations**:
- `ElastiCube`: Missing `Eq`, `PartialEq` (contains `RecordBatch` which doesn't impl Eq)
- `OptimizationConfig`: Could implement `Eq`, `PartialEq`
- `CacheStats`: Could implement `PartialEq`

**Recommendation**: Implement where reasonable, document why not implemented otherwise

#### 6. **Type Conversion Methods (C-CONV)**

**Issue**: Some conversions use builder patterns, others use `From`/`Into`

```rust
// Current
impl OptimizationConfig {
    pub fn to_session_config(&self) -> SessionConfig { ... }
    pub fn to_runtime_env(&self) -> Arc<RuntimeEnv> { ... }
}
```

**Consideration**: These are appropriate as they're not pure conversions

---

## Identified Issues

### Critical (Must Fix Before 1.0)

1. **Error Enum Extensibility**
   - Priority: High
   - Add `#[non_exhaustive]` to `Error` enum
   - Impact: Allows adding error variants without breaking changes

2. **Public API Documentation Gaps**
   - Priority: High
   - Document why `QueryBuilder` can't be directly constructed
   - Add examples for all feature-gated APIs
   - Document the Arc requirement for `query()`

### Important (Should Fix Before 1.0)

3. **Builder Consistency**
   - Priority: Medium
   - Standardize error handling in builder methods
   - Consider: `load_*` methods could return `Result<Self>` for consistency

4. **Trait Implementations**
   - Priority: Medium
   - Add `PartialEq` where appropriate
   - Consider `Hash` for config types

5. **Data Source Trait Send Safety**
   - Priority: Medium
   - Currently marked as deferred in checklist
   - May be needed for async/parallel processing

### Nice to Have (Can Do Post-1.0)

6. **Convenience Methods**
   - Add non-Arc query method: `fn query_ref(&self) -> Result<QueryBuilder>`
   - Builder methods with default values

7. **Type Safety Enhancements**
   - Newtype wrappers for `batch_size`, `memory_limit`
   - Compile-time guarantees for builder state

---

## Recommendations

### Phase 8.1 Action Items

#### Immediate Changes (Non-Breaking, Safe to Make Now)

1. **Add `#[non_exhaustive]` to Error enum** ✅
   ```rust
   #[non_exhaustive]
   #[derive(Error, Debug)]
   pub enum Error { ... }
   ```

2. **Implement additional trait impls** ✅
   ```rust
   #[derive(Debug, Clone, PartialEq)]
   pub struct OptimizationConfig { ... }

   #[derive(Debug, Clone, PartialEq)]
   pub struct CacheStats { ... }
   ```

3. **Improve documentation** ✅
   - Add doc comments explaining Arc requirement
   - Document feature gate requirements
   - Add more comprehensive examples

4. **Add method documentation** ✅
   - Document all error variants
   - Add `# Errors` sections to fallible functions
   - Add `# Panics` sections where applicable

#### Pre-1.0 Considerations

5. **Evaluate Arc API** 🔍
   - Keep current design but add convenience wrapper?
   - Add `query_ref()` that internally uses Arc?

6. **Builder consistency** 🔍
   - Audit all builder methods for Result usage
   - Make load_* methods return Result<Self>?

#### Post-1.0 Enhancements

7. **Advanced type safety** (v2.0+)
   - Builder typestate pattern
   - Newtypes for numeric limits

---

## Breaking vs. Non-Breaking Changes

### ✅ Safe to Make Now (Non-Breaking)

- Adding `#[non_exhaustive]` to enums
- Adding trait implementations (`Clone`, `PartialEq`, etc.)
- Adding new public methods
- Improving documentation
- Adding convenience wrappers

### ⚠️ Breaking Changes (Defer Until Major Version)

- Changing method signatures
- Removing public items
- Changing error variants (if not non_exhaustive)
- Making load_* methods return Result
- Changing Arc requirements

---

## Semantic Versioning Plan

### Current: 0.1.0

- Pre-1.0, so we have flexibility
- Per Cargo SemVer: 0.y.z versions treat 'y' as major version
- Breaking changes allowed in minor versions (0.1 → 0.2)

### Recommended Version Path

**0.1.0** (Current)
- Phase 7 complete (testing, docs, examples)

**0.2.0** (Next - API Stabilization)
- Add `#[non_exhaustive]` to Error
- Implement additional traits
- Improve documentation
- Minor API refinements (if any breaking changes needed)

**0.3.0** (Polish & Testing)
- Address any feedback from 0.2
- Final API adjustments
- Extended real-world testing

**0.4.0** (Release Candidate)
- Feature freeze
- Extensive testing
- Documentation review
- Prepare migration guides

**1.0.0** (Stable Release)
- Stable API guarantees
- SemVer compatibility promises
- Long-term support commitment

---

## Migration Path

### For 0.1.0 → 0.2.0 Users

Most code will work unchanged. Key changes:

1. **Error Matching**:
   ```rust
   // Before: Exhaustive matching allowed
   match err {
       Error::Arrow(e) => ...,
       Error::DataFusion(e) => ...,
       // etc - must list all variants
   }

   // After: Must include catch-all
   match err {
       Error::Arrow(e) => ...,
       Error::DataFusion(e) => ...,
       _ => ...,  // Required due to #[non_exhaustive]
   }
   ```

2. **Documentation improvements** - no code changes needed

3. **New trait implementations** - enables new use cases, no breaking changes

---

## Conclusion

The ElastiCube public API is **production-ready** with minor improvements needed for long-term stability. The recommended changes are mostly non-breaking and can be implemented in version 0.2.0.

### Key Strengths
- Well-designed builder and fluent APIs
- Comprehensive error handling
- Good documentation coverage
- Proper use of Rust idioms
- Extensible design with traits

### Priority Actions
1. ✅ Add `#[non_exhaustive]` to Error enum
2. ✅ Implement additional common traits
3. ✅ Improve documentation (Arc requirements, feature gates)
4. 🔍 Evaluate builder consistency
5. 🔍 Consider convenience APIs for Arc usage

### Readiness for 1.0
After addressing the recommendations in this document, the API will be ready for a 1.0 release with strong SemVer compatibility guarantees.

---

**Next Steps**: Implement Phase 8.1 changes → Release 0.2.0 → Gather feedback → Release 1.0.0
