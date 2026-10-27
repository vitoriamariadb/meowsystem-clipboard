use std::time::Duration;

use anyhow::Result;
use sqlx::{SqliteConnection, prelude::*};

pub const LINHAS_MANTIDAS: i64 = 1000;
pub const ORCAMENTO_IMAGEM_BYTES: i64 = 1024 * 1048576;
pub const INTERVALO: Duration = Duration::from_secs(3600);
const PAGINAS_LIVRES_PARA_VACUUM: i64 = 2000;

async fn contar(conn: &mut SqliteConnection, sql: &'static str) -> Result<i64> {
    Ok(sqlx::query(sql).fetch_one(conn).await?.get::<i64, _>(0))
}

pub async fn podar(conn: &mut SqliteConnection, linhas: i64, orcamento: i64) -> Result<bool> {
    let entradas = contar(conn, "SELECT COUNT(*) FROM ClipboardEntries").await?;
    let conteudos = contar(conn, "SELECT COUNT(*) FROM ClipboardContents").await?;

    sqlx::query(
        r#"
        DELETE FROM ClipboardEntries
         WHERE id NOT IN (SELECT id FROM ClipboardEntries ORDER BY id DESC LIMIT ?)
           AND id NOT IN (SELECT id FROM FavoriteClipboardEntries)
        "#,
    )
    .bind(linhas)
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM ClipboardEntries WHERE id IN (
          SELECT id FROM (
            SELECT p.id AS id,
                   SUM(p.peso) OVER (ORDER BY p.id DESC
                                     ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS acumulado,
                   p.peso AS peso
              FROM (SELECT id, SUM(LENGTH(content)) AS peso
                      FROM ClipboardContents
                     WHERE mime LIKE 'image/%' OR mime = 'application/x-qt-image'
                     GROUP BY id) p
          ) WHERE acumulado - peso > ?
        ) AND id NOT IN (SELECT id FROM FavoriteClipboardEntries)
        "#,
    )
    .bind(orcamento)
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "DELETE FROM ClipboardEntries WHERE id NOT IN (SELECT DISTINCT id FROM ClipboardContents)",
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query("DELETE FROM ClipboardContents WHERE id NOT IN (SELECT id FROM ClipboardEntries)")
        .execute(&mut *conn)
        .await?;

    let apagou = contar(conn, "SELECT COUNT(*) FROM ClipboardEntries").await? != entradas
        || contar(conn, "SELECT COUNT(*) FROM ClipboardContents").await? != conteudos;
    let livres = contar(conn, "PRAGMA freelist_count").await?;

    if apagou || livres > PAGINAS_LIVRES_PARA_VACUUM {
        sqlx::query("VACUUM").execute(&mut *conn).await?;
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&mut *conn)
            .await?;
    }

    Ok(apagou)
}

#[cfg(test)]
mod tests {
    use sqlx::{Connection, SqliteConnection};

    use super::*;

    async fn banco() -> SqliteConnection {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        sqlx::raw_sql(include_str!("../../migrations/20240723123147_init.sql"))
            .execute(&mut conn)
            .await
            .unwrap();
        conn
    }

    async fn entrada(conn: &mut SqliteConnection, id: i64, mime: &str, bytes: usize) {
        sqlx::query("INSERT INTO ClipboardEntries (id, creation) VALUES (?, ?)")
            .bind(id)
            .bind(id)
            .execute(&mut *conn)
            .await
            .unwrap();
        sqlx::query("INSERT INTO ClipboardContents (id, mime, content) VALUES (?, ?, ?)")
            .bind(id)
            .bind(mime)
            .bind(vec![7u8; bytes])
            .execute(&mut *conn)
            .await
            .unwrap();
    }

    async fn ids(conn: &mut SqliteConnection) -> Vec<i64> {
        sqlx::query("SELECT id FROM ClipboardEntries ORDER BY id")
            .fetch_all(conn)
            .await
            .unwrap()
            .iter()
            .map(|r| r.get::<i64, _>(0))
            .collect()
    }

    #[tokio::test]
    async fn mantem_so_as_mais_novas_e_poupa_favoritos() {
        let mut conn = banco().await;
        for id in 1..=10 {
            entrada(&mut conn, id, "text/plain", 3).await;
        }
        sqlx::query("INSERT INTO FavoriteClipboardEntries (id, position) VALUES (2, 0)")
            .execute(&mut conn)
            .await
            .unwrap();

        assert!(podar(&mut conn, 4, i64::MAX).await.unwrap());

        assert_eq!(ids(&mut conn).await, vec![2, 7, 8, 9, 10]);
        let favoritos: i64 = contar(&mut conn, "SELECT COUNT(*) FROM FavoriteClipboardEntries")
            .await
            .unwrap();
        assert_eq!(favoritos, 1);
        let conteudos = contar(&mut conn, "SELECT COUNT(*) FROM ClipboardContents")
            .await
            .unwrap();
        assert_eq!(conteudos, 5);
    }

    #[tokio::test]
    async fn orcamento_de_imagem_corta_da_mais_velha() {
        let mut conn = banco().await;
        for id in 1..=4 {
            entrada(&mut conn, id, "image/png", 400).await;
        }
        entrada(&mut conn, 5, "text/plain", 5000).await;

        assert!(podar(&mut conn, 1000, 1000).await.unwrap());

        assert_eq!(ids(&mut conn).await, vec![2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn imagem_nova_maior_que_o_orcamento_fica() {
        let mut conn = banco().await;
        entrada(&mut conn, 1, "image/png", 100).await;
        entrada(&mut conn, 2, "image/png", 5000).await;

        assert!(podar(&mut conn, 1000, 1000).await.unwrap());

        assert_eq!(ids(&mut conn).await, vec![2]);
    }

    #[tokio::test]
    async fn orfaos_saem_e_sem_nada_a_apagar_devolve_falso() {
        let mut conn = banco().await;
        entrada(&mut conn, 1, "text/plain", 3).await;
        sqlx::query("INSERT INTO ClipboardEntries (id, creation) VALUES (2, 2)")
            .execute(&mut conn)
            .await
            .unwrap();

        assert!(podar(&mut conn, 1000, i64::MAX).await.unwrap());
        assert_eq!(ids(&mut conn).await, vec![1]);

        assert!(!podar(&mut conn, 1000, i64::MAX).await.unwrap());
        assert_eq!(ids(&mut conn).await, vec![1]);
    }
}
