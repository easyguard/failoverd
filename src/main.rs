const LAN_INTERFACE: &str = "eth1"; // TODO: Make this configurable
const ROUTER_IP: &str = "10.10.99.1"; // TODO: Make this configurable

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

#[tokio::main]
async fn main() {
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
		// .arg("-R")
		.spawn()
		.expect("Failed to run DHCP client!");
	let dhcp_client_status = dhcp_client.wait().expect("Failed to wait for DHCP client!");
	if dhcp_client_status.success() {
		println!("[failoverd] DHCP server found on LAN interface: {}", LAN_INTERFACE);
		// Need to start the failover routing process
		failover().await;
	} else {
		println!("[failoverd] No DHCP server found on LAN interface: {}", LAN_INTERFACE);
		interface.set_up(false).expect("Failed to bring down LAN interface!");
		// Need to start the regular routing process
		routing(); // This will never return
	}
}

fn routing() -> ! {
	start_service("networking");
	start_service("dhcpd");
	println!("We are in routing mode!");
	loop {}
}

async fn failover() -> ! {
	let ping_payload = [0; 8];
	loop {
		let ping = surge_ping::ping(ROUTER_IP.parse().unwrap(), &ping_payload).await;

		if ping.is_err() {
			println!("Router did not respond to ping: {:?}", ping.err());
			tokio::time::sleep(std::time::Duration::from_secs(1)).await;

			let ping = surge_ping::ping(ROUTER_IP.parse().unwrap(), &ping_payload).await;

			if ping.is_ok() {
				println!("Router responded: {:?}", ping.unwrap().1.as_millis());
				tokio::time::sleep(std::time::Duration::from_secs(5)).await;
				continue;
			}

			println!("Router still did not respond to ping: {:?}. Assuming it is dead.", ping.err());

			let mut interface = interfaces::Interface::get_by_name(LAN_INTERFACE).unwrap().expect("LAN interface not found! Is the interface name correct?");
			interface.set_up(false).expect("Failed to bring down LAN interface!");
			routing();
		}

		let ping = ping.unwrap();

		println!("Ping to Router: {}ms", ping.1.as_millis());

		tokio::time::sleep(std::time::Duration::from_secs(10)).await;
	}
}
