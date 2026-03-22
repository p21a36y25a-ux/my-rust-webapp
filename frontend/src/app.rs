use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::{EventSource, HtmlInputElement};
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

#[derive(Clone, Debug, PartialEq)]
enum View {
    Home,
    Dashboard,
    EmployeeRegister,
    Vacation,
    Payroll,
}

fn menu_items() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("Employee", vec!["Register Employees", "Click-in", "Register Contracts", "Employee Files", "Employee Status"]),
        ("Salary/Compensation", vec!["Salary Determination", "Salary Period", "Additional Days/Hours", "Additional Income", "Salary Calculation", "Payroll List", "E-Declaration"]),
        ("Vacation", vec!["Vacation Request", "Vacation Hours", "Holiday Status", "Holiday Calendar"]),
        ("Click-in/Click-out", vec!["Record Click", "Open Entries", "Click List", "Employees Present"]),
        ("HR Definitions", vec!["Employee Status", "Contract Types", "Employer Type", "Vacation Types", "Probation Types", "Element Calculation", "Coefficient", "Salary Elements"]),
        ("Company", vec!["Company Details", "Branches", "Departments/Units", "Job Positions"]),
        ("Administration", vec!["Municipal Registration", "State Registration", "Bank Registration", "Marital Status"]),
    ]
}

