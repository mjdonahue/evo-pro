//! Specialized query builders for repositories
//!
//! This module provides specialized query builders that make it easier to
//! construct complex queries in a type-safe way.

use sqlx::{QueryBuilder, Sqlite};
use std::fmt::Debug;

/// Condition operator for query building
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionOperator {
    /// Equal (=)
    Equal,
    /// Not equal (!=)
    NotEqual,
    /// Greater than (>)
    GreaterThan,
    /// Greater than or equal (>=)
    GreaterThanOrEqual,
    /// Less than (<)
    LessThan,
    /// Less than or equal (<=)
    LessThanOrEqual,
    /// Like (LIKE)
    Like,
    /// In (IN)
    In,
    /// Is null (IS NULL)
    IsNull,
    /// Is not null (IS NOT NULL)
    IsNotNull,
}

impl ConditionOperator {
    /// Get the SQL representation of the operator
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Equal => "=",
            Self::NotEqual => "!=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::Like => "LIKE",
            Self::In => "IN",
            Self::IsNull => "IS NULL",
            Self::IsNotNull => "IS NOT NULL",
        }
    }
}

/// Logical operator for combining conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOperator {
    /// AND
    And,
    /// OR
    Or,
}

impl LogicalOperator {
    /// Get the SQL representation of the operator
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::And => "AND",
            Self::Or => "OR",
        }
    }
}

/// Order direction for sorting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderDirection {
    /// Ascending
    Asc,
    /// Descending
    Desc,
}

impl OrderDirection {
    /// Get the SQL representation of the direction
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

/// Join type for SQL joins
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    /// INNER JOIN
    Inner,
    /// LEFT JOIN
    Left,
    /// RIGHT JOIN
    Right,
    /// FULL JOIN
    Full,
}

impl JoinType {
    /// Get the SQL representation of the join type
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Inner => "INNER JOIN",
            Self::Left => "LEFT JOIN",
            Self::Right => "RIGHT JOIN",
            Self::Full => "FULL JOIN",
        }
    }
}

/// Aggregation function for SQL aggregations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateFunction {
    /// COUNT
    Count,
    /// SUM
    Sum,
    /// AVG
    Avg,
    /// MIN
    Min,
    /// MAX
    Max,
    /// GROUP_CONCAT
    GroupConcat,
}

impl AggregateFunction {
    /// Get the SQL representation of the aggregation function
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Count => "COUNT",
            Self::Sum => "SUM",
            Self::Avg => "AVG",
            Self::Min => "MIN",
            Self::Max => "MAX",
            Self::GroupConcat => "GROUP_CONCAT",
        }
    }
}

/// Enhanced query builder that provides a more flexible and type-safe way to build queries
pub struct EnhancedQueryBuilder<'a> {
    /// The underlying SQLx query builder
    builder: QueryBuilder<'a, Sqlite>,
    /// Whether a WHERE clause has been added
    has_where: bool,
    /// Whether an ORDER BY clause has been added
    has_order_by: bool,
    /// Whether a LIMIT clause has been added
    has_limit: bool,
    /// Whether an OFFSET clause has been added
    has_offset: bool,
    /// Whether a JOIN clause has been added
    has_join: bool,
    /// Whether a GROUP BY clause has been added
    has_group_by: bool,
    /// Whether a HAVING clause has been added
    has_having: bool,
}

impl<'a> EnhancedQueryBuilder<'a> {
    /// Create a new enhanced query builder with the given base query
    pub fn new(base_query: &str) -> Self {
        Self {
            builder: QueryBuilder::new(base_query),
            has_where: false,
            has_order_by: false,
            has_limit: false,
            has_offset: false,
            has_join: false,
            has_group_by: false,
            has_having: false,
        }
    }

