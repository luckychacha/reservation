use std::process::Command;

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        .emit_rerun_if_changed(true)
        .type_attribute("reservation.ReservationStatus", "#[derive(sqlx::Type)]")
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();
    Command::new("cargo").args(["fmt"]).output().unwrap();

    println!("cargo:rerun-if-changed=protos/reservation.proto");
}
