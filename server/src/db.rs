use std::env;

use rocket_db_pools::sqlx::{
    self,
    postgres::{PgPoolOptions, PgRow},
    query, Pool, Postgres, Row,
};

// use r2d2_postgres::{
//     postgres::{NoTls, Row},
//     r2d2::{Pool, PooledConnection},
//     PostgresConnectionManager,
// };

pub async fn get_db_pool() -> Pool<Postgres> {
    let database_url = env::var("DB_URL").unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    sqlx::query(
        r#"
        /* https://gist.github.com/david-sanabria/0d3ff67eb56d2750502aed4186d6a4a7 */
        CREATE EXTENSION IF NOT EXISTS "pgcrypto";
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(r#"CREATE OR REPLACE FUNCTION base36_encode( long_number bigint ) 
        RETURNS text
        AS $BODY$
        /*
        * base36_encode()
        *
        * This function accepts a small or big number (base 10) and reduces its length into a string
        * that is URI-safe using the lower case 26-letter English alphabet 
        * as well as the numbers 0 - 9. The result is returned as a text string.
        *
        */
        declare
            k_base        constant integer := 36;
            k_alphabet    constant text[] := string_to_array( '0123456789abcdefghijklmnopqrstuvwxyz'::text, null);
            
            v_return_text text := '';
            v_remainder   integer;
            v_interim	  bigint;
        begin
        
            v_interim := abs( long_number );  -- Negative Numbers (sign) are ignored
        
            --Conversion Loop
            loop
        
                v_remainder     := v_interim % k_base;
            v_interim       := v_interim / k_base;
            v_return_text   := ''|| k_alphabet[ (v_remainder + 1) ] || v_return_text ;
        
            exit when v_interim <= 0;
        
            end loop ;
        
        
            return v_return_text;
        
        end;$BODY$
        LANGUAGE plpgsql
        immutable		    /* Makes no changes to data in tables */
        returns null ON NULL INPUT  /* Don't bother to call if the value is NULL */
        SECURITY INVOKER            /* No reason to use DEFINER for security */
        cost 5                      /* A made up number. Any advice? */
        ;
    "#).execute(&pool).await.unwrap();
    sqlx::query(
        r#"
        /* completed|cancelled|timed_out|queued|running|failed */

        CREATE TABLE IF NOT EXISTS tanks (
            id          TEXT PRIMARY KEY,
            url         VARCHAR NOT NULL,
            code        VARCHAR NOT NULL,
            log         VARCHAR NOT NULL,
            successful  BOOL NOT NULL,
            language    VARCHAR NOT NULL,
            timestamp   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP(0)
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS simulations (
            id          VARCHAR PRIMARY KEY,
            log         VARCHAR NOT NULL,
            successful  BOOL NOT NULL,
            timestamp   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP(0)
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS runs (
            id          VARCHAR PRIMARY KEY,
            out         VARCHAR NOT NULL,
            err         VARCHAR NOT NULL,
            timestamp   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP(0)
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

// pub fn get_db_pool() -> Pool<PostgresConnectionManager<NoTls>> {
//     let manager =
//         PostgresConnectionManager::new(env::var("DB_URL").unwrap().parse().unwrap(), NoTls);
//     let pool = Pool::new(manager).unwrap();
//     let mut client = pool.get().unwrap();
//     client.batch_execute(r#"
//         /* https://gist.github.com/david-sanabria/0d3ff67eb56d2750502aed4186d6a4a7 */
//         CREATE EXTENSION IF NOT EXISTS "pgcrypto";

//         CREATE OR REPLACE FUNCTION base36_encode( long_number bigint )
//         RETURNS text
//         AS $BODY$
//         /*
//         * base36_encode()
//         *
//         * This function accepts a small or big number (base 10) and reduces its length into a string
//         * that is URI-safe using the lower case 26-letter English alphabet
//         * as well as the numbers 0 - 9. The result is returned as a text string.
//         *
//         */
//         declare
//             k_base        constant integer := 36;
//             k_alphabet    constant text[] := string_to_array( '0123456789abcdefghijklmnopqrstuvwxyz'::text, null);

//             v_return_text text := '';
//             v_remainder   integer;
//             v_interim	  bigint;
//         begin

//             v_interim := abs( long_number );  -- Negative Numbers (sign) are ignored

//             --Conversion Loop
//             loop

//                 v_remainder     := v_interim % k_base;
//             v_interim       := v_interim / k_base;
//             v_return_text   := ''|| k_alphabet[ (v_remainder + 1) ] || v_return_text ;

//             exit when v_interim <= 0;

//             end loop ;

//             return v_return_text;

//         end;$BODY$
//         LANGUAGE plpgsql
//         immutable		    /* Makes no changes to data in tables */
//         returns null ON NULL INPUT  /* Don't bother to call if the value is NULL */
//         SECURITY INVOKER            /* No reason to use DEFINER for security */
//         cost 5                      /* A made up number. Any advice? */
//         ;

//         /* completed|cancelled|timed_out|queued|running|failed */
//         CREATE TABLE IF NOT EXISTS tanks (
//             id          TEXT PRIMARY KEY,
//             url         VARCHAR NOT NULL,
//             code        VARCHAR NOT NULL,
//             log         VARCHAR NOT NULL,
//             successful  BOOL NOT NULL,
//             language    VARCHAR NOT NULL,
//             timestamp   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP(0)
//         );

//         CREATE TABLE IF NOT EXISTS simulations (
//             id          VARCHAR PRIMARY KEY,
//             log         VARCHAR NOT NULL,
//             successful  BOOL NOT NULL,
//             timestamp   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP(0)
//         );

//         CREATE TABLE IF NOT EXISTS runs (
//             id          VARCHAR PRIMARY KEY,
//             out         VARCHAR NOT NULL,
//             err         VARCHAR NOT NULL,
//             timestamp   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP(0)
//         );
//     "#).unwrap();

//     pool
// }

pub async fn insert_tank(pool: &Pool<Postgres>, code: String, post_fix: String, language: String) {
    sqlx::query(
        r#"
            WITH cte AS (
                SELECT ENCODE(DIGEST($1 || $2,'sha256'), 'hex') AS id
            )
            INSERT INTO tanks (id, url, code, log, successful, language)
            SELECT 
                id, 
                SUBSTRING(base36_encode(('x'||lpad(id,16,'0'))::bit(64)::bigint), 0, 8), 
                $1, 
                'waiting to build',
                false,
                $3
            FROM cte;
        "#,
    )
    .bind(&code)
    .bind(&post_fix)
    .bind(&language)
    .execute(pool)
    .await
    .unwrap();
}

pub async fn get_existing(pool: &Pool<Postgres>, code: String, post_fix: String) -> Vec<PgRow> {
    sqlx::query(
        "
                WITH cte AS (
                    SELECT ENCODE(DIGEST($1 || $2,'sha256'), 'hex') AS id
                ), matches AS (
                    SELECT * FROM tanks, cte
                    WHERE tanks.id = cte.id
                )
                SELECT *
                FROM matches;
            ",
    )
    .bind(code)
    .bind(post_fix)
    .fetch_all(pool)
    .await
    .unwrap()
}

pub async fn get_tank_by_url(pool: &Pool<Postgres>, url: &str) -> Vec<PgRow> {
    sqlx::query(
        "
                SELECT * FROM tanks
                WHERE url = $1
            ",
    )
    .bind(&url)
    .fetch_all(pool)
    .await
    .unwrap()
}

pub async fn get_simulation_by_url(pool: &Pool<Postgres>, url: &str) -> Vec<PgRow> {
    sqlx::query(
        "
                SELECT * FROM simulations
                WHERE id = $1
            ",
    )
    .bind(&url)
    .fetch_all(pool)
    .await
    .unwrap()
}

pub async fn upsert_simulation_by_url(pool: &Pool<Postgres>, url: &str) {
    sqlx::query(
        "
                INSERT INTO simulations (id, log, successful)
                VALUES ($1, 'waiting to build', false)
                ON CONFLICT (id) DO NOTHING;
            ",
    )
    .bind(&url)
    .execute(pool)
    .await
    .unwrap();
}

pub async fn get_simulation_log_by_id(pool: &Pool<Postgres>, id: &str) -> Vec<PgRow> {
    sqlx::query(
        "
                SELECT * FROM runs
                WHERE id = $1
            ",
    )
    .bind(&id)
    .fetch_all(pool)
    .await
    .unwrap()
}

pub async fn get_recent_simulations(pool: &Pool<Postgres>) -> Vec<PgRow> {
    sqlx::query(
        "
                SELECT json_agg(to_json(r))::varchar
                FROM (
                    SELECT
                        id,
                        timestamp,
                        SPLIT_PART(log, E'\n', -1)::json as results,
                        SPLIT_PART(log, E'\n', -1)::json->'tanks' as tanks,
                        SPLIT_PART(log, E'\n', -1)::json->'winner' as winner
                    FROM simulations
                    WHERE log != 'waiting to build'
                    ORDER BY timestamp DESC
                    LIMIT 10
                ) r
            ",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}
