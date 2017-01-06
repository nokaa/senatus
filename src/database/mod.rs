mod get;

pub use self::get::*;

use r2d2;
use r2d2_postgres::PostgresConnectionManager;

use super::errors::*;
use super::board::NewBoard;
use super::moderator::Mod;
use super::post::Post;

pub type Pool = r2d2::Pool<PostgresConnectionManager>;

pub fn create_tables(pool: Pool) -> Result<()> {
    let conn = pool.get().unwrap();
    // Create the boards table
    conn.execute("CREATE TABLE IF NOT EXISTS boards (
                    short_name VARCHAR PRIMARY KEY,
                    long_name VARCHAR NOT NULL,
                    description TEXT NOT NULL,
                    post_number BIGINT NOT NULL
                  )",
                 &[])?;

    // Create the admins table
    conn.execute("CREATE TABLE IF NOT EXISTS admins (
                    username VARCHAR PRIMARY KEY,
                    password VARCHAR NOT NULL,
                    boards VARCHAR[] NOT NULL,
                    admin boolean NOT NULL
                  )",
                 &[])?;

    // Create the posts table
    conn.execute("CREATE TABLE IF NOT EXISTS posts (
                    post_number BIGINT NOT NULL,
                    parent BIGINT NOT NULL,
                    board VARCHAR NOT NULL,
                    subject VARCHAR,
                    name VARCHAR NOT NULL,
                    email VARCHAR NOT NULL,
                    content TEXT NOT NULL,
                    thread boolean NOT NULL,
                    pinned boolean NOT NULL,
                    active boolean NOT NULL,
                    last_modified TIMESTAMP,
                    PRIMARY KEY (board, post_number)
                  )",
                 &[])?;

    Ok(())
}

pub fn create_admin(pool: Pool, admin: Mod) -> Result<()> {
    let conn = pool.get().unwrap();

    conn.execute("INSERT INTO admins (username, password, boards, admin) VALUES ($1, $2, $3, $4)",
                 &[&admin.username, &admin.password, &admin.boards, &admin.admin])?;
    Ok(())
}

pub fn create_boards(pool: Pool, boards: &[NewBoard]) -> Result<()> {
    let conn = pool.get().unwrap();

    for b in boards {
        conn.execute("INSERT INTO boards (short_name, long_name, description, post_number) \
                      VALUES ($1, $2, $3, $4) ON CONFLICT (short_name) DO NOTHING",
                     &[&b.short_name, &b.long_name, &b.description, &0i64])?;
    }

    Ok(())
}

pub fn board_exists(pool: Pool, board_name: &str) -> Result<bool> {
    let conn = pool.get().unwrap();

    // TODO(nokaa): Look at the EXISTS keyword for this check:
    // https://stackoverflow.com/questions/7471625/fastest-check-if-row-exists-in-postgresql
    let rows = conn.query("SELECT short_name FROM boards WHERE short_name = $1",
               &[&board_name])?;
    Ok(!rows.is_empty())

}

pub fn thread_exists(pool: Pool, board_name: &str, thread_number: i64) -> Result<bool> {
    let conn = pool.get().unwrap();

    // TODO(nokaa): Look at the EXISTS keyword for this check:
    // https://stackoverflow.com/questions/7471625/fastest-check-if-row-exists-in-postgresql
    let rows =
        conn.query("SELECT post_number, board, thread FROM posts WHERE board = $1 AND \
                    post_number = $2 AND thread = $3",
                   &[&board_name, &thread_number, &true])?;
    Ok(!rows.is_empty())
}

pub fn create_thread(pool: Pool, thread: Post) -> Result<i64> {
    let conn = pool.get().unwrap();
    // let trans = conn.transaction()?;

    // Get the post number for this post
    let rows =
        conn.query("UPDATE boards SET post_number = post_number + 1 WHERE short_name = $1 \
                    RETURNING post_number",
                   &[&thread.board])?;
    let post_number: i64 = rows.get(0).get(0);

    let time = ::chrono::UTC::now().naive_utc();
    conn.execute("INSERT INTO posts (post_number, parent, board, subject, name, email, content, \
                  thread, pinned, active, last_modified) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, \
                  $9, $10);",
                 &[&post_number,
                   &post_number,
                   &thread.board,
                   &thread.subject,
                   &thread.name,
                   &thread.email,
                   &thread.content,
                   &true,
                   &false,
                   &true,
                   &Some(time)])?;
    // trans.commit()?;
    Ok(post_number)
}

pub fn create_post(pool: Pool, post: Post) -> Result<()> {
    let conn = pool.get().unwrap();

    // Get the post number for this post
    let rows =
        conn.query("UPDATE boards SET post_number = post_number + 1 WHERE short_name = $1 \
                    RETURNING post_number",
                   &[&post.board])?;
    let post_number: i64 = rows.get(0).get(0);

    conn.execute("INSERT INTO posts (post_number, parent, board, subject, name, email, content, \
                  thread, pinned, active, last_modified,
                  ) VALUES ($1, $2, $3, \
                  $4, $5, $6, $7, $8, $9, $10, $11);",
                 &[&post_number,
                   &post.parent,
                   &post.board,
                   &post.subject,
                   &post.name,
                   &post.email,
                   &post.content,
                   &false,
                   &false,
                   &false,
                   &None::<::chrono::NaiveDateTime>])?;
    Ok(())
}
