use gloo_file::{futures::read_as_data_url, File};
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::{window, EventSource, HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

const API_BASE: &str = "http://localhost:8080";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Branch {
    id: String,
    name: String,
    municipality: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Employee {
    id: String,
    branch_id: String,
    name: String,
    surname: String,
    job_position: String,
    official_email: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
    csrf_token: String,
    role: String,
    user_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LeaveRecord {
    id: String,
    employee_id: String,
    leave_type: String,
    start_date: String,
    end_date: String,
    status: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct PayrollResult {
    employee_id: String,
    month_label: String,
    gross_total: f64,
    net_total: f64,
    currency: String,
    edi_line: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Department {
    id: String,
    branch_id: String,
    name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct JobPosition {
    id: String,
    name: String,
    description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ContractRecord {
    id: String,
    employee_id: String,
    contract_type: String,
    start_date: String,
    end_date: Option<String>,
    base_salary_eur: f64,
    coefficient: f64,
    status: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct SalaryElementRecord {
    id: String,
    employee_id: String,
    element_name: String,
    amount: f64,
    period_label: String,
}

#[derive(Clone, Debug, PartialEq)]
enum View {
    Home,
    Dashboard,
    EmployeeRegister,
    Vacation,
    Payroll,
    Management,
}

const KOSOVO_MUNICIPALITIES: &[&str] = &[
    "Prishtinë", "Prizren", "Ferizaj", "Pejë", "Gjakovë", "Gjilan",
    "Mitrovicë e Jugut", "Vushtrri", "Suharekë", "Rahovec", "Drenas",
    "Lipjan", "Podujevë", "Viti", "Kamenicë", "Istog", "Klinë",
    "Skenderaj", "Malishevë", "Deçan", "Dragash", "Kaçanik", "Shtime",
    "Obiliq", "Fushë Kosovë", "Novo Bërdë", "Ranillug", "Partesh",
    "Kllokot", "Graçanicë", "Mitrovica e Veriut", "Zubin Potok",
    "Zveçan", "Leposaviq", "Junik", "Mamushë", "Hani i Elezit",
];

fn menu_target(label: &str) -> Option<View> {
    match label {
        "Regjistro Punonjës" => Some(View::EmployeeRegister),
        "Kërkesë Pushimi" | "Orët e Pushimit" | "Statusi i Festave" | "Kalendari i Festave" => Some(View::Vacation),
        "Llogaritja e Pagës" | "Lista e Pagave" | "E-Deklarata" => Some(View::Payroll),
        "Regjistro Kontrata" | "Elementet e Pagës" | "Degët" | "Departamentet/Njësitë" | "Pozitat e Punës" | "Statusi i Punonjësit" => {
            Some(View::Management)
        }
        _ => None,
    }
}

fn menu_title_target(title: &str) -> Option<View> {
    match title {
        "Punonjësi" => Some(View::EmployeeRegister),
        "Paga/Kompensimi" => Some(View::Payroll),
        "Pushimi" => Some(View::Vacation),
        "Definicionet HR" | "Kompania" | "Administrata" => Some(View::Management),
        _ => None,
    }
}

fn menu_items() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("Punonjësi", vec!["Regjistro Punonjës", "Hyrje-Dalje", "Regjistro Kontrata", "Dosjet e Punonjësve", "Statusi i Punonjësit"]),
        ("Paga/Kompensimi", vec!["Përcaktimi i Pagës", "Periudha e Pagës", "Ditë/Orë Shtesë", "Të Ardhura Shtesë", "Llogaritja e Pagës", "Lista e Pagave", "E-Deklarata"]),
        ("Pushimi", vec!["Kërkesë Pushimi", "Orët e Pushimit", "Statusi i Festave", "Kalendari i Festave"]),
        ("Hyrje/Dalje", vec!["Regjistro Hyrje", "Hyrje të Hapura", "Lista e Hyrjeve", "Punonjës të Pranishëm"]),
        ("Definicionet HR", vec!["Statusi i Punonjësit", "Llojet e Kontratave", "Lloji i Punëdhënësit", "Llojet e Pushimit", "Llojet e Provës", "Llogaritja e Elementeve", "Koeficienti", "Elementet e Pagës"]),
        ("Kompania", vec!["Detajet e Kompanisë", "Degët", "Departamentet/Njësitë", "Pozitat e Punës"]),
        ("Administrata", vec!["Regjistrimi Komunal", "Regjistrimi Shtetëror", "Regjistrimi Bankar", "Statusi Martesor"]),
    ]
}

#[function_component(App)]
pub fn app() -> Html {
    let branches = use_state(Vec::<Branch>::new);
    let selected_branch = use_state(|| None::<String>);
    let employees = use_state(Vec::<Employee>::new);
    let selected_employee = use_state(|| None::<String>);
    let attendance_events = use_state(Vec::<String>::new);
    let leave_records = use_state(Vec::<LeaveRecord>::new);
    let payroll_result = use_state(|| None::<PayrollResult>);
    let departments = use_state(Vec::<Department>::new);
    let job_positions = use_state(Vec::<JobPosition>::new);
    let contracts = use_state(Vec::<ContractRecord>::new);
    let salary_elements = use_state(Vec::<SalaryElementRecord>::new);
    let camera_photo = use_state(|| None::<String>);
    let camera_input_ref = use_node_ref();

    let access_token = use_state(|| LocalStorage::get::<String>("access_token").ok());
    let csrf_token = use_state(|| LocalStorage::get::<String>("csrf_token").ok());
    let user_role = use_state(|| LocalStorage::get::<String>("role").unwrap_or_else(|_| "guest".to_owned()));
    let view = use_state(|| View::Home);
    let show_login_popup = use_state(|| false);
    let error_msg = use_state(String::new);

    let login_email = use_state(|| "system_admin@example.com".to_owned());
    let login_password = use_state(|| "Password123!".to_owned());
    let department_name = use_state(|| "Operations".to_owned());
    let job_name = use_state(|| "Logistics Coordinator".to_owned());
    let salary_element_name = use_state(|| "Transport allowance".to_owned());
    let salary_element_amount = use_state(|| "35.0".to_owned());
    let contract_type_inputs = use_state(HashMap::<String, String>::new);
    let contract_salary_inputs = use_state(HashMap::<String, String>::new);
    let contract_status_inputs = use_state(HashMap::<String, String>::new);
    let salary_name_inputs = use_state(HashMap::<String, String>::new);
    let salary_amount_inputs = use_state(HashMap::<String, String>::new);
    let salary_period_inputs = use_state(HashMap::<String, String>::new);
    let employee_positions = use_state(|| vec![
        "Menaxher".to_owned(),
        "Asistent menaxher".to_owned(),
        "Depoist Depo".to_owned(),
        "Keshilltarë per Klient".to_owned(),
        "Arkitekt".to_owned(),
        "Ndihmese".to_owned(),
        "Shtepiak".to_owned(),
        "Menaxher i Depos".to_owned(),
        "Kordinator I Shitjes".to_owned(),
        "Menaxher Importi".to_owned(),
        "Vozites".to_owned(),
        "Inxhinier i Hidros".to_owned(),
    ]);
    let new_position_input = use_state(String::new);
    let reg_pozita = use_state(|| "Menaxher".to_owned());
    let reg_country = use_state(|| "Kosova".to_owned());
    let reg_branch_id = use_state(String::new);
    let reg_municipality = use_state(|| "Prishtinë".to_owned());

    {
        let branches = branches.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(resp) = Request::get(&format!("{}/api/company/branches", API_BASE)).send().await {
                    if let Ok(items) = resp.json::<Vec<Branch>>().await {
                        branches.set(items);
                    }
                }
            });
            || ()
        });
    }

    {
        let access_token = access_token.clone();
        let attendance_events = attendance_events.clone();
        use_effect_with((*access_token).clone(), move |token| {
            if let Some(t) = token {
                if let Ok(es) = EventSource::new(&format!("{}/api/attendance/feed", API_BASE)) {
                    let onmessage = wasm_bindgen::closure::Closure::<dyn FnMut(_)>::new({
                        let attendance_events = attendance_events.clone();
                        move |e: web_sys::MessageEvent| {
                            if let Some(text) = e.data().as_string() {
                                let mut next = (*attendance_events).clone();
                                next.insert(0, text);
                                if next.len() > 40 {
                                    next.truncate(40);
                                }
                                attendance_events.set(next);
                            }
                        }
                    });
                    es.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                    onmessage.forget();
                    let _ = t;
                    std::mem::forget(es);
                }
            }
            || ()
        });
    }

    let load_employees = {
        let employees = employees.clone();
        let selected_branch = selected_branch.clone();
        let access_token = access_token.clone();
        let error_msg = error_msg.clone();
        Callback::from(move |_| {
            let employees = employees.clone();
            let selected = (*selected_branch).clone();
            let token = (*access_token).clone();
            let error_msg = error_msg.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let mut req = Request::get(&format!(
                    "{}/api/employees{}",
                    API_BASE,
                    selected
                        .as_ref()
                        .map(|id| format!("?branch_id={}", id))
                        .unwrap_or_default()
                ));

                let _ = token;

                match req.send().await {
                    Ok(resp) if resp.ok() => match resp.json::<Vec<Employee>>().await {
                        Ok(items) => employees.set(items),
                        Err(_) => error_msg.set("Could not parse employees".to_owned()),
                    },
                    _ => error_msg.set("Failed to load employees. Login as manager/hr/system admin.".to_owned()),
                }
            });
        })
    };

    let on_login = {
        let login_email = login_email.clone();
        let login_password = login_password.clone();
        let access_token = access_token.clone();
        let csrf_token = csrf_token.clone();
        let user_role = user_role.clone();
        let view = view.clone();
        let show_login_popup = show_login_popup.clone();
        let error_msg = error_msg.clone();
        Callback::from(move |_| {
            let email = (*login_email).clone();
            let password = (*login_password).clone();
            let access_token = access_token.clone();
            let csrf_token = csrf_token.clone();
            let user_role = user_role.clone();
            let view = view.clone();
            let show_login_popup = show_login_popup.clone();
            let error_msg = error_msg.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let body = serde_json::json!({ "email": email, "password": password });
                let req = Request::post(&format!("{}/api/auth/login", API_BASE))
                    .body(body.to_string())
                    .expect("valid request");

                let _ = req.headers().set("Content-Type", "application/json");

                match req.send().await {
                    Ok(resp) if resp.ok() => {
                        if let Ok(auth) = resp.json::<LoginResponse>().await {
                            let _ = LocalStorage::set("access_token", auth.access_token.clone());
                            let _ = LocalStorage::set("csrf_token", auth.csrf_token.clone());
                            let _ = LocalStorage::set("role", auth.role.clone());
                            access_token.set(Some(auth.access_token));
                            csrf_token.set(Some(auth.csrf_token));
                            user_role.set(auth.role);
                            show_login_popup.set(false);
                            view.set(View::Dashboard);
                            error_msg.set(String::new());
                        }
                    }
                    _ => error_msg.set("Login failed".to_owned()),
                }
            });
        })
    };

    let quick_punch = {
        let access_token = access_token.clone();
        let csrf_token = csrf_token.clone();
        let employees = employees.clone();
        let selected_employee = selected_employee.clone();
        let camera_photo = camera_photo.clone();
        let error_msg = error_msg.clone();
        Callback::from(move |click_type: String| {
            let token = (*access_token).clone();
            let csrf = (*csrf_token).clone();
            let employees = (*employees).clone();
            let selected_employee = (*selected_employee).clone();
            let photo = (*camera_photo).clone();
            let error_msg = error_msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if employees.is_empty() {
                    error_msg.set("Select branch and load employees first".to_owned());
                    return;
                }
                let Some(photo) = photo else {
                    error_msg.set("Capture a camera photo before clock-in/out".to_owned());
                    return;
                };
                let employee_id = selected_employee
                    .and_then(|id| employees.iter().find(|e| e.id == id).map(|e| e.id.clone()))
                    .unwrap_or_else(|| employees[0].id.clone());
                let body = serde_json::json!({
                    "employee_id": employee_id,
                    "click_type": click_type,
                    "camera_photo_base64": photo,
                    "note": "Web quick punch"
                });

                let mut req = Request::post(&format!("{}/api/attendance/punch", API_BASE))
                    .body(body.to_string())
                    .expect("valid request");

                let _ = req.headers().set("Content-Type", "application/json");

                if let Some(tk) = token {
                    let _ = req
                        .headers()
                        .set("Authorization", &format!("Bearer {}", tk));
                }
                if let Some(cs) = csrf {
                    let _ = req.headers().set("x-csrf-token", &cs);
                }

                if req.send().await.is_err() {
                    error_msg.set("Clock-in/out failed".to_owned());
                }
            });
        })
    };

    let capture_photo = {
        let camera_input_ref = camera_input_ref.clone();
        Callback::from(move |_| {
            if let Some(input) = camera_input_ref.cast::<HtmlInputElement>() {
                input.set_value("");
                let _ = input.click();
            }
        })
    };

    let on_camera_selected = {
        let camera_photo = camera_photo.clone();
        let error_msg = error_msg.clone();
        Callback::from(move |event: Event| {
            let input: HtmlInputElement = event.target_unchecked_into();
            let Some(files) = input.files() else {
                return;
            };
            if files.length() == 0 {
                return;
            }
            let Some(raw_file) = files.get(0) else {
                return;
            };

            let camera_photo = camera_photo.clone();
            let error_msg = error_msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match read_as_data_url(&File::from(raw_file)).await {
                    Ok(data_url) => {
                        let encoded = data_url
                            .split_once(',')
                            .map(|(_, b64)| b64.to_owned())
                            .unwrap_or(data_url);
                        camera_photo.set(Some(encoded));
                        error_msg.set(String::new());
                    }
                    Err(_) => error_msg.set("Failed to read captured photo".to_owned()),
                }
            });
        })
    };

    let submit_leave = {
        let access_token = access_token.clone();
        let csrf_token = csrf_token.clone();
        let leave_records = leave_records.clone();
        let error_msg = error_msg.clone();
        Callback::from(move |_| {
            let token = (*access_token).clone();
            let csrf = (*csrf_token).clone();
            let leave_records = leave_records.clone();
            let error_msg = error_msg.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let body = serde_json::json!({
                    "leave_type": "Vjetor",
                    "start_date": "2026-07-01",
                    "end_date": "2026-07-05"
                });

                let mut req = Request::post(&format!("{}/api/leave", API_BASE))
                    .body(body.to_string())
                    .expect("valid request");

                let _ = req.headers().set("Content-Type", "application/json");

                if let Some(tk) = token.clone() {
                    let _ = req
                        .headers()
                        .set("Authorization", &format!("Bearer {}", tk));
                }
                if let Some(cs) = csrf {
                    let _ = req.headers().set("x-csrf-token", &cs);
                }

                if req.send().await.is_ok() {
                    let mut list_req = Request::get(&format!("{}/api/leave", API_BASE));
                    let _ = token;
                    if let Ok(resp) = list_req.send().await {
                        if let Ok(items) = resp.json::<Vec<LeaveRecord>>().await {
                            leave_records.set(items);
                        }
                    }
                } else {
                    error_msg.set("Leave request failed".to_owned());
                }
            });
        })
    };

    let run_payroll = {
        let access_token = access_token.clone();
        let csrf_token = csrf_token.clone();
        let employees = employees.clone();
        let payroll_result = payroll_result.clone();
        let error_msg = error_msg.clone();
        Callback::from(move |_| {
            let token = (*access_token).clone();
            let csrf = (*csrf_token).clone();
            let employee_id = employees
                .first()
                .map(|e| e.id.clone())
                .unwrap_or_default();
            let payroll_result = payroll_result.clone();
            let error_msg = error_msg.clone();

            wasm_bindgen_futures::spawn_local(async move {
                if employee_id.is_empty() {
                    error_msg.set("No employee selected for payroll".to_owned());
                    return;
                }

                let body = serde_json::json!({
                    "employee_id": employee_id,
                    "month_label": "2026-03",
                    "base_hourly_rate": 6.5,
                    "worked_hours": 168.0,
                    "extra_hours": 18.0,
                    "bonus_eur": 40.0,
                    "deduction_eur": 20.0,
                    "tier2_rate_multiplier": 1.3,
                    "tier3_rate_multiplier": 1.6
                });

                let mut req = Request::post(&format!("{}/api/payroll/calculate", API_BASE))
                    .body(body.to_string())
                    .expect("valid request");

                let _ = req.headers().set("Content-Type", "application/json");

                if let Some(tk) = token {
                    let _ = req
                        .headers()
                        .set("Authorization", &format!("Bearer {}", tk));
                }
                if let Some(cs) = csrf {
                    let _ = req.headers().set("x-csrf-token", &cs);
                }

                if let Ok(resp) = req.send().await {
                    if let Ok(result) = resp.json::<PayrollResult>().await {
                        payroll_result.set(Some(result));
                    }
                }
            });
        })
    };

    let load_management = {
        let access_token = access_token.clone();
        let departments = departments.clone();
        let job_positions = job_positions.clone();
        let contracts = contracts.clone();
        let salary_elements = salary_elements.clone();
        Callback::from(move |_| {
            let token = (*access_token).clone();
            let departments = departments.clone();
            let job_positions = job_positions.clone();
            let contracts = contracts.clone();
            let salary_elements = salary_elements.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut departments_req = Request::get(&format!("{}/api/company/departments", API_BASE));
                if let Some(tk) = token.clone() {
                    departments_req = departments_req.header("Authorization", &format!("Bearer {}", tk));
                }
                if let Ok(resp) = departments_req.send().await {
                    if let Ok(items) = resp.json::<Vec<Department>>().await {
                        departments.set(items);
                    }
                }

                if let Ok(resp) = Request::get(&format!("{}/api/company/job-positions", API_BASE)).send().await {
                    if let Ok(items) = resp.json::<Vec<JobPosition>>().await {
                        job_positions.set(items);
                    }
                }

                let mut contracts_req = Request::get(&format!("{}/api/contracts", API_BASE));
                if let Some(tk) = token.clone() {
                    contracts_req = contracts_req.header("Authorization", &format!("Bearer {}", tk));
                }
                if let Ok(resp) = contracts_req.send().await {
                    if let Ok(items) = resp.json::<Vec<ContractRecord>>().await {
                        contracts.set(items);
                    }
                }

                let mut salary_req = Request::get(&format!("{}/api/salary-elements", API_BASE));
                if let Some(tk) = token {
                    salary_req = salary_req.header("Authorization", &format!("Bearer {}", tk));
                }
                if let Ok(resp) = salary_req.send().await {
                    if let Ok(items) = resp.json::<Vec<SalaryElementRecord>>().await {
                        salary_elements.set(items);
                    }
                }
            });
        })
    };

    let create_department = {
        let access_token = access_token.clone();
        let csrf_token = csrf_token.clone();
        let branches = branches.clone();
        let department_name = department_name.clone();
        let error_msg = error_msg.clone();
        let load_management = load_management.clone();
        Callback::from(move |_| {
            let token = (*access_token).clone();
            let csrf = (*csrf_token).clone();
            let branches = (*branches).clone();
            let name = (*department_name).clone();
            let error_msg = error_msg.clone();
            let load_management = load_management.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let Some(branch) = branches.first() else {
                    error_msg.set("No branch available for department creation".to_owned());
                    return;
                };
                let body = serde_json::json!({ "branch_id": branch.id, "name": name });
                let mut req = Request::post(&format!("{}/api/company/departments", API_BASE))
                    .body(body.to_string())
                    .expect("valid request");
                let _ = req.headers().set("Content-Type", "application/json");
                if let Some(tk) = token {
                    let _ = req.headers().set("Authorization", &format!("Bearer {}", tk));
                }
                if let Some(cs) = csrf {
                    let _ = req.headers().set("x-csrf-token", &cs);
                }
                if req.send().await.is_ok() {
                    load_management.emit(());
                }
            });
        })
    };

    let create_job_position = {
        let access_token = access_token.clone();
        let csrf_token = csrf_token.clone();
        let job_name = job_name.clone();
        let load_management = load_management.clone();
        Callback::from(move |_| {
            let token = (*access_token).clone();
            let csrf = (*csrf_token).clone();
            let name = (*job_name).clone();
            let load_management = load_management.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let body = serde_json::json!({ "name": name, "description": "Created from web management tab" });
                let mut req = Request::post(&format!("{}/api/company/job-positions", API_BASE))
                    .body(body.to_string())
                    .expect("valid request");
                let _ = req.headers().set("Content-Type", "application/json");
                if let Some(tk) = token {
                    let _ = req.headers().set("Authorization", &format!("Bearer {}", tk));
                }
                if let Some(cs) = csrf {
                    let _ = req.headers().set("x-csrf-token", &cs);
                }
                if req.send().await.is_ok() {
                    load_management.emit(());
                }
            });
        })
    };

    let create_contract = {
        let access_token = access_token.clone();
        let csrf_token = csrf_token.clone();
        let employees = employees.clone();
        let error_msg = error_msg.clone();
        let load_management = load_management.clone();
        Callback::from(move |_| {
            let token = (*access_token).clone();
            let csrf = (*csrf_token).clone();
            let employees = (*employees).clone();
            let error_msg = error_msg.clone();
            let load_management = load_management.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let Some(employee) = employees.first() else {
                    error_msg.set("Load employees before creating contracts".to_owned());
                    return;
                };
                let body = serde_json::json!({
                    "employee_id": employee.id,
                    "contract_type": "FullTime",
                    "start_date": "2026-03-01",
                    "end_date": null,
                    "base_salary_eur": 650.0,
                    "coefficient": 1.0,
                    "status": "active"
                });
                let mut req = Request::post(&format!("{}/api/contracts", API_BASE))
                    .body(body.to_string())
                    .expect("valid request");
                let _ = req.headers().set("Content-Type", "application/json");
                if let Some(tk) = token {
                    let _ = req.headers().set("Authorization", &format!("Bearer {}", tk));
                }
                if let Some(cs) = csrf {
                    let _ = req.headers().set("x-csrf-token", &cs);
                }
                if req.send().await.is_ok() {
                    load_management.emit(());
                }
            });
        })
    };

    let create_salary_element = {
        let access_token = access_token.clone();
        let csrf_token = csrf_token.clone();
        let employees = employees.clone();
        let salary_element_name = salary_element_name.clone();
        let salary_element_amount = salary_element_amount.clone();
        let error_msg = error_msg.clone();
        let load_management = load_management.clone();
        Callback::from(move |_| {
            let token = (*access_token).clone();
            let csrf = (*csrf_token).clone();
            let employees = (*employees).clone();
            let name = (*salary_element_name).clone();
            let amount = (*salary_element_amount).parse::<f64>().unwrap_or(0.0);
            let error_msg = error_msg.clone();
            let load_management = load_management.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let Some(employee) = employees.first() else {
                    error_msg.set("Load employees before creating salary elements".to_owned());
                    return;
                };
                let body = serde_json::json!({
                    "employee_id": employee.id,
                    "element_name": name,
                    "amount": amount,
                    "period_label": "2026-03"
                });
                let mut req = Request::post(&format!("{}/api/salary-elements", API_BASE))
                    .body(body.to_string())
                    .expect("valid request");
                let _ = req.headers().set("Content-Type", "application/json");
                if let Some(tk) = token {
                    let _ = req.headers().set("Authorization", &format!("Bearer {}", tk));
                }
                if let Some(cs) = csrf {
                    let _ = req.headers().set("x-csrf-token", &cs);
                }
                if req.send().await.is_ok() {
                    load_management.emit(());
                }
            });
        })
    };

    let register_form = {
        let emp_positions_list = (*employee_positions).clone();
        let branch_list = (*branches).clone();
        let cur_pozita = (*reg_pozita).clone();
        let cur_country = (*reg_country).clone();
        let cur_branch = (*reg_branch_id).clone();
        let cur_muni = (*reg_municipality).clone();
        html! {
            <section class="card grid-form">
                <h3>{"Regjistrimi i Punonjësit"}</h3>
                { field("Emri", "Arta") }
                { field("Mbiemri", "Krasniqi") }
                { date_field("Data e Lindjes", "1995-02-10") }
                <div class="field">
                    <label>{"Shteti"}</label>
                    <select onchange={{
                        let reg_country = reg_country.clone();
                        Callback::from(move |e: Event| {
                            let sel: HtmlSelectElement = e.target_unchecked_into();
                            reg_country.set(sel.value());
                        })
                    }}>
                        <option value="Kosova" selected={cur_country == "Kosova"}>{"Kosova"}</option>
                        <option value="Shqipëria" selected={cur_country == "Shqipëria"}>{"Shqipëria"}</option>
                        <option value="Maqedonia" selected={cur_country == "Maqedonia"}>{"Maqedonia"}</option>
                    </select>
                </div>
                { field("Numri Personal", "123456789") }
                { field("Numri i Punës", "WID-1001") }
                { field("Adresa", "Prishtinë") }
                <div class="field">
                    <label>{"Komuna"}</label>
                    <select onchange={{
                        let reg_municipality = reg_municipality.clone();
                        Callback::from(move |e: Event| {
                            let sel: HtmlSelectElement = e.target_unchecked_into();
                            reg_municipality.set(sel.value());
                        })
                    }}>
                        { for KOSOVO_MUNICIPALITIES.iter().map(|m| {
                            let is_sel = cur_muni == *m;
                            let mv = m.to_string();
                            html! { <option value={mv.clone()} selected={is_sel}>{mv}</option> }
                        }) }
                    </select>
                </div>
                { field("Tel", "+38344111222") }
                { field("Email Zyrtar", "employee@example.com") }
                { date_field("Data e Punësimit", "2026-01-15") }
                <div class="field">
                    <label>{"Pozita"}</label>
                    <select onchange={{
                        let reg_pozita = reg_pozita.clone();
                        Callback::from(move |e: Event| {
                            let sel: HtmlSelectElement = e.target_unchecked_into();
                            reg_pozita.set(sel.value());
                        })
                    }}>
                        { for emp_positions_list.iter().map(|p| {
                            let is_sel = cur_pozita == *p;
                            let pv = p.clone();
                            html! { <option value={pv.clone()} selected={is_sel}>{pv}</option> }
                        }) }
                    </select>
                </div>
                <div class="field">
                    <label>{"Dega"}</label>
                    <select onchange={{
                        let reg_branch_id = reg_branch_id.clone();
                        Callback::from(move |e: Event| {
                            let sel: HtmlSelectElement = e.target_unchecked_into();
                            reg_branch_id.set(sel.value());
                        })
                    }}>
                        { for branch_list.iter().map(|b| {
                            let is_sel = cur_branch == b.id;
                            let bid = b.id.clone();
                            let bname = b.name.clone();
                            html! { <option value={bid} selected={is_sel}>{bname}</option> }
                        }) }
                    </select>
                </div>
                { field("Statusi Martesor", "Beqar") }
                { field("Edukimi", "Bachelor") }
                { field("Kontakt Emergjent", "Prind") }
                { field("Lidhja Familjare", "Nënë") }
                { field("Telefoni Emergjent", "+38344111333") }
            </section>
        }
    };

    html! {
        <div class="shell">
            <header class="topbar">
                <h1>{"Prezenca, HR & Paga"}</h1>
                <div class="actions">
                    <button class="btn cozy" onclick={{ let show_login_popup = show_login_popup.clone(); Callback::from(move |_| show_login_popup.set(true)) }}>{"Hyrje"}</button>
                </div>
            </header>

            if *view != View::Home {
                <nav class="mega-menu">
                    { for menu_items().into_iter().map(|(title, subs)| html!{
                        <div class="menu-item">
                            <button class="menu-title" onclick={{
                                let view = view.clone();
                                let load_management = load_management.clone();
                                let title_label = title.to_owned();
                                Callback::from(move |_| {
                                    if let Some(next_view) = menu_title_target(&title_label) {
                                        if next_view == View::Management {
                                            load_management.emit(());
                                        }
                                        view.set(next_view);
                                    }
                                })
                            }}>{title}</button>
                            <div class="submenu">
                                {
                                    for subs.into_iter().map(|s| {
                                        let label = s.to_owned();
                                        let view = view.clone();
                                        let load_management = load_management.clone();
                                        html! {
                                            <a onclick={Callback::from(move |_| {
                                                if let Some(next_view) = menu_target(&label) {
                                                    if next_view == View::Management {
                                                        load_management.emit(());
                                                    }
                                                    view.set(next_view);
                                                }
                                            })}>{s}</a>
                                        }
                                    })
                                }
                            </div>
                        </div>
                    }) }
                </nav>
            }

            <main>
                if *view == View::Home {
                    <section class="hero">
                        <h2>{"Zgjedh degën"}</h2>
                        <div class="branch-grid">
                            { for branches.iter().map(|b| {
                                let bid = b.id.clone();
                                let selected_branch = selected_branch.clone();
                                let load_employees = load_employees.clone();
                                html! {
                                    <button class="branch-btn" onclick={Callback::from(move |_| {
                                        selected_branch.set(Some(bid.clone()));
                                        load_employees.emit(());
                                    })}>{ &b.name }</button>
                                }
                            }) }
                        </div>

                        <div class="card">
                            <h3>{"Punonjësit e Degës"}</h3>
                            <div class="employee-inline">
                                { for employees.iter().map(|e| {
                                    let selected_employee = selected_employee.clone();
                                    let employee_id = e.id.clone();
                                    html! {
                                        <button class="employee-pill" onclick={Callback::from(move |_| selected_employee.set(Some(employee_id.clone())))}>{ format!("{} {}", e.name, e.surname) }</button>
                                    }
                                }) }
                            </div>
                            if let Some(id) = &*selected_employee {
                                <p>{format!("Punonjësi i zgjedhur: {}", id)}</p>
                            }
                            <div class="view-tabs">
                                <button onclick={{ let quick_punch = quick_punch.clone(); Callback::from(move |_| quick_punch.emit("clock_in".to_owned())) }}>{"Hyrje"}</button>
                                <button onclick={{ let quick_punch = quick_punch.clone(); Callback::from(move |_| quick_punch.emit("clock_out".to_owned())) }}>{"Dalje"}</button>
                            </div>
                        </div>
                    </section>
                } else {
                    <section class="widgets">
                        <div class="card"><h3>{"Prezenca"}</h3><p>{format!("{} punonjës të listuar", employees.len())}</p></div>
                        <div class="card"><h3>{"Ditëlindjet/Pushimet e ardhshme"}</h3><p>{"Arta - 10 Shk, Kërkesat e pushimit në skedën Pushimi"}</p></div>
                        <div class="card">
                            <h3>{"Hyrje e Shpejtë"}</h3>
                            <button onclick={capture_photo.clone()}>{"Kap Foto me Kamerë"}</button>
                            <button onclick={Callback::from(move |_| quick_punch.emit("clock_in".to_owned()))}>{"Hyrje"}</button>
                            if camera_photo.is_some() {
                                <p>{"Foto e kamerës e gatshme"}</p>
                            } else {
                                <p>{"Nuk është kapur asnjë foto"}</p>
                            }
                        </div>
                    </section>

                    <section class="view-tabs">
                        <button onclick={{ let view = view.clone(); Callback::from(move |_| view.set(View::Dashboard)) }}>{"Paneli"}</button>
                        <button onclick={{ let view = view.clone(); Callback::from(move |_| view.set(View::EmployeeRegister)) }}>{"Regjistro Punonjës"}</button>
                        <button onclick={{ let view = view.clone(); Callback::from(move |_| view.set(View::Vacation)) }}>{"Pushimi"}</button>
                        <button onclick={{ let view = view.clone(); Callback::from(move |_| view.set(View::Payroll)) }}>{"Paga"}</button>
                        <button onclick={{ let view = view.clone(); let load_management = load_management.clone(); Callback::from(move |_| { view.set(View::Management); load_management.emit(()); }) }}>{"Menaxhimi"}</button>
                    </section>

                    if *view == View::EmployeeRegister {
                        {register_form}
                    }

                    if *view == View::Vacation {
                        <section class="card">
                            <h3>{"Pushimi & Kalendari i Festave"}</h3>
                            <p>{"Bajrami Madh, Bajrami Vogël, Krishtlindjet, Viti i Ri, Dita e Pavarësisë"}</p>
                            <button onclick={submit_leave}>{"Kërko Pushim"}</button>
                            <ul>{ for leave_records.iter().map(|l| html!{<li>{format!("{} {}-{} [{}]", l.leave_type, fmt_date(&l.start_date), fmt_date(&l.end_date), l.status)}</li>}) }</ul>
                        </section>
                    }

                    if *view == View::Payroll {
                        <section class="card">
                            <h3>{"Llogaritja e Pagës (EUR, rregullat e Kosovës)"}</h3>
                            <p>{"Si parazgjedhje: 20 ditë * 8h = 160 orë standarde; mbikoha >160h; premium >200h"}</p>
                            <button onclick={run_payroll}>{"Llogarit Pagën"}</button>
                            if let Some(res) = &*payroll_result {
                                <p>{format!("{} -> Bruto {:.2} EUR / Neto {:.2} EUR", res.month_label, res.gross_total, res.net_total)}</p>
                                <code>{&res.edi_line}</code>
                            }
                        </section>
                    }

                    if *view == View::Management {
                        <section class="widgets">
                            <div class="card">
                                <h3>{"Statusi i Punonjësit / Pozitat"}</h3>
                                <div class="view-tabs">
                                    <input
                                        placeholder="Shto pozitë të re..."
                                        value={(*new_position_input).clone()}
                                        oninput={{
                                            let new_position_input = new_position_input.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                new_position_input.set(input.value());
                                            })
                                        }}
                                    />
                                    <button onclick={{
                                        let employee_positions = employee_positions.clone();
                                        let new_position_input = new_position_input.clone();
                                        Callback::from(move |_| {
                                            let name = (*new_position_input).trim().to_owned();
                                            if name.is_empty() { return; }
                                            let mut next = (*employee_positions).clone();
                                            next.push(name);
                                            employee_positions.set(next);
                                            new_position_input.set(String::new());
                                        })
                                    }} class="icon-btn icon-btn--add">{"+"}</button>
                                </div>
                                <ul>{ for (*employee_positions).iter().enumerate().map(|(i, pos)| {
                                    let emp_pos = employee_positions.clone();
                                    let pos_name = pos.clone();
                                    html! {
                                        <li>
                                            <span>{pos_name}</span>
                                            <button class="icon-btn icon-btn--delete" onclick={Callback::from(move |_| {
                                                let mut next = (*emp_pos).clone();
                                                next.remove(i);
                                                emp_pos.set(next);
                                            })}>{"🗑"}</button>
                                        </li>
                                    }
                                }) }</ul>
                            </div>
                            <div class="card">
                                <h3>{"Departamentet"}</h3>
                                <input
                                    value={(*department_name).clone()}
                                    oninput={{
                                        let department_name = department_name.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            department_name.set(input.value());
                                        })
                                    }}
                                />
                                <button onclick={create_department}>{"Krijo Departament"}</button>
                                <ul>{ for departments.iter().map(|d| {
                                    let department_id = d.id.clone();
                                    let department_name_value = d.name.clone();

                                    let edit_department = {
                                        let access_token = access_token.clone();
                                        let csrf_token = csrf_token.clone();
                                        let load_management = load_management.clone();
                                        let department_id = department_id.clone();
                                        let department_name_value = department_name_value.clone();
                                        Callback::from(move |_| {
                                            let Some(win) = window() else { return; };
                                            let Ok(Some(next_name)) = win.prompt_with_message_and_default("Emri i departamentit", &department_name_value) else { return; };
                                            let token = (*access_token).clone();
                                            let csrf = (*csrf_token).clone();
                                            let load_management = load_management.clone();
                                            let department_id = department_id.clone();
                                            wasm_bindgen_futures::spawn_local(async move {
                                                let body = serde_json::json!({ "name": next_name });
                                                let mut req = Request::put(&format!("{}/api/company/departments/{}", API_BASE, department_id))
                                                    .body(body.to_string())
                                                    .expect("valid request");
                                                let _ = req.headers().set("Content-Type", "application/json");
                                                if let Some(tk) = token {
                                                    let _ = req.headers().set("Authorization", &format!("Bearer {}", tk));
                                                }
                                                if let Some(cs) = csrf {
                                                    let _ = req.headers().set("x-csrf-token", &cs);
                                                }
                                                if req.send().await.is_ok() {
                                                    load_management.emit(());
                                                }
                                            });
                                        })
                                    };

                                    let delete_department = {
                                        let access_token = access_token.clone();
                                        let csrf_token = csrf_token.clone();
                                        let load_management = load_management.clone();
                                        let department_id = department_id.clone();
                                        Callback::from(move |_| {
                                            let Some(win) = window() else { return; };
                                            if !win.confirm_with_message("A e fshini këtë departament?").unwrap_or(false) {
                                                return;
                                            }
                                            let token = (*access_token).clone();
                                            let csrf = (*csrf_token).clone();
                                            let load_management = load_management.clone();
                                            let department_id = department_id.clone();
                                            wasm_bindgen_futures::spawn_local(async move {
                                                let mut req = Request::delete(&format!("{}/api/company/departments/{}", API_BASE, department_id));
                                                if let Some(tk) = token {
                                                    req = req.header("Authorization", &format!("Bearer {}", tk));
                                                }
                                                if let Some(cs) = csrf {
                                                    req = req.header("x-csrf-token", &cs);
                                                }
                                                if req.send().await.is_ok() {
                                                    load_management.emit(());
                                                }
                                            });
                                        })
                                    };

                                    html! {
                                        <li>
                                            <span>{department_name_value}</span>
                                            <button class="icon-btn icon-btn--edit" onclick={edit_department}>{"✏"}</button>
                                            <button class="icon-btn icon-btn--delete" onclick={delete_department}>{"🗑"}</button>
                                        </li>
                                    }
                                }) }</ul>
                            </div>
                            <div class="card">
                                <h3>{"Pozitat e Punës"}</h3>
                                <input
                                    value={(*job_name).clone()}
                                    oninput={{
                                        let job_name = job_name.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            job_name.set(input.value());
                                        })
                                    }}
                                />
                                <button onclick={create_job_position}>{"Krijo Pozitë Pune"}</button>
                                <ul>{ for job_positions.iter().map(|p| {
                                    let position_id = p.id.clone();
                                    let position_name = p.name.clone();
                                    let position_description = p.description.clone();

                                    let edit_position = {
                                        let access_token = access_token.clone();
                                        let csrf_token = csrf_token.clone();
                                        let load_management = load_management.clone();
                                        let position_id = position_id.clone();
                                        let position_name = position_name.clone();
                                        let position_description = position_description.clone();
                                        Callback::from(move |_| {
                                            let Some(win) = window() else { return; };
                                            let Ok(Some(next_name)) = win.prompt_with_message_and_default("Emri i pozitës së punës", &position_name) else { return; };
                                            let token = (*access_token).clone();
                                            let csrf = (*csrf_token).clone();
                                            let load_management = load_management.clone();
                                            let position_id = position_id.clone();
                                            let position_description = position_description.clone();
                                            wasm_bindgen_futures::spawn_local(async move {
                                                let body = serde_json::json!({
                                                    "name": next_name,
                                                    "description": position_description
                                                });
                                                let mut req = Request::put(&format!("{}/api/company/job-positions/{}", API_BASE, position_id))
                                                    .body(body.to_string())
                                                    .expect("valid request");
                                                let _ = req.headers().set("Content-Type", "application/json");
                                                if let Some(tk) = token {
                                                    let _ = req.headers().set("Authorization", &format!("Bearer {}", tk));
                                                }
                                                if let Some(cs) = csrf {
                                                    let _ = req.headers().set("x-csrf-token", &cs);
                                                }
                                                if req.send().await.is_ok() {
                                                    load_management.emit(());
                                                }
                                            });
                                        })
                                    };

                                    let delete_position = {
                                        let access_token = access_token.clone();
                                        let csrf_token = csrf_token.clone();
                                        let load_management = load_management.clone();
                                        let position_id = position_id.clone();
                                        Callback::from(move |_| {
                                            let Some(win) = window() else { return; };
                                            if !win.confirm_with_message("A e fshini këtë pozitë pune?").unwrap_or(false) {
                                                return;
                                            }
                                            let token = (*access_token).clone();
                                            let csrf = (*csrf_token).clone();
                                            let load_management = load_management.clone();
                                            let position_id = position_id.clone();
                                            wasm_bindgen_futures::spawn_local(async move {
                                                let mut req = Request::delete(&format!("{}/api/company/job-positions/{}", API_BASE, position_id));
                                                if let Some(tk) = token {
                                                    req = req.header("Authorization", &format!("Bearer {}", tk));
                                                }
                                                if let Some(cs) = csrf {
                                                    req = req.header("x-csrf-token", &cs);
                                                }
                                                if req.send().await.is_ok() {
                                                    load_management.emit(());
                                                }
                                            });
                                        })
                                    };

                                    html! {
                                        <li>
                                            <span>{position_name}</span>
                                            <button class="icon-btn icon-btn--edit" onclick={edit_position}>{"✏"}</button>
                                            <button class="icon-btn icon-btn--delete" onclick={delete_position}>{"🗑"}</button>
                                        </li>
                                    }
                                }) }</ul>
                            </div>
                            <div class="card">
                                <h3>{"Kontratat"}</h3>
                                <button onclick={create_contract}>{"Krijo Kontratë për Punonjësin e Parë"}</button>
                                <ul>{ for contracts.iter().take(8).map(|c| {
                                    let contract_id = c.id.clone();
                                    let contract_type = c.contract_type.clone();
                                    let start_date = c.start_date.clone();
                                    let end_date = c.end_date.clone();
                                    let base_salary_eur = c.base_salary_eur;
                                    let coefficient = c.coefficient;
                                    let status = c.status.clone();

                                    let contract_type_value = (*contract_type_inputs)
                                        .get(&contract_id)
                                        .cloned()
                                        .unwrap_or(contract_type.clone());
                                    let contract_salary_value = (*contract_salary_inputs)
                                        .get(&contract_id)
                                        .cloned()
                                        .unwrap_or(format!("{:.2}", base_salary_eur));
                                    let contract_status_value = (*contract_status_inputs)
                                        .get(&contract_id)
                                        .cloned()
                                        .unwrap_or(status.clone());

                                    let on_contract_type_change = {
                                        let contract_type_inputs = contract_type_inputs.clone();
                                        let contract_id = contract_id.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            let mut next = (*contract_type_inputs).clone();
                                            next.insert(contract_id.clone(), input.value());
                                            contract_type_inputs.set(next);
                                        })
                                    };

                                    let on_contract_salary_change = {
                                        let contract_salary_inputs = contract_salary_inputs.clone();
                                        let contract_id = contract_id.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            let mut next = (*contract_salary_inputs).clone();
                                            next.insert(contract_id.clone(), input.value());
                                            contract_salary_inputs.set(next);
                                        })
                                    };

                                    let on_contract_status_change = {
                                        let contract_status_inputs = contract_status_inputs.clone();
                                        let contract_id = contract_id.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            let mut next = (*contract_status_inputs).clone();
                                            next.insert(contract_id.clone(), input.value());
                                            contract_status_inputs.set(next);
                                        })
                                    };

                                    let edit_contract = {
                                        let access_token = access_token.clone();
                                        let csrf_token = csrf_token.clone();
                                        let load_management = load_management.clone();
                                        let contract_type_inputs = contract_type_inputs.clone();
                                        let contract_salary_inputs = contract_salary_inputs.clone();
                                        let contract_status_inputs = contract_status_inputs.clone();
                                        let contract_id = contract_id.clone();
                                        let contract_type = contract_type.clone();
                                        let start_date = start_date.clone();
                                        let end_date = end_date.clone();
                                        let status = status.clone();
                                        Callback::from(move |_| {
                                            let token = (*access_token).clone();
                                            let csrf = (*csrf_token).clone();
                                            let load_management = load_management.clone();
                                            let contract_id = contract_id.clone();
                                            let start_date = start_date.clone();
                                            let end_date = end_date.clone();
                                            let contract_type = (*contract_type_inputs)
                                                .get(&contract_id)
                                                .cloned()
                                                .unwrap_or(contract_type.clone());
                                            let salary = (*contract_salary_inputs)
                                                .get(&contract_id)
                                                .and_then(|v| v.parse::<f64>().ok())
                                                .unwrap_or(base_salary_eur);
                                            let status = (*contract_status_inputs)
                                                .get(&contract_id)
                                                .cloned()
                                                .unwrap_or(status.clone());
                                            wasm_bindgen_futures::spawn_local(async move {
                                                let body = serde_json::json!({
                                                    "contract_type": contract_type,
                                                    "start_date": start_date,
                                                    "end_date": end_date,
                                                    "base_salary_eur": salary,
                                                    "coefficient": coefficient,
                                                    "status": status
                                                });
                                                let mut req = Request::put(&format!("{}/api/contracts/{}", API_BASE, contract_id))
                                                    .body(body.to_string())
                                                    .expect("valid request");
                                                let _ = req.headers().set("Content-Type", "application/json");
                                                if let Some(tk) = token {
                                                    let _ = req.headers().set("Authorization", &format!("Bearer {}", tk));
                                                }
                                                if let Some(cs) = csrf {
                                                    let _ = req.headers().set("x-csrf-token", &cs);
                                                }
                                                if req.send().await.is_ok() {
                                                    load_management.emit(());
                                                }
                                            });
                                        })
                                    };

                                    let delete_contract = {
                                        let access_token = access_token.clone();
                                        let csrf_token = csrf_token.clone();
                                        let load_management = load_management.clone();
                                        let contract_id = contract_id.clone();
                                        Callback::from(move |_| {
                                            let Some(win) = window() else { return; };
                                            if !win.confirm_with_message("A e fshini këtë kontratë?").unwrap_or(false) {
                                                return;
                                            }
                                            let token = (*access_token).clone();
                                            let csrf = (*csrf_token).clone();
                                            let load_management = load_management.clone();
                                            let contract_id = contract_id.clone();
                                            wasm_bindgen_futures::spawn_local(async move {
                                                let mut req = Request::delete(&format!("{}/api/contracts/{}", API_BASE, contract_id));
                                                if let Some(tk) = token {
                                                    req = req.header("Authorization", &format!("Bearer {}", tk));
                                                }
                                                if let Some(cs) = csrf {
                                                    req = req.header("x-csrf-token", &cs);
                                                }
                                                if req.send().await.is_ok() {
                                                    load_management.emit(());
                                                }
                                            });
                                        })
                                    };

                                    html! {
                                        <li>
                                            <div class="view-tabs">
                                                <input value={contract_type_value} oninput={on_contract_type_change} />
                                                <input value={contract_salary_value} oninput={on_contract_salary_change} />
                                                <input value={contract_status_value} oninput={on_contract_status_change} />
                                                <button class="icon-btn icon-btn--edit" onclick={edit_contract}>{"✓"}</button>
                                                <button class="icon-btn icon-btn--delete" onclick={delete_contract}>{"🗑"}</button>
                                            </div>
                                        </li>
                                    }
                                }) }</ul>
                            </div>
                        </section>

                        <section class="card">
                            <h3>{"Elementet e Pagës"}</h3>
                            <div class="view-tabs">
                                <input
                                    value={(*salary_element_name).clone()}
                                    oninput={{
                                        let salary_element_name = salary_element_name.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            salary_element_name.set(input.value());
                                        })
                                    }}
                                />
                                <input
                                    value={(*salary_element_amount).clone()}
                                    oninput={{
                                        let salary_element_amount = salary_element_amount.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            salary_element_amount.set(input.value());
                                        })
                                    }}
                                />
                                <button onclick={create_salary_element}>{"Shto Element Page"}</button>
                            </div>
                            <ul>{ for salary_elements.iter().take(10).map(|s| {
                                let salary_id = s.id.clone();
                                let element_name = s.element_name.clone();
                                let period_label = s.period_label.clone();
                                let amount = s.amount;

                                let salary_name_value = (*salary_name_inputs)
                                    .get(&salary_id)
                                    .cloned()
                                    .unwrap_or(element_name.clone());
                                let salary_amount_value = (*salary_amount_inputs)
                                    .get(&salary_id)
                                    .cloned()
                                    .unwrap_or(format!("{:.2}", amount));
                                let salary_period_value = (*salary_period_inputs)
                                    .get(&salary_id)
                                    .cloned()
                                    .unwrap_or(period_label.clone());

                                let on_salary_name_change = {
                                    let salary_name_inputs = salary_name_inputs.clone();
                                    let salary_id = salary_id.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        let mut next = (*salary_name_inputs).clone();
                                        next.insert(salary_id.clone(), input.value());
                                        salary_name_inputs.set(next);
                                    })
                                };

                                let on_salary_amount_change = {
                                    let salary_amount_inputs = salary_amount_inputs.clone();
                                    let salary_id = salary_id.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        let mut next = (*salary_amount_inputs).clone();
                                        next.insert(salary_id.clone(), input.value());
                                        salary_amount_inputs.set(next);
                                    })
                                };

                                let on_salary_period_change = {
                                    let salary_period_inputs = salary_period_inputs.clone();
                                    let salary_id = salary_id.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        let mut next = (*salary_period_inputs).clone();
                                        next.insert(salary_id.clone(), input.value());
                                        salary_period_inputs.set(next);
                                    })
                                };

                                let edit_salary = {
                                    let access_token = access_token.clone();
                                    let csrf_token = csrf_token.clone();
                                    let load_management = load_management.clone();
                                    let salary_name_inputs = salary_name_inputs.clone();
                                    let salary_amount_inputs = salary_amount_inputs.clone();
                                    let salary_period_inputs = salary_period_inputs.clone();
                                    let salary_id = salary_id.clone();
                                    let element_name = element_name.clone();
                                    let period_label = period_label.clone();
                                    Callback::from(move |_| {
                                        let token = (*access_token).clone();
                                        let csrf = (*csrf_token).clone();
                                        let load_management = load_management.clone();
                                        let salary_id = salary_id.clone();
                                        let element_name = (*salary_name_inputs)
                                            .get(&salary_id)
                                            .cloned()
                                            .unwrap_or(element_name.clone());
                                        let period_label = (*salary_period_inputs)
                                            .get(&salary_id)
                                            .cloned()
                                            .unwrap_or(period_label.clone());
                                        let amount = (*salary_amount_inputs)
                                            .get(&salary_id)
                                            .and_then(|v| v.parse::<f64>().ok())
                                            .unwrap_or(amount);
                                        wasm_bindgen_futures::spawn_local(async move {
                                            let body = serde_json::json!({
                                                "element_name": element_name,
                                                "amount": amount,
                                                "period_label": period_label
                                            });
                                            let mut req = Request::put(&format!("{}/api/salary-elements/{}", API_BASE, salary_id))
                                                .body(body.to_string())
                                                .expect("valid request");
                                            let _ = req.headers().set("Content-Type", "application/json");
                                            if let Some(tk) = token {
                                                let _ = req.headers().set("Authorization", &format!("Bearer {}", tk));
                                            }
                                            if let Some(cs) = csrf {
                                                let _ = req.headers().set("x-csrf-token", &cs);
                                            }
                                            if req.send().await.is_ok() {
                                                load_management.emit(());
                                            }
                                        });
                                    })
                                };

                                let delete_salary = {
                                    let access_token = access_token.clone();
                                    let csrf_token = csrf_token.clone();
                                    let load_management = load_management.clone();
                                    let salary_id = salary_id.clone();
                                    Callback::from(move |_| {
                                        let Some(win) = window() else { return; };
                                        if !win.confirm_with_message("A e fshini këtë element page?").unwrap_or(false) {
                                            return;
                                        }
                                        let token = (*access_token).clone();
                                        let csrf = (*csrf_token).clone();
                                        let load_management = load_management.clone();
                                        let salary_id = salary_id.clone();
                                        wasm_bindgen_futures::spawn_local(async move {
                                            let mut req = Request::delete(&format!("{}/api/salary-elements/{}", API_BASE, salary_id));
                                            if let Some(tk) = token {
                                                req = req.header("Authorization", &format!("Bearer {}", tk));
                                            }
                                            if let Some(cs) = csrf {
                                                req = req.header("x-csrf-token", &cs);
                                            }
                                            if req.send().await.is_ok() {
                                                load_management.emit(());
                                            }
                                        });
                                    })
                                };

                                html! {
                                    <li>
                                        <div class="view-tabs">
                                            <input value={salary_name_value} oninput={on_salary_name_change} />
                                            <input value={salary_amount_value} oninput={on_salary_amount_change} />
                                            <input value={salary_period_value} oninput={on_salary_period_change} />
                                            <button class="icon-btn icon-btn--edit" onclick={edit_salary}>{"✓"}</button>
                                            <button class="icon-btn icon-btn--delete" onclick={delete_salary}>{"🗑"}</button>
                                        </div>
                                    </li>
                                }
                            }) }</ul>
                        </section>
                    }

                    <section class="card">
                        <h3>{"Regjistri i prezencës në kohë reale"}</h3>
                        <ul>{ for attendance_events.iter().map(|e| html!{<li>{e}</li>}) }</ul>
                    </section>
                }

                if *show_login_popup {
                    <div class="modal-overlay" onclick={{ let show_login_popup = show_login_popup.clone(); Callback::from(move |_| show_login_popup.set(false)) }}>
                        <section class="card login-modal" onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                            <h3>{"Hyrje Demo"}</h3>
                            <label>{"Email"}</label>
                            <input value={(*login_email).clone()} oninput={{ let login_email = login_email.clone(); Callback::from(move |e: InputEvent| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                login_email.set(input.value());
                            }) }} />
                            <label>{"Fjalëkalimi"}</label>
                            <input type="password" value={(*login_password).clone()} oninput={{ let login_password = login_password.clone(); Callback::from(move |e: InputEvent| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                login_password.set(input.value());
                            }) }} />
                            <div class="view-tabs">
                                <button class="btn cozy" onclick={on_login.clone()}>{"Hyr"}</button>
                                <button onclick={{ let show_login_popup = show_login_popup.clone(); Callback::from(move |_| show_login_popup.set(false)) }}>{"Mbyll"}</button>
                            </div>
                            <small>{format!("Roli: {}", (*user_role).clone())}</small>
                        </section>
                    </div>
                }

                <input
                    type="file"
                    accept="image/*"
                    capture="environment"
                    onchange={on_camera_selected}
                    ref={camera_input_ref}
                    style="display:none"
                />

                if !error_msg.is_empty() {
                    <p class="error">{(*error_msg).clone()}</p>
                }
            </main>
        </div>
    }
}

fn field(title: &str, placeholder: &str) -> Html {
    html! {
        <div class="field">
            <label>{title}</label>
            <input placeholder={placeholder.to_owned()} />
            <small>{"E detyrueshme"}</small>
        </div>
    }
}

fn date_field(title: &str, default_val: &str) -> Html {
    html! {
        <div class="field">
            <label>{title}</label>
            <input type="date" value={default_val.to_owned()} />
            <small>{"Format: dd-mm-yyyy"}</small>
        </div>
    }
}

/// Converts ISO `yyyy-mm-dd` to display format `dd-mm-yyyy`.
fn fmt_date(iso: &str) -> String {
    let p: Vec<&str> = iso.split('-').collect();
    if p.len() == 3 { format!("{}-{}-{}", p[2], p[1], p[0]) } else { iso.to_owned() }
}
