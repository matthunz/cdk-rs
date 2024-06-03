use clap::Parser;
use clap::Subcommand;
use flate2::read::GzDecoder;
use std::env::current_dir;
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;
use tar::Archive;
use tokio::runtime::Runtime;
use zip::ZipArchive;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Build {
        #[arg(long)]
        example: Option<String>,
    },
    #[command(name = "ls")]
    List,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Build { example } => build(example),
        Commands::List => list(),
    }
}

fn build(example: Option<String>) {
    let zip = Runtime::new().unwrap().block_on(async move {
        reqwest::get(
            "https://github.com/aws/aws-cdk/releases/download/v2.144.0/aws-cdk-2.144.0.zip",
        )
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap()
    });

    let mut archive = ZipArchive::new(Cursor::new(&zip[..])).unwrap();

    let target_dir = "cdk-target";

    let mut tmp_dir = PathBuf::from(target_dir);
    tmp_dir.push("tmp");
    archive.extract(&tmp_dir).unwrap();

    tmp_dir.push("js");
    tmp_dir.push("aws-cdk-lib@2.144.0.jsii.tgz");

    let data = File::open(&tmp_dir).unwrap();

    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd.arg("build");

    if let Some(ref example) = example {
        cargo_cmd.arg("--example");
        cargo_cmd.arg(example);
    }

    cargo_cmd.spawn().unwrap().wait().unwrap();

    match fs::create_dir(target_dir) {
        Ok(_) => {
            let gz_decoder = GzDecoder::new(data);

            let mut archive = Archive::new(gz_decoder);
            archive.unpack(target_dir).unwrap();
        }
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
        Err(e) => todo!("{:?}", e),
    }

    let mut file_path = PathBuf::from(target_dir);
    file_path.push("worker.js");
    let worker_js = include_str!("../assets/worker.js");
    fs::write(&file_path, worker_js).unwrap();

    let mut file_path = PathBuf::from(target_dir);
    file_path.push("package.json");
    let package_json = include_str!("../assets/package.json");
    fs::write(&file_path, package_json).unwrap();

    let mut file_path = PathBuf::from(target_dir);
    file_path.push("cdk.json");
    let package_json = include_str!("../assets/cdk.json");
    fs::write(&file_path, package_json).unwrap();

    Command::new("npm.cmd")
        .arg("install")
        .current_dir(&target_dir)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    let mut p = PathBuf::from("target");
    p.push("debug");
    p.push("examples");
    p.push("app.exe");

    let mut output_path = PathBuf::from(target_dir);
    output_path.push("app.exe");

    fs::copy(p, output_path).unwrap();
}

fn list() {
    let mut path = current_dir().unwrap();
    path.push("cdk-target");

    Command::new("npm.cmd")
        .arg("run")
        .arg("cdk")
        .arg("ls")
        .current_dir(&path)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}
