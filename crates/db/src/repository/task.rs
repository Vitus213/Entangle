use crate::models::task::{
    CreateTask, Task, TaskListItem, TaskResponse, TaskUser, UpdateTask, UpdateTaskStatus,
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct TaskRepository;

// Helper struct for joined task query (max 16 fields for sqlx)
#[derive(sqlx::FromRow)]
struct TaskWithCreator {
    id: Uuid,
    doc_id: Option<Uuid>,
    title: String,
    description: Option<String>,
    assignee_id: Option<Uuid>,
    created_by: Uuid,
    status: String,
    priority: String,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    doc_title: Option<String>,
    creator_nickname: String,
    creator_avatar_url: Option<String>,
}

impl TaskRepository {
    /// 创建任务
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        input: CreateTask,
    ) -> Result<Task, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (id, doc_id, title, description, assignee_id, created_by, priority, due_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(input.doc_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(input.assignee_id)
        .bind(user_id)
        .bind(input.priority.to_string())
        .bind(input.due_date)
        .fetch_one(pool)
        .await
    }

    /// 根据 ID 获取任务
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// 获取任务详情（包含用户信息）
    pub async fn find_with_details(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<TaskResponse>, sqlx::Error> {
        // First get the task with creator info
        let task = sqlx::query_as::<_, TaskWithCreator>(
            r#"
            SELECT
                t.id, t.doc_id, t.title, t.description, t.assignee_id, t.created_by,
                t.status, t.priority, t.due_date, t.completed_at, t.created_at, t.updated_at,
                d.title as doc_title,
                creator.nickname as creator_nickname,
                creator.avatar_url as creator_avatar_url
            FROM tasks t
            LEFT JOIN documents d ON t.doc_id = d.id
            JOIN users creator ON t.created_by = creator.id
            WHERE t.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        match task {
            Some(t) => {
                // Fetch assignee info separately if exists
                let assignee = if let Some(assignee_id) = t.assignee_id {
                    sqlx::query_as::<_, (Uuid, String, Option<String>)>(
                        "SELECT id, nickname, avatar_url FROM users WHERE id = $1"
                    )
                    .bind(assignee_id)
                    .fetch_optional(pool)
                    .await?
                    .map(|a| TaskUser {
                        id: a.0,
                        nickname: a.1,
                        avatar_url: a.2,
                    })
                } else {
                    None
                };

                Ok(Some(TaskResponse {
                    id: t.id,
                    doc_id: t.doc_id,
                    doc_title: t.doc_title,
                    title: t.title,
                    description: t.description,
                    status: t.status,
                    priority: t.priority,
                    due_date: t.due_date,
                    completed_at: t.completed_at,
                    created_at: t.created_at,
                    updated_at: t.updated_at,
                    created_by: TaskUser {
                        id: t.created_by,
                        nickname: t.creator_nickname,
                        avatar_url: t.creator_avatar_url,
                    },
                    assignee,
                }))
            }
            None => Ok(None),
        }
    }

    /// 获取用户创建的任务
    pub async fn find_created_by(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TaskListItem>, sqlx::Error> {
        Self::find_tasks_list(pool, "t.created_by = $1", user_id, limit, offset).await
    }

    /// 获取分配给用户的任务
    pub async fn find_assigned_to(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TaskListItem>, sqlx::Error> {
        Self::find_tasks_list(pool, "t.assignee_id = $1", user_id, limit, offset).await
    }

    /// 获取用户相关的所有任务（创建的或分配给的）
    pub async fn find_for_user(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TaskListItem>, sqlx::Error> {
        Self::find_tasks_list(pool, "(t.created_by = $1 OR t.assignee_id = $1)", user_id, limit, offset).await
    }

    /// 获取文档相关的任务
    pub async fn find_by_document(
        pool: &PgPool,
        doc_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TaskListItem>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (
            Uuid, Option<Uuid>, String, String, String,
            Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>,
            Option<String>,  // doc_title
            Option<Uuid>, Option<String>, Option<String>,  // assignee
        )>(
            r#"
            SELECT
                t.id, t.doc_id, t.title, t.status, t.priority, t.due_date, t.created_at,
                d.title as doc_title,
                assignee.id as assignee_id,
                assignee.nickname as assignee_nickname,
                assignee.avatar_url as assignee_avatar_url
            FROM tasks t
            LEFT JOIN documents d ON t.doc_id = d.id
            LEFT JOIN users assignee ON t.assignee_id = assignee.id
            WHERE t.doc_id = $1
            ORDER BY t.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(doc_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(Self::map_task_list_items(rows))
    }

    /// 内部查询方法
    async fn find_tasks_list(
        pool: &PgPool,
        where_clause: &str,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TaskListItem>, sqlx::Error> {
        let query = format!(
            r#"
            SELECT
                t.id, t.doc_id, t.title, t.status, t.priority, t.due_date, t.created_at,
                d.title as doc_title,
                assignee.id as assignee_id,
                assignee.nickname as assignee_nickname,
                assignee.avatar_url as assignee_avatar_url
            FROM tasks t
            LEFT JOIN documents d ON t.doc_id = d.id
            LEFT JOIN users assignee ON t.assignee_id = assignee.id
            WHERE {}
            ORDER BY t.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            where_clause
        );

        let rows = sqlx::query_as::<_, (
            Uuid, Option<Uuid>, String, String, String,
            Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>,
            Option<String>,
            Option<Uuid>, Option<String>, Option<String>,
        )>(&query)
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(Self::map_task_list_items(rows))
    }

    fn map_task_list_items(rows: Vec<(
        Uuid, Option<Uuid>, String, String, String,
        Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>,
        Option<String>,
        Option<Uuid>, Option<String>, Option<String>,
    )>) -> Vec<TaskListItem> {
        rows.into_iter()
            .map(|r| TaskListItem {
                id: r.0,
                doc_id: r.1,
                doc_title: r.7,
                title: r.2,
                status: r.3,
                priority: r.4,
                due_date: r.5,
                created_at: r.6,
                assignee: r.8.map(|id| TaskUser {
                    id,
                    nickname: r.9.clone().unwrap_or_default(),
                    avatar_url: r.10.clone(),
                }),
            })
            .collect()
    }

    /// 更新任务
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        input: UpdateTask,
    ) -> Result<Task, sqlx::Error> {
        let task = Self::find_by_id(pool, id).await?.ok_or(sqlx::Error::RowNotFound)?;

        let title = input.title.unwrap_or(task.title);
        let description = input.description.or(task.description);
        let assignee_id = input.assignee_id.or(task.assignee_id);
        let priority = input.priority.map(|p| p.to_string()).unwrap_or(task.priority);
        let due_date = input.due_date.or(task.due_date);

        sqlx::query_as::<_, Task>(
            r#"
            UPDATE tasks
            SET title = $1, description = $2, assignee_id = $3, priority = $4, due_date = $5, updated_at = NOW()
            WHERE id = $6
            RETURNING *
            "#,
        )
        .bind(&title)
        .bind(&description)
        .bind(assignee_id)
        .bind(&priority)
        .bind(due_date)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// 更新任务状态
    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        input: UpdateTaskStatus,
    ) -> Result<Task, sqlx::Error> {
        let status_str = input.status.to_string();
        let completed_at = if status_str == "completed" {
            Some(chrono::Utc::now())
        } else {
            None
        };

        sqlx::query_as::<_, Task>(
            r#"
            UPDATE tasks
            SET status = $1, completed_at = COALESCE($2, completed_at), updated_at = NOW()
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(&status_str)
        .bind(completed_at)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// 分配任务
    pub async fn assign(
        pool: &PgPool,
        id: Uuid,
        assignee_id: Option<Uuid>,
    ) -> Result<Task, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            r#"
            UPDATE tasks
            SET assignee_id = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(assignee_id)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// 删除任务
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM tasks WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 检查用户是否是任务的创建者
    pub async fn is_creator(pool: &PgPool, task_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM tasks WHERE id = $1 AND created_by = $2",
        )
        .bind(task_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(result > 0)
    }

    /// 检查用户是否可以查看任务（创建者或被分配者）
    pub async fn can_view(pool: &PgPool, task_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM tasks WHERE id = $1 AND (created_by = $2 OR assignee_id = $2)",
        )
        .bind(task_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(result > 0)
    }

    /// 获取用户待办任务数量
    pub async fn count_pending_for_user(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM tasks
            WHERE (created_by = $1 OR assignee_id = $1)
            AND status IN ('pending', 'in_progress')
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
    }
}
