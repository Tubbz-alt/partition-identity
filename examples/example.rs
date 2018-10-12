extern crate partition_identity;

use partition_identity::{PartitionID, PartitionIDVariant};
use std::env;
use std::process::exit;

fn main() {
    let mut args = env::args().skip(1);
    match args.next() {
        Some(arg) => match arg.as_str() {
            "from-path" => {
                let mut first = true;
                for device in args {
                    if ! first { println!() }
                    first = false;
                    println!("{}:", device);
                    println!("ID: {:?}", PartitionID::by_id(PartitionIDVariant::ID, &device));
                    println!("Label: {:?}", PartitionID::by_id(PartitionIDVariant::Label, &device));
                    println!("PartLabel: {:?}", PartitionID::by_id(PartitionIDVariant::PartLabel, &device));
                    println!("PartUUID: {:?}", PartitionID::by_id(PartitionIDVariant::PartUUID, &device));
                    println!("Path: {:?}", PartitionID::by_id(PartitionIDVariant::Path, &device));
                    println!("UUID: {:?}", PartitionID::by_id(PartitionIDVariant::UUID, &device));
                }
            }
            "by-uuid" => {
                for id in args {
                    let var = PartitionID { variant: PartitionIDVariant::UUID, id: id.clone() };
                    println!("{}: {:?}", id, var.from_id());
                }
            }
            "by-partuuid" => {
                for id in args {
                    let var = PartitionID { variant: PartitionIDVariant::PartUUID, id: id.clone() };
                    println!("{}: {:?}", id, var.from_id());
                }
            }
            _ => {
                eprintln!("invalid subcommand: valid commansd: [from-path, by-uuid, by-partuuid, ]");
                exit(1);
            }
        }
        None => {
            eprintln!("must give subcommand: [from-path, by-uuid, by-partuuid, ]");
            exit(1);
        }
    }
}