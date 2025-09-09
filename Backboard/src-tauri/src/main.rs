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
use tauri::Emitter;
use tauri::Window;
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
    if update_reset_key_password {
        upload_reset_key_password(
            blueboard_url.clone(),
            reset_key_password,
            import_key.clone(),
        )
        .await?;
    }

    let config = Configuration {
        base_path: blueboard_url,
        api_key: Some(ApiKey {
            prefix: None,
            key: import_key,
        }),
        ..Configuration::new()
    };

    let users = api_import_users_get(&config.clone(), None, None, None, None)
        .await
        .map_err(handle_api_err)?;

    window.emit("import-users", &users.len()).unwrap();

    let grades = process_grades_csv_file(grades_file_path).map_err(|err| err.to_string())?;

    let students = if let Some(path) = students_file_path {
        Some(process_students_csv_file(path).map_err(|err| err.to_string())?)
    } else {
        None
    };

    let mut grade_map: HashMap<String, Vec<BackboardGrade>> = HashMap::new();

    for grade in grades {
        grade_map
            .entry(hash(grade.clone().om_code))
            .or_default()
            .push(grade);
    }

    let mut students_map = if students.is_some() {
        Some(HashMap::new())
    } else {
        None
    };

    if let (Some(students), Some(students_map)) = (students, &mut students_map) {
        for student in students {
            students_map.insert(hash(student.clone().om_code), student);
        }
    }

    let mut count = 0;
    for user in &users {
        let om_code_hashed = &user.clone().om_code_hashed.unwrap().unwrap();
        let Some(user_grades) = grade_map.get(om_code_hashed) else {
            continue;
        };

        let public_key = user.public_key.clone().unwrap().unwrap();

        let school_class = if let Some(students_map) = &mut students_map {
            Some(students_map.get(om_code_hashed).unwrap().class.clone())
        } else {
            user_grades
                .iter()
                .find_map(|grade| grade.school_class.clone())
        };
        let student_name = if let Some(students_map) = &mut students_map {
            students_map.get(om_code_hashed).unwrap().name.clone()
        } else {
            user_grades[0].student_name.clone()
        };

        let grade_collection = GradeCollection {
            grades: user_grades.clone(),
            school_class,
            student_name,
            user: user.clone().into(),
        };

        let grade_collection_encrypted = kyber_encrypt(
            serde_json::to_string(&grade_collection).unwrap(),
            public_key,
        )
        .map_err(|e| e.to_string())?;

        api_import_grades_user_id_post(
            &config,
            &user.id.unwrap().to_string(),
            Some(ImportImportGradesRequestBody {
                json_encrypted: grade_collection_encrypted,
            }),
        )
        .await
        .map_err(handle_api_err)?;

        count += 1;
        window
            .emit("import-progress", (count / users.len()) * 100)
            .unwrap();
    }

    Ok(())
}

#[tauri::command]
async fn status(blueboard_url: String) -> Result<StatusViewServiceStatusResponse, String> {
    let mut config = Configuration::new();
    config.base_path = blueboard_url;

    api_status_service_status_get(&config)
        .await
        .map_err(handle_api_err)
}

fn main() {
    env_logger::init();
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