    /// Add a WHERE clause if one hasn't been added yet, otherwise add the given logical operator
    pub fn add_where_clause(&mut self, logical_op: Option<LogicalOperator>) -> &mut Self {
        if !self.has_where {
            self.builder.push(" WHERE ");
            self.has_where = true;
        } else if let Some(op) = logical_op {
            self.builder.push(format!(" {} ", op.as_sql()));
        }
        self
    }

    /// Add a condition to the query
    pub fn add_condition<T: Debug + sqlx::Type<Sqlite> + Send + 'a>(
        &mut self,
        field: &str,
        op: ConditionOperator,
        value: Option<T>,
        logical_op: Option<LogicalOperator>,
    ) -> &mut Self {
        // Skip if value is None, unless the operator is IsNull or IsNotNull
        if value.is_none() && op != ConditionOperator::IsNull && op != ConditionOperator::IsNotNull {
            return self;
        }

        self.add_where_clause(logical_op);

        match op {
            ConditionOperator::IsNull => {
                self.builder.push(format!("{} IS NULL", field));
            }
            ConditionOperator::IsNotNull => {
                self.builder.push(format!("{} IS NOT NULL", field));
            }
            ConditionOperator::In => {
                // For IN operator, value should be a collection
                // This is a simplified implementation
                self.builder.push(format!("{} IN (", field));
                self.builder.push_bind(value.unwrap());
                self.builder.push(")");
            }
            _ => {
                self.builder.push(format!("{} {} ", field, op.as_sql()));
                self.builder.push_bind(value.unwrap());
            }
        }

        self
    }

    /// Add an ORDER BY clause
    pub fn add_order_by(&mut self, field: &str, direction: OrderDirection) -> &mut Self {
        if !self.has_order_by {
            self.builder.push(" ORDER BY ");
            self.has_order_by = true;
        } else {
            self.builder.push(", ");
        }

        self.builder.push(format!("{} {}", field, direction.as_sql()));
        self
    }

    /// Add a LIMIT clause
    pub fn add_limit(&mut self, limit: i64) -> &mut Self {
        if !self.has_limit {
            self.builder.push(" LIMIT ");
            self.builder.push_bind(limit);
            self.has_limit = true;
        }
        self
    }

    /// Add an OFFSET clause
    pub fn add_offset(&mut self, offset: i64) -> &mut Self {
        if !self.has_offset {
            self.builder.push(" OFFSET ");
            self.builder.push_bind(offset);
            self.has_offset = true;
        }
        self
    }

    /// Build the query as a specific type
    pub fn build_query_as<T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin>(
        &'a self,
    ) -> sqlx::query::QueryAs<'a, Sqlite, T, sqlx::sqlite::SqliteArguments<'a>> {
        self.builder.build_query_as()
    }

    /// Build the query
    pub fn build_query(
        &'a self,
    ) -> sqlx::query::Query<'a, Sqlite, sqlx::sqlite::SqliteArguments<'a>> {
        self.builder.build()
    }

    /// Get a reference to the underlying SQLx query builder
    pub fn builder(&self) -> &QueryBuilder<'a, Sqlite> {
        &self.builder
    }

    /// Get a mutable reference to the underlying SQLx query builder
    pub fn builder_mut(&mut self) -> &mut QueryBuilder<'a, Sqlite> {
        &mut self.builder
    }

    /// Add a JOIN clause
    pub fn add_join(&mut self, join_type: JoinType, table: &str, on_condition: &str) -> &mut Self {
        self.builder.push(format!(" {} {} ON {}", join_type.as_sql(), table, on_condition));
        self.has_join = true;
        self
    }

    /// Add a GROUP BY clause
    pub fn add_group_by(&mut self, fields: &[&str]) -> &mut Self {
        if !self.has_group_by {
            self.builder.push(" GROUP BY ");
            self.has_group_by = true;
        } else {
            self.builder.push(", ");
        }

        self.builder.push(fields.join(", "));
        self
    }

    /// Add a HAVING clause
    pub fn add_having_clause(&mut self, logical_op: Option<LogicalOperator>) -> &mut Self {
        if !self.has_having {
            self.builder.push(" HAVING ");
            self.has_having = true;
        } else if let Some(op) = logical_op {
            self.builder.push(format!(" {} ", op.as_sql()));
        }
        self
    }

    /// Add a HAVING condition
    pub fn add_having_condition<T: Debug + sqlx::Type<Sqlite> + Send + 'a>(
        &mut self,
        field: &str,
        op: ConditionOperator,
        value: Option<T>,
        logical_op: Option<LogicalOperator>,
    ) -> &mut Self {
        // Skip if value is None, unless the operator is IsNull or IsNotNull
        if value.is_none() && op != ConditionOperator::IsNull && op != ConditionOperator::IsNotNull {
            return self;
        }

        self.add_having_clause(logical_op);

        match op {
            ConditionOperator::IsNull => {
                self.builder.push(format!("{} IS NULL", field));
            }
            ConditionOperator::IsNotNull => {
                self.builder.push(format!("{} IS NOT NULL", field));
            }
            ConditionOperator::In => {
                // For IN operator, value should be a collection
                // This is a simplified implementation
                self.builder.push(format!("{} IN (", field));
                self.builder.push_bind(value.unwrap());
                self.builder.push(")");
            }
            _ => {
                self.builder.push(format!("{} {} ", field, op.as_sql()));
                self.builder.push_bind(value.unwrap());
            }
        }

        self
    }

    /// Add an aggregate function to the query
    pub fn add_aggregate(&mut self, func: AggregateFunction, field: &str, alias: &str) -> &mut Self {
        self.builder.push(format!("{}({}) AS {}", func.as_sql(), field, alias));
        self
    }
}

