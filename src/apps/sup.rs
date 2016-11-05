extern crate hwloc;

use hwloc::Topology;

fn main() {
    let topo = Topology::new();

    println!("CPU Binding (current process) supported: {}", topo.support().cpu().set_current_process());
    println!("CPU Binding (any process) supported: {}", topo.support().cpu().set_process());

    println!("CPU Binding (current thread) supported: {}", topo.support().cpu().set_current_thread());
    println!("CPU Binding (any thread) supported: {}", topo.support().cpu().set_thread());

    println!("Memory Binding supported: {}", topo.support().memory().set_current_process());

    println!("All Flags:\n{:?}", topo.support());
}
