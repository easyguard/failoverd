const LAN_INTERFACE: &str = "eth1"; // TODO: Make this configurable

fn start_service(name: &str) {
	println!("[failoverd] Starting service: {}", name);
	let mut status = std::process::Command::new("rc-service")
		.arg(name)
		.arg("start")
		.spawn()
		.expect("Failed to start service!");
	let status = status.wait().expect("Failed to wait for service!");
	if status.success() {
		println!("[failoverd] Service started: {}", name);
	} else {
		println!("[failoverd] Failed to start service: {}", name);
	}
}

fn main() {
	println!("[failoverd] Starting!");
	// Bring up the LAN interface
	println!("[failoverd] Bringing up LAN interface: {}", LAN_INTERFACE);
	let mut interface = interfaces::Interface::get_by_name(LAN_INTERFACE).unwrap().expect("LAN interface not found! Is the interface name correct?");
	interface.set_up(true).expect("Failed to bring up LAN interface!");
	// Check if a DHCP server is running on the LAN interface
	println!("[failoverd] Checking for DHCP server on LAN interface: {}", LAN_INTERFACE);
	let mut dhcp_client = std::process::Command::new("udhcpc")
		.arg("-i")
		.arg(LAN_INTERFACE)
		.arg("-n")
		.arg("-q")
		.arg("-R")
		.spawn()
		.expect("Failed to run DHCP client!");
	let dhcp_client_status = dhcp_client.wait().expect("Failed to wait for DHCP client!");
	interface.set_up(false).expect("Failed to bring down LAN interface!");
	if dhcp_client_status.success() {
		println!("[failoverd] DHCP server found on LAN interface: {}", LAN_INTERFACE);
		// Need to start the failover routing process
		// TODO
		unimplemented!();
	} else {
		println!("[failoverd] No DHCP server found on LAN interface: {}", LAN_INTERFACE);
		// Need to start the regular routing process
		start_service("networking");
		start_service("dhcpd");
	}
}