/// Task query builder that provides specialized methods for building task queries
pub struct TaskQueryBuilder<'a> {
    /// The enhanced query builder
    builder: EnhancedQueryBuilder<'a>,
}

impl<'a> TaskQueryBuilder<'a> {
    /// Create a new task query builder
    pub fn new() -> Self {
        let base_query = r#"SELECT
            id as "id: _", title, description, status as "status: TaskStatus", start_time as "start_time: _",
            end_time as "end_time: _", due_date as "due_date: _", priority as "priority: TaskPriority",
            importance as "importance: TaskImportance", tags as "tags: _", url, metadata as "metadata: _",
            created_at as "created_at: _", updated_at as "updated_at: _", created_by_id as "created_by_id: _",
            assignee_participant_id as "assignee_participant_id: _", workspace_id as "workspace_id: _",
            conversation_id as "conversation_id: _", memory_id as "memory_id: _", plan_id as "plan_id: _",
            document_id as "document_id: _", file_id as "file_id: _"
        FROM tasks"#;

        Self {
            builder: EnhancedQueryBuilder::new(base_query),
        }
    }

    /// Add a filter for workspace ID
    pub fn with_workspace_id(&mut self, workspace_id: Option<uuid::Uuid>) -> &mut Self {
        if let Some(id) = workspace_id {
            self.builder.add_condition("workspace_id", ConditionOperator::Equal, Some(id), Some(LogicalOperator::And));
        }
        self
    }

    /// Add a filter for plan ID
    pub fn with_plan_id(&mut self, plan_id: Option<uuid::Uuid>) -> &mut Self {
        if let Some(id) = plan_id {
            self.builder.add_condition("plan_id", ConditionOperator::Equal, Some(id), Some(LogicalOperator::And));
        }
        self
    }

    /// Add a filter for status
    pub fn with_status(&mut self, status: Option<crate::entities::tasks::TaskStatus>) -> &mut Self {
        if let Some(s) = status {
            self.builder.add_condition("status", ConditionOperator::Equal, Some(s), Some(LogicalOperator::And));
        }
        self
    }

