Here is a Product Requirement Document (PRD) for a **Mini OLAP (Online Analytical Processing) Database** built in Rust, designed specifically for educational purposes.

---

# Product Requirement Document: "RustyCube" (Mini OLAP DB)

## 1. Project Overview

**RustyCube** is a lightweight, in-memory OLAP database engine implemented in Rust. Its primary purpose is educational: to demonstrate the core principles of column-oriented storage, query execution, and aggregation without the complexities of a production-grade distributed system.

### Goals

* **Learnability:** The codebase must be clean, well-documented, and readable for intermediate Rust developers.
* **Performance Awareness:** Demonstrate why column-stores differ from row-stores (OLTP) regarding analytical queries.
* **Zero External Dependencies (Core):** Minimize external crates for core logic to show "how things work under the hood" (e.g., implementing a simple parser vs. using a heavy library).

---

## 2. Target Audience

* Students learning database internals.
* Rust developers looking to understand systems programming and memory management.
* Data engineers curious about how engines like ClickHouse or Apache Druid work internally.

---

## 3. Key Features (MVP Scope)

### 3.1 Storage Engine

* **Columnar Storage:** Data is stored by column, not by row.
* **Primitive Types:** Support for `Int64`, `Float64`, and `String` (UTF-8).
* **Immutable Segments:** Data is loaded in batches; segments are immutable once written (simplifies concurrency).

### 3.2 Query Engine

* **Vectorized Execution:** Operations process chunks of data (vectors) rather than single values to utilize CPU cache efficiently.
* **Basic SQL-like Syntax:** A very simple parser to handle `SELECT`, `FROM`, `WHERE`, and `GROUP BY`.
* **Aggregations:** Support for `COUNT`, `SUM`, `MIN`, `MAX`, and `AVG`.

### 3.3 Interface

* **REPL (Read-Eval-Print Loop):** A command-line interface to input queries and view results.
* **CSV Ingestion:** Ability to load data from standard CSV files.

---

## 4. Technical Architecture

### 4.1 Technology Stack

* **Language:** Rust (Stable)
* **Error Handling:** `thiserror` or `anyhow` (for simplicity).
* **Serialization:** `serde` (for CSV parsing).
* **CLI:** `clap` or `rustyline`.
* **Testing:** Standard `cargo test` suite.

### 4.2 Core Components

#### A. The Catalog

Manages metadata: table names, column names, and data types.

#### B. The Storage Layer (Columnar)

Instead of a `Vec<Struct>`, we use a `Struct<Vec>`.

* **Concept:** A `DataFrame` or `Table` struct holding a collection of `Series` (columns).
* **Compression (Optional for v2):** Dictionary encoding for strings.

#### C. The Execution Engine

* **Physical Plan:** A tree of operators (Scan -> Filter -> Aggregate -> Project).
* **Volcano Model (Batch variation):** Each operator calls `next_batch()` on its child to pull data.

---

## 5. Data Flow Example

1. **Ingestion:** User runs `LOAD 'data.csv' INTO table_users`.
2. **Parsing:** The system reads the CSV, transposes rows into columns, and stores them in memory vectors.
3. **Query:** User runs `SELECT city, COUNT(*) FROM table_users GROUP BY city`.
4. **Execution:**
* **Scan:** Reads the `city` column.
* **Aggregate:** Hash aggregation maps `city` strings to counters.


5. **Output:** Returns a formatted ASCII table of results.

---

## 6. Implementation Roadmap

### Phase 1: The Foundation (Weeks 1-2)

* Define the `Column` trait and concrete types (`IntColumn`, `StringColumn`).
* Implement a simple `Table` struct that holds columns.
* Create a manual "hard-coded" query test (e.g., iterate a column and sum it without SQL).

### Phase 2: The Ingestion (Week 3)

* Implement CSV reading.
* Handle type inference (guess if a CSV column is Int or String).

### Phase 3: The Query Engine (Weeks 4-5)

* Implement physical operators: `TableScan`, `Filter` (Selection), `Project`.
* Implement `GroupBy` using a `HashMap`.

### Phase 4: The Interface (Week 6)

* Build the SQL Parser (using a parser combinator like `nom` or a handwritten recursive descent parser for learning).
* Build the REPL.

---

## 7. Success Metrics

* **Code Clarity:** Can a new developer understand the `Filter` operator in under 10 minutes?
* **Correctness:** Does `SUM(column)` match the result of a spreadsheet calculation?
* **Memory Safety:** No `unsafe` blocks unless absolutely justified and documented.
