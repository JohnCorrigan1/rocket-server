#[macro_use]
extern crate rocket;
use rocket::data::{Limits, ToByteUnit};
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::{
    serde::{json::Json, Serialize},
    tokio::time::sleep,
};
use std::os::unix::fs::PermissionsExt;
use std::{fs, path::Path, time::Duration};
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

#[derive(Serialize)]
struct MemoryInfo {
    total_memory: f64,
    used_memory: f64,
    memory_usage: f64,
}

#[get("/memory")]
async fn memory() -> Json<MemoryInfo> {
    let system = System::new_with_specifics(
        sysinfo::RefreshKind::new().with_memory(MemoryRefreshKind::everything()),
    );

    let total_memory = system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0; // in GB
    let used_memory = system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0; // in GB
    let memory_usage = used_memory / total_memory * 100.0;

    Json(MemoryInfo {
        total_memory,
        used_memory,
        memory_usage,
    })
}

#[derive(Serialize)]
struct Storage {
    name: String,
    disk_type: String,
    total: f64,
    available: f64,
    usage: f64,
    mount_point: String,
    file_system: String,
}

#[get("/storage")]
async fn storage() -> Json<Vec<Storage>> {
    let disks = Disks::new_with_refreshed_list();
    let disk_usage = disks
        .iter()
        .filter(|disk| {
            disk.file_system().to_str().unwrap_or("ext3") == "ext4"
                && !disk
                    .mount_point()
                    .to_str()
                    .unwrap_or("/var")
                    .starts_with("/var")
        })
        .map(|disk| {
            let available = disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0; // in GB
            let total = disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0; // in GB

            Storage {
                name: disk.name().to_str().unwrap_or("N/A").to_string(),
                disk_type: disk.kind().to_string(),
                total,
                available,
                usage: (total - available) / total * 100.0,
                mount_point: disk.mount_point().to_str().unwrap_or("N/A").to_string(),
                file_system: disk.file_system().to_str().unwrap_or("N/A").to_string(),
            }
        })
        .collect();
    Json(disk_usage)
}

#[derive(FromForm)]
struct ImageUpload<'r> {
    image: TempFile<'r>,
}

#[post("/upload", data = "<form>", format = "multipart/form-data", rank = 1)]
async fn upload(mut form: Form<ImageUpload<'_>>) -> Result<String, std::io::Error> {
    let filename = match form.image.name() {
        Some(name) => name.to_string(),
        None => "image.jpeg".to_string(),
    };

    let content_type = match form.image.content_type() {
        Some(content_type) => content_type
            .media_type()
            .extension()
            .unwrap_or("mkv".into())
            .to_string(),
        None => "mkv".to_string(),
    };

    let path = Path::new("/mnt/sdb1").join(format!("{}.{}", &filename, &content_type));
    form.image.persist_to(&path).await?;

    let mut perms = fs::metadata(&path)?.permissions();
    perms.set_mode(0o744); //rwxr--r-- read write execute for owner, read for others
    fs::set_permissions(&path, perms)?;
    Ok(format!(
        "{} uploaded successfully. With content type: {}",
        filename, content_type
    ))
}

#[launch]
fn rocket() -> _ {
    let cors = rocket_cors::CorsOptions::default().to_cors().unwrap();
    let config = rocket::config::Config {
        address: std::net::Ipv4Addr::new(0, 0, 0, 0).into(),
        temp_dir: "/mnt/sdb1/temp".into(),
        limits: Limits::new()
            .limit("file", 100.gigabytes())
            .limit("image", 100.megabytes())
            .limit("data-form", 100.gigabytes()),
        ..Default::default()
    };

    rocket::build()
        .configure(config)
        .attach(cors)
        .mount("/api", routes![cpu, memory, storage, upload])
}
