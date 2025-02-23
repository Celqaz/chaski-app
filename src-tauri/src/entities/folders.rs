use crate::db::establish_connection;
use crate::schema::articles;
use crate::schema::feeds::dsl::*;
use crate::schema::filters;
use diesel::prelude::*;

pub fn rename(current_name: String, new_name: String, app_handle: tauri::AppHandle) {
    let conn = &mut establish_connection(&app_handle);

    let result = diesel::update(feeds)
        .filter(folder.eq(current_name))
        .set(folder.eq(new_name))
        .execute(conn);

    match result {
        Ok(count) => {
            log::info!(target: "chaski:folders","Folder renamed. Total feed updated: {count:?}");
        }
        Err(err) => {
            log::error!(target: "chaski:folders","Renaming folder: {err:?}");
        }
    }
}

pub fn delete(folder_name: String, app_handle: tauri::AppHandle) -> bool {
    let conn = &mut establish_connection(&app_handle);

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        let _ = diesel::delete(
            filters::table.filter(
                filters::feed_id.eq_any(
                    feeds
                        .filter(folder.eq(&folder_name))
                        .select(id)
                        .load::<i32>(conn)?,
                ),
            ),
        )
        .execute(conn)?;

        let _ = diesel::delete(
            articles::table.filter(
                articles::feed_id.eq_any(
                    feeds
                        .filter(folder.eq(&folder_name))
                        .select(id)
                        .load::<i32>(conn)?,
                ),
            ),
        )
        .execute(conn)?;

        let _ = diesel::delete(feeds.filter(folder.eq(folder_name))).execute(conn)?;

        Ok(())
    })
    .unwrap_or_else(|err| {
        log::error!(target: "chaski:folders", "Error deleting folder and its data: {err:?}");
    });

    log::info!(target: "chaski:folders", "Folder and its related data deleted successfully");
    true
}
