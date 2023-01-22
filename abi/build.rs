use std::process::Command;

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        // .emit_rerun_if_changed(true)
        .type_attribute("reservation.ReservationStatus", "#[derive(sqlx::Type)]")
        .type_attribute(
            "reservation.ReservationQuery",
            "#[derive(derive_builder::Builder)]",
        )
        .field_attribute(
            "reservation.ReservationQuery.start",
            "#[builder(setter(into, strip_option))]",
        )
        .field_attribute(
            "reservation.ReservationQuery.end",
            "#[builder(setter(into, strip_option))]",
        )
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();
    Command::new("cargo").args(["fmt"]).output().unwrap();

    println!("cargo:rerun-if-changed=protos/reservation.proto");
}
