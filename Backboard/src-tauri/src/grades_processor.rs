use api::models::ImportIndexUsersResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct BackboardGrade {
    #[serde(rename(deserialize = "Tanuló név"))]
    pub student_name: String,
    #[serde(rename(deserialize = "Tanuló osztálya"), skip_serializing)]
    pub school_class: Option<String>,
    #[serde(rename(deserialize = "Születési idő"), skip_serializing)]
    date_of_birth: Option<String>,
    #[serde(rename(deserialize = "Tanuló azonosítója"), skip_serializing)]
    om_code: String,
    #[serde(rename(deserialize = "Tárgy kategória"))]
    subject_category: String,
    #[serde(rename(deserialize = "Tantárgy"))]
    subject: String,
    #[serde(rename(deserialize = "Osztály/Csoport név"))]
    group: String,
    #[serde(rename(deserialize = "Pedagógus név"), default)]
    teacher: Option<String>,
    #[serde(rename(deserialize = "Téma"))]
    theme: String,
    #[serde(rename(deserialize = "Értékelés módja", serialize = "Type"), default)]
    grade_type: Option<String>,
    #[serde(rename(deserialize = "Osztályzat"))]
    text_grade: String,
    #[serde(rename(deserialize = "Jegy"), default)]
    grade: Option<String>,
    #[serde(rename(deserialize = "Szöveges értékelés"))]
    short_text_grade: String,
    #[serde(rename(deserialize = "Százalékos értékelés"))]
    grade_percentage: String,
    #[serde(rename(deserialize = "Magatartás"))]
    behavior_grade: String,
    #[serde(rename(deserialize = "Szorgalom"))]
    diligence_grade: String,
    #[serde(rename(deserialize = "Bejegyzés dátuma"))]
    create_date: String,
    #[serde(rename(deserialize = "Rögzítés dátuma"))]
    record_date: String,
    #[serde(rename(deserialize = "Utolsó mentés dátuma"))]
    last_save_date: String,
}
impl BackboardGrade {
    pub fn hashed_om_code(&self) -> String {
        crate::cryptography::hash(&self.om_code)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackboardStudent {
    #[serde(rename(deserialize = "Név"))]
    pub name: String,
    #[serde(rename(deserialize = "Oktatási azonosítója"))]
    om_code: String,
    #[serde(rename(deserialize = "Osztály"))]
    pub class: String,
}
impl BackboardStudent {
    pub fn hashed_om_code(&self) -> String {
        crate::cryptography::hash(&self.om_code)
    }
}

pub fn process_grades_csv_file(path: String) -> Result<Vec<BackboardGrade>, csv::Error> {
    log::info!("processing grades from {path:?}");
    let mut csv_raw = csv::ReaderBuilder::new().delimiter(b';').from_path(path)?;
    let mut grades = Vec::new();
    for grade in csv_raw.deserialize() {
        grades.push(grade?);
    }
    log::info!("successfully processed grades");
    log::debug!("{grades:?}");

    Ok(grades)
}

pub fn process_students_csv_file(path: String) -> Result<Vec<BackboardStudent>, csv::Error> {
    log::info!("processing students from {path:?}");
    let mut csv_raw = csv::ReaderBuilder::new().delimiter(b';').from_path(path)?;
    let mut students = Vec::new();
    for student in csv_raw.deserialize() {
        students.push(student?);
    }
    log::info!("successfully processed students");
    log::debug!("{students:?}");

    Ok(students)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct BackboardUser {
    id: String,
    public_key: String,
    om_code_hashed: String,
}

impl From<ImportIndexUsersResponse> for BackboardUser {
    fn from(user: ImportIndexUsersResponse) -> Self {
        BackboardUser {
            id: user.id.unwrap().to_string(),
            public_key: user.public_key.unwrap().unwrap(),
            om_code_hashed: user.om_code_hashed.unwrap().unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GradeCollection {
    pub grades: Vec<BackboardGrade>,
    pub school_class: Option<String>,
    pub student_name: String,
    pub user: BackboardUser,
}

#[test]
fn parse_grades() {
    let path = String::from("test_grades.csv");
    assert!(std::fs::exists(&path).unwrap());
    let grades = process_grades_csv_file(path).inspect_err(|err| eprintln!("{err}"));
    assert!(grades.is_ok());
    let _grades = grades.unwrap();
}

#[test]
fn parse_students() {
    let path = String::from("test_students.csv");
    assert!(std::fs::exists(&path).unwrap());
    let students = process_students_csv_file(path).inspect_err(|err| eprintln!("{err}"));
    assert!(students.is_ok());
    let _students = students.unwrap();
}
