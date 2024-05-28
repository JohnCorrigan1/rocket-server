#[macro_use]
extern crate rocket;

//use rocket::data::{Data, ToByteUnit};
//use rocket::http::{ContentType, Status};
//use std::fs::File;
//use std::io::Write;
//use sys_info;
use sysinfo::{Cpu, CpuRefreshKind, DiskUsage, Disks, MemoryRefreshKind, System};

#[get("/cpu")]
async fn cpu() -> String {
    let system = System::new_with_specifics(
        sysinfo::RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
    );
    let cpu = system.global_cpu_info().cpu_usage();

    format!("{{cpuUsage: {:.2}}}", &cpu)
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
    rocket::build().mount("/api", routes![cpu, memory, storage])
}

//#[post("/upload", data = "<data>")]
