// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cryptography;
mod grades_processor;

use crate::cryptography::{hash, kyber_encrypt};
use crate::grades_processor::BackboardGrade;
use crate::grades_processor::GradeCollection;
use crate::grades_processor::process_grades_csv_file;
use api::apis::Error;
use api::apis::configuration::{ApiKey, Configuration};
use api::apis::import_api::{
    api_import_grades_user_id_post, api_import_reset_key_password_put, api_import_users_get,
};
use api::apis::status_api::api_status_service_status_get;
use api::models::{
    ImportImportGradesRequestBody, ImportUpdateResetKeyPasswordRequestBody,
    StatusViewServiceStatusResponse,
};
use grades_processor::process_students_csv_file;
use std::collections::HashMap;
use tauri::{Emitter, Window};
use tauri_plugin_autostart::MacosLauncher;

/// extract http error status code if available, otherwise convert it into a string
fn handle_api_err<E: std::fmt::Debug>(e: Error<E>) -> String {
    log::error!("error: {e:#?}");
    match e {
        Error::ResponseError(response_error) => response_error.status.as_str().to_string(),
        other_err => other_err.to_string(),
    }
}
#[tauri::command]
async fn upload_reset_key_password(
    blueboard_url: String,
    reset_key_password: String,
    import_key: String,
) -> Result<(), String> {
    let mut config = Configuration::new();
    config.base_path = blueboard_url;
    config.api_key = Some(ApiKey {
        prefix: None,
        key: import_key,
    });

    log::info!("uploading reset key password");
    api_import_reset_key_password_put(
        &config,
        Some(ImportUpdateResetKeyPasswordRequestBody::new(
            reset_key_password,
        )),
    )
    .await
    .map_err(handle_api_err)
}

#[tauri::command]
async fn import_grades(
    window: Window,
    grades_file_path: String,
    students_file_path: Option<String>,
    blueboard_url: String,
    reset_key_password: String,
    import_key: String,
    update_reset_key_password: bool,
) -> Result<(), String> {
    log::info!("importing grades");
    if update_reset_key_password {
        upload_reset_key_password(
            blueboard_url.clone(),
            reset_key_password,
            import_key.clone(),
        )
        .await?;
        log::info!("succesfully uploaded reset key password");
    }

    let config = Configuration {
        base_path: blueboard_url,
        api_key: Some(ApiKey {
            prefix: None,
            key: import_key,
        }),
        ..Configuration::new()
    };

    let users = api_import_users_get(&config, None, None, None, None)
        .await
        .map_err(handle_api_err)?;
    let num_users = users.len();
    log::info!("users fetched from server already there ({num_users})");
    log::debug!("{users:?}");

    window.emit("import-users", num_users).unwrap();

    let imported_grades =
        process_grades_csv_file(grades_file_path).map_err(|err| err.to_string())?;
    let mut imported_grade_map: HashMap<String, Vec<BackboardGrade>> = HashMap::new();
    for grade in imported_grades {
        imported_grade_map
            .entry(hash(&grade.om_code))
            .or_default()
            .push(grade);
    }
    log::debug!("om-id mapped grades: {imported_grade_map:?}");

    let students_map = if let Some(path) = students_file_path {
        let students = process_students_csv_file(path).map_err(|err| err.to_string())?;
        students
            .into_iter()
            .map(|s| (hash(&s.om_code), s))
            .collect::<HashMap<_, _>>()
    } else {
        HashMap::new()
    };
    log::debug!("hashed om-id mapped students: {students_map:?}");

    let mut count = 0;
    for user in users {
        log::debug!("processing {user:?}");
        let om_code_hashed = &user.om_code_hashed.clone().unwrap().unwrap();
        let Some(user_grades) = imported_grade_map.get(om_code_hashed) else {
            log::warn!("no imported grades found");
            continue;
        };
        log::debug!("freshly imported grades {user_grades:?}");

        let public_key = user.public_key.clone().unwrap().unwrap();
        log::debug!("user's public key: {public_key:?}");

        let (school_class, student_name) =
            if let Some(student_info) = &students_map.get(om_code_hashed) {
                (Some(&student_info.class), &student_info.name)
            } else {
                log::warn!("user not found in student data, falling back to grades");
                let cls = user_grades.iter().find_map(|g| g.school_class.as_ref());
                (cls, &user_grades[0].student_name)
            };
        log::debug!("user's school class: {school_class:?}");

        log::debug!("user's name: {student_name}");

        let grade_collection = GradeCollection {
            grades: user_grades.clone(),
            school_class: school_class.cloned(),
            student_name: student_name.clone(),
            user: user.clone().into(),
        };
        log::trace!("user's grade collection: {grade_collection:?}");

        log::info!("encrypting user's grade collection");
        let grade_collection_encrypted = kyber_encrypt(
            &serde_json::to_string(&grade_collection).unwrap(),
            public_key,
        )
        .map_err(|e| e.to_string())?;
        log::info!("successfully encrypted user's grade collection");

        log::info!("posting user's data");
        api_import_grades_user_id_post(
            &config,
            &user.id.unwrap().to_string(),
            Some(ImportImportGradesRequestBody {
                json_encrypted: grade_collection_encrypted,
            }),
        )
        .await
        .map_err(handle_api_err)?;
        log::info!("successfully posted user's data");

        count += 1;
        window
            .emit("import-progress", (count / num_users) * 100)
            .unwrap();
    }

    Ok(())
}

#[tauri::command]
async fn status(blueboard_url: String) -> Result<StatusViewServiceStatusResponse, String> {
    let mut config = Configuration::new();
    config.base_path = blueboard_url;
    log::info!("requesting service status");

    let res = api_status_service_status_get(&config)
        .await
        .map_err(handle_api_err);
    log::debug!("service status: {res:?}");
    res
}

fn main() {
    let log_p = std::path::Path::new(".lovassyapp-backboard.log");
    ftail::Ftail::new()
        .console(log::LevelFilter::Info)
        .single_file(&log_p, true, log::LevelFilter::Debug)
        .init()
        .unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            status,
            upload_reset_key_password,
            import_grades
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
