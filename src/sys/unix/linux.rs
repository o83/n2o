use libc;
use nix::sched::{self, CpuSet};

pub fn set_affinity(cpu_id: usize) {
    let id = unsafe { libc::pthread_self() as isize };
    let mut cpu = CpuSet::new();
    cpu.set(cpu_id);
    sched::sched_setaffinity(id, &cpu);
}