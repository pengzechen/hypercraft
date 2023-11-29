use arm_gic::gic_v2::{GicDistributor, GicHypervisorInterface, GicCpuInterface};
use crate::arch::gic::*;
/// el2 irq handler
#[no_mangle]
pub extern "C" fn irq_aarch64_el2() {
    debug!("IRQ routed to EL2");
    let (src, id) = gicc_get_current_irq();
    debug!("src {:#x} id{:#x}", src, id);
    if let Some(irq_id) = pending_irq() {
        // deactivate_irq(irq_id);
        inject_irq(irq_id);
    }
}