    /// Add a filter for priority
    pub fn with_priority(&mut self, priority: Option<crate::entities::tasks::TaskPriority>) -> &mut Self {
        if let Some(p) = priority {
            self.builder.add_condition("priority", ConditionOperator::Equal, Some(p), Some(LogicalOperator::And));
        }
        self
    }

    /// Add a filter for importance
    pub fn with_importance(&mut self, importance: Option<crate::entities::tasks::TaskImportance>) -> &mut Self {
        if let Some(i) = importance {
            self.builder.add_condition("importance", ConditionOperator::Equal, Some(i), Some(LogicalOperator::And));
        }
        self
    }

    /// Add a filter for active tasks only (not completed or failed)
    pub fn active_only(&mut self, active_only: Option<bool>) -> &mut Self {
        if active_only.unwrap_or(false) {
            self.builder.add_where_clause(Some(LogicalOperator::And));
            self.builder.builder_mut().push("status NOT IN (2, 3)"); // Not Completed or Failed
        }
        self
    }

    /// Add a filter for overdue tasks
    pub fn overdue_only(&mut self, overdue_only: Option<bool>) -> &mut Self {
        if overdue_only.unwrap_or(false) {
            self.builder.add_where_clause(Some(LogicalOperator::And));
            self.builder.builder_mut().push("due_date < datetime('now') AND status NOT IN (2, 3)");
        }
        self
    }

    /// Add a filter for due date range
    pub fn with_due_date_range(
        &mut self,
        after: Option<chrono::DateTime<chrono::Utc>>,
        before: Option<chrono::DateTime<chrono::Utc>>,
    ) -> &mut Self {
        if let Some(after_date) = after {
            self.builder.add_condition("due_date", ConditionOperator::GreaterThanOrEqual, Some(after_date), Some(LogicalOperator::And));
        }
        if let Some(before_date) = before {
            self.builder.add_condition("due_date", ConditionOperator::LessThanOrEqual, Some(before_date), Some(LogicalOperator::And));
        }
        self
    }

    /// Add a search term filter
    pub fn with_search_term(&mut self, search_term: Option<&str>) -> &mut Self {
        if let Some(term) = search_term {
            if !term.is_empty() {
                self.builder.add_where_clause(Some(LogicalOperator::And));
                self.builder.builder_mut().push("(title LIKE ");
                self.builder.builder_mut().push_bind(format!("%{}%", term));
                self.builder.builder_mut().push(" OR description LIKE ");
                self.builder.builder_mut().push_bind(format!("%{}%", term));
                self.builder.builder_mut().push(")");
            }
        }
        self
    }

    /// Add pagination
    pub fn with_pagination(&mut self, limit: Option<usize>, offset: Option<usize>) -> &mut Self {
        if let Some(limit_val) = limit {
            self.builder.add_limit(limit_val as i64);
        }
        if let Some(offset_val) = offset {
            self.builder.add_offset(offset_val as i64);
        }
        self
    }

    /// Add default ordering
    pub fn with_default_ordering(&mut self) -> &mut Self {
        self.builder.add_order_by("due_date", OrderDirection::Asc);
        self.builder.add_order_by("priority", OrderDirection::Desc);
        self.builder.add_order_by("importance", OrderDirection::Desc);
        self.builder.add_order_by("title", OrderDirection::Asc);
        self
    }

    /// Build the query as a Task
    pub fn build_query_as_task(
        &'a self,
    ) -> sqlx::query::QueryAs<'a, Sqlite, crate::entities::tasks::Task, sqlx::sqlite::SqliteArguments<'a>> {
        self.builder.build_query_as()
    }

    /// Get a reference to the enhanced query builder
    pub fn builder(&self) -> &EnhancedQueryBuilder<'a> {
        &self.builder
    }

    /// Get a mutable reference to the enhanced query builder
    pub fn builder_mut(&mut self) -> &mut EnhancedQueryBuilder<'a> {
        &mut self.builder
    }
}