#[function_component(App)]
pub fn app() -> Html {
    let branches = use_state(Vec::<Branch>::new);
    let selected_branch = use_state(|| None::<String>);
    let employees = use_state(Vec::<Employee>::new);
    let attendance_events = use_state(Vec::<String>::new);
    let leave_records = use_state(Vec::<LeaveRecord>::new);
    let payroll_result = use_state(|| None::<PayrollResult>);

    let access_token = use_state(|| LocalStorage::get::<String>("access_token").ok());
    let csrf_token = use_state(|| LocalStorage::get::<String>("csrf_token").ok());
    let user_role = use_state(|| LocalStorage::get::<String>("role").unwrap_or_else(|_| "guest".to_owned()));
    let view = use_state(|| View::Home);
    let error_msg = use_state(String::new);

    let login_email = use_state(|| "system_admin@example.com".to_owned());
    let login_password = use_state(|| "Password123!".to_owned());

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
        let error_msg = error_msg.clone();
        Callback::from(move |_| {
            let email = (*login_email).clone();
            let password = (*login_password).clone();
            let access_token = access_token.clone();
            let csrf_token = csrf_token.clone();
            let user_role = user_role.clone();
            let view = view.clone();
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
        let error_msg = error_msg.clone();
        Callback::from(move |click_type: String| {
            let token = (*access_token).clone();
            let csrf = (*csrf_token).clone();
            let employees = (*employees).clone();
            let error_msg = error_msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if employees.is_empty() {
                    error_msg.set("Select branch and load employees first".to_owned());
                    return;
                }
                let employee_id = employees[0].id.clone();
                let photo = format!("camera-demo:{}:{}", click_type, js_sys::Date::now());
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

    let register_form = html! {
        <section class="card grid-form">
            <h3>{"Regjistrimi i punonjesit / Employee Registration"}</h3>
            { field("Name / Emri", "Arta") }
            { field("Surname / Mbiemri", "Krasniqi") }
            { field("Birthdate / Data e lindjes", "1995-02-10") }
            { field("Country / Shteti (Kosova, Shqiperia, Maqedonia)", "Kosova") }
            { field("Personal ID / Numri personal", "123456789") }
            { field("Work ID / Numri i punes", "WID-1001") }
            { field("Address / Adresa", "Prishtine") }
            { field("Municipality / Komuna", "Prishtine") }
            { field("Tel", "+38344111222") }
            { field("Official Email / Email zyrtar", "employee@example.com") }
            { field("Employment Date / Data e punesimit", "2026-01-15") }
            { field("Marital Status / Statusi martesor", "Single") }
            { field("Education / Edukimi", "Bachelor") }
            { field("Emergency Contact / Kontakt emergjent", "Prind") }
            { field("Family Connection / Lidhja familjare", "Nene") }
            { field("Emergency Phone / Telefoni emergjent", "+38344111333") }
            <small>{"Pozitat e paracaktuara: Menaxher, Asistent menaxher, Depoist Depo, Keshilltarë per Klient, Arkitekt, Ndihmese, Shtepiak, Menaxher i Depos, Kordinator I Shitjes, Menaxher Importi, Vozites, Inxhinier i Hidros."}</small>
        </section>
    };

    html! {
        <div class="shell">
            <header class="topbar">
                <h1>{"Time Attendance, HR & Payroll"}</h1>
                <div class="actions">
                    <button class="btn cozy" onclick={on_login}>{"Log In"}</button>
                </div>
            </header>

            if *view != View::Home {
                <nav class="mega-menu">
                    { for menu_items().into_iter().map(|(title, subs)| html!{
                        <div class="menu-item">
                            <span>{title}</span>
                            <div class="submenu">
                                { for subs.into_iter().map(|s| html!{ <a>{s}</a> }) }
                            </div>
                        </div>
                    }) }
                </nav>
            }

            <main>
                if *view == View::Home {
                    <section class="hero">
                        <h2>{"Zgjedh degen / Choose a branch"}</h2>
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
                            <h3>{"Branch Employees"}</h3>
                            <ul>
                                { for employees.iter().map(|e| {
                                    let quick_punch = quick_punch.clone();
                                    let quick_punch_out = quick_punch.clone();
                                    html! {
                                        <li>
                                            <span>{ format!("{} {} - {}", e.name, e.surname, e.job_position) }</span>
                                            <button onclick={Callback::from(move |_| quick_punch.emit("clock_in".to_owned()))}>{"Clock In"}</button>
                                            <button onclick={Callback::from(move |_| quick_punch_out.emit("clock_out".to_owned()))}>{"Clock Out"}</button>
                                        </li>
                                    }
                                }) }
                            </ul>
                        </div>
                    </section>
                } else {
                    <section class="widgets">
                        <div class="card"><h3>{"Presence / Prezenca"}</h3><p>{format!("{} employees listed", employees.len())}</p></div>
                        <div class="card"><h3>{"Upcoming birthdays/leave"}</h3><p>{"Arta - 10 Feb, Leave requests in Vacation tab"}</p></div>
                        <div class="card"><h3>{"Quick Clock"}</h3><button onclick={Callback::from(move |_| quick_punch.emit("clock_in".to_owned()))}>{"Clock In"}</button></div>
                    </section>

                    <section class="view-tabs">
                        <button onclick={{ let view = view.clone(); Callback::from(move |_| view.set(View::Dashboard)) }}>{"Dashboard"}</button>
                        <button onclick={{ let view = view.clone(); Callback::from(move |_| view.set(View::EmployeeRegister)) }}>{"Register Employee"}</button>
                        <button onclick={{ let view = view.clone(); Callback::from(move |_| view.set(View::Vacation)) }}>{"Vacation"}</button>
                        <button onclick={{ let view = view.clone(); Callback::from(move |_| view.set(View::Payroll)) }}>{"Payroll"}</button>
                    </section>

                    if *view == View::EmployeeRegister {
                        {register_form}
                    }

                    if *view == View::Vacation {
                        <section class="card">
                            <h3>{"Vacation & Holidays Calendar"}</h3>
                            <p>{"Bajrami madh, Bajrami vogel, Krishtlindjet, Viti Ri, Dita e Pavarësis"}</p>
                            <button onclick={submit_leave}>{"Request Vacation"}</button>
                            <ul>{ for leave_records.iter().map(|l| html!{<li>{format!("{} {}-{} [{}]", l.leave_type, l.start_date, l.end_date, l.status)}</li>}) }</ul>
                        </section>
                    }

                    if *view == View::Payroll {
                        <section class="card">
                            <h3>{"Payroll Calculation (EUR, Kosovo rules)"}</h3>
                            <p>{"Default: 20 days * 8h = 160 standard hours; overtime >160h; premium >200h"}</p>
                            <button onclick={run_payroll}>{"Run Payroll Calculation"}</button>
                            if let Some(res) = &*payroll_result {
                                <p>{format!("{} -> Gross {:.2} EUR / Net {:.2} EUR", res.month_label, res.gross_total, res.net_total)}</p>
                                <code>{&res.edi_line}</code>
                            }
                        </section>
                    }

                    <section class="card">
                        <h3>{"Real-time attendance feed"}</h3>
                        <ul>{ for attendance_events.iter().map(|e| html!{<li>{e}</li>}) }</ul>
                    </section>
                }

                <section class="card login-box">
                    <h3>{"Demo Login"}</h3>
                    <label>{"Email"}</label>
                    <input value={(*login_email).clone()} oninput={{ let login_email = login_email.clone(); Callback::from(move |e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        login_email.set(input.value());
                    }) }} />
                    <label>{"Password"}</label>
                    <input type="password" value={(*login_password).clone()} oninput={{ let login_password = login_password.clone(); Callback::from(move |e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        login_password.set(input.value());
                    }) }} />
                    <small>{format!("Role: {}", (*user_role).clone())}</small>
                </section>

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
            <small>{"Validation: Required / Kerkohen"}</small>
        </div>
    }
}
