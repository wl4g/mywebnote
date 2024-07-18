use anyhow::{ Error, Ok };
use axum::async_trait;

use crate::config::config_api::DbProperties;
use crate::types::users::User;
use crate::types::PageRequest;
use crate::types::PageResponse;
use super::AsyncRepository;
use super::sqlite::SQLiteRepository;

pub struct UserSQLiteRepository {
  inner: SQLiteRepository<User>,
}

impl UserSQLiteRepository {
  pub async fn new(config: &DbProperties) -> Result<Self, Error> {
    Ok(UserSQLiteRepository {
      inner: SQLiteRepository::new(config).await?,
    })
  }
}

#[async_trait]
impl AsyncRepository<User> for UserSQLiteRepository {
  async fn select(
    &self,
    user: User,
    page: PageRequest
  ) -> Result<(PageResponse, Vec<User>), Error> {
    let result = dynamic_sqlite_query!(
      user,
      "users",
      self.inner.get_pool(),
      "update_time",
      page,
      User
    ).unwrap();

    tracing::info!("query users: {:?}", result);
    Ok((result.0, result.1))

    // sqlx
    //   ::query_as::<_, User>("SELECT * FROM users LIMIT $1 OFFSET $2")
    //   .bind(page.get_offset())
    //   .bind(page.get_limit())
    //   .fetch_all(self.inner.get_pool()).await
    //   .map_err(|e| {
    //      tracing::info!("Error to select all: {:?}", e);
    //      Error::msg(e.to_string())
    //   })
  }

  async fn select_by_id(&self, id: i64) -> Result<User, Error> {
    let user = sqlx
      ::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
      .bind(id)
      .fetch_one(self.inner.get_pool()).await
      .unwrap();

    tracing::info!("query user: {:?}", user);
    Ok(user)
  }

  async fn insert(&self, mut user: User) -> Result<i64, Error> {
    let inserted_id = dynamic_sqlite_insert!(user, "users", self.inner.get_pool()).unwrap();
    tracing::info!("Inserted user.id: {:?}", inserted_id);
    Ok(inserted_id)

    // //  let result = sqlx
    // //   ::query(
    // //     r#"
    // //     INSERT INTO users (id, name, email, password, create_by, create_time, update_by, update_time, del_flag)
    // //      VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    // //     "#
    // //   )
    // //   .bind(user.base.id)
    // //   .bind(user.name)
    // //   .bind(user.email)
    // //   .bind(user.phone)
    // //   .bind(user.password) // TODO persistent encrypt password
    // //   .bind(user.base.create_by)
    // //   .bind(user.base.create_time)
    // //   .bind(user.base.update_by)
    // //   .bind(user.base.update_time)
    // //   .bind(user.base.del_flag)
    // //   .execute(self.inner.get_pool()).await
    // //   .unwrap();
    // tracing::info!("Inserted result: {:?}, user.id: {:?}", result, id);

    // Ok(id)
  }

  async fn update(&self, mut user: User) -> Result<i64, Error> {
    let updated_id = dynamic_sqlite_update!(user, "users", self.inner.get_pool()).unwrap();
    tracing::info!("Updated user.id: {:?}", updated_id);
    Ok(updated_id)

    // let id = param.base.id.ok_or_else(|| Error::msg("User id is required for update"))?;
    // let update_result = sqlx
    //   ::query("UPDATE users SET name = $1, email = $2 WHERE id = $3")
    //   .bind(param.name)
    //   .bind(param.email)
    //   .bind(id)
    //   .execute(self.inner.get_pool()).await
    //   .unwrap();
    // tracing::info!("updated result: {:?}", update_result);
    // Ok(update_result.rows_affected() as i64)
  }

  async fn delete_all(&self) -> Result<u64, Error> {
    let delete_result = sqlx
      ::query("DELETE FROM users")
      .execute(self.inner.get_pool()).await
      .unwrap();

    tracing::info!("Deleted result: {:?}", delete_result);
    Ok(delete_result.rows_affected())
  }

  async fn delete_by_id(&self, id: i64) -> Result<u64, Error> {
    let delete_result = sqlx
      ::query("DELETE FROM users WHERE id = $1")
      .bind(id)
      .execute(self.inner.get_pool()).await
      .unwrap();

    tracing::info!("Deleted result: {:?}", delete_result);
    Ok(delete_result.rows_affected())
  }
}
