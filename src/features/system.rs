use sysinfo::{RefreshKind, System, SystemExt};

pub enum SystemType {
    Components,
    Cpu,
    Disks,
    DisksList,
    Everything,
    Memory,
    Networks,
    Processes,
    UsersList,
}

pub fn generate_serde_value(system_type: SystemType) {
    let mut system = System::new();

    match system_type {
        Components => {
            system.refresh_components();
            system.refresh_components_list();
            for component in system.get_components() {
                println!("{:#?}", component);
            }
        }

        /*
        Cpu => {
            system.refresh_cpu();
            for component in system.get_() {
                println!("{:#?}", component);
            }
        }*/
        Networks => {
            system.refresh_networks();
            system.refresh_networks_list();
            for network in system.get_networks() {
                println!("{:#?}", network);
            }
        }
    }
}
