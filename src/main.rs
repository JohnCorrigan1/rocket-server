#[macro_use]
extern crate rocket;
use rocket::{
    serde::{json::Json, Serialize},
    tokio::time::sleep,
};
use std::str::FromStr;
use std::time::Duration;
//use rocket::data::{Data, ToByteUnit};
//use rocket::http::{ContentType, Status};
//use std::fs::File;
//use std::io::Write;
//use sys_info;
//use rocket_contrib::json::Json;
//use rocket_contrib::json::Json;

use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};

use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, System};

#[derive(Serialize)]
struct CpuInfo {
    cpu_usage: f32,
}

#[get("/cpu", format = "json")]
async fn cpu() -> Json<CpuInfo> {
    let mut system = System::new_with_specifics(
        sysinfo::RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
    );
    system.refresh_cpu_usage();
    let _ = sleep(Duration::from_millis(200)).await;
    system.refresh_cpu_usage();
    Json(CpuInfo {
        cpu_usage: system.global_cpu_info().cpu_usage(),
    })
}

#[get("/memory")]
async fn memory() -> String {
    let system = System::new_with_specifics(
        sysinfo::RefreshKind::new().with_memory(MemoryRefreshKind::everything()),
    );

    let total_memory = system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0; // in GB
    let used_memory = system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0; // in GB
    let memory_usage = used_memory / total_memory * 100.0;

    format!(
        "{{totalMemory: {:.2} usedMemory: {:.2}, memoryUsage: {:.2}}}",
        &total_memory, &used_memory, &memory_usage
    )
}

#[get("/storage")]
async fn storage() -> String {
    let disks = Disks::new_with_refreshed_list();
    let disk_usage = disks
        .iter()
        .map(|disk| {
            let available = disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0; // in GB
            let total = disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0; // in GB
            let name = disk.name().to_str();
            let usage = (total - available) / total * 100.0;
            let disk_type = disk.kind().to_string();
            //let mount_point = disk.mount_point().to_str().to_string();

            format!(
                "{{name: {}, type: {}, total: {:.2}, available: {:.2}, usage: {:.2}}}",
                name.unwrap(),
                disk_type,
                total,
                available,
                usage
            )
        })
        .collect::<Vec<String>>()
        .join(",\n");

    format!("[{}]", disk_usage)
}

#[launch]
fn rocket() -> _ {
    let cors = rocket_cors::CorsOptions::default().to_cors().unwrap();

    rocket::build()
        .attach(cors)
        .mount("/api", routes![cpu, memory, storage])
}

//#[post("/upload", data = "<data>")]
