use spin::Mutex;
use spinlock::SpinNoIrq;

use arm_gic::gic_v2::{GicDistributor, GicHypervisorInterface, GicCpuInterface};
use arm_gic::GIC_LIST_REGS_NUM;

use crate::arch::utils::bit_extract;
use lazy_init::LazyInit;

/// GICD
pub static mut GICD: LazyInit<&SpinNoIrq<GicDistributor>>= LazyInit::new();
/// GICC
pub static mut GICC: LazyInit<&GicCpuInterface>= LazyInit::new();
/// GICH
pub static mut GICH: LazyInit<&GicHypervisorInterface>= LazyInit::new();

pub const GICD_BASE: usize = 0x08000000;
pub const GICC_BASE: usize = 0x08010000;
pub const GICH_BASE: usize = 0x08030000;
pub const GICV_BASE: usize = 0x08040000;

const GIC_SGIS_NUM: usize = 16;
// mask
const LR_VIRTIRQ_MASK: usize = 0x3ff;
const LR_PHYSIRQ_MASK: usize = 0x3ff << 10;

const LR_PENDING_BIT: u32 = 1 << 28;
const LR_HW_BIT: u32 = 1 << 31;

// GICC BITS
pub const GICC_CTLR_EN_BIT: usize = 0x1;
pub const GICC_CTLR_EOIMODENS_BIT: usize = 1 << 9;

pub static GIC_LRS_NUM: Mutex<usize> = Mutex::new(0);


pub fn gicc_get_current_irq() -> (usize, usize) {
    unsafe {
        if let Some(gicc) = GICC.try_get() {
            let iar = gicc.get_iar();
            let irq = iar as usize;
            // current_cpu().current_irq = irq;
            let id = bit_extract(irq, 0, 10);
            let src = bit_extract(irq, 10, 3);
            (id, src)
        } else {
            warn!("No available gicc for gicc_get_current_irq");
            (usize::MAX, usize::MAX)
        }
    }
}

pub fn interrupt_cpu_ipi_send(cpu_id: usize, ipi_id: usize) {
    if ipi_id < GIC_SGIS_NUM {
        unsafe {
            if let Some(gicd) = GICD.try_get_mut() {
                gicd.lock().send_sgi(cpu_id, ipi_id);
            } else {
                warn!("No available gicd in interrupt_cpu_ipi_send!");
            }
        }
    }
}

pub fn pending_irq() -> Option<usize> {
    unsafe {
        if let Some(gicc) = GICC.try_get() {
        let iar = gicc.get_iar();
        debug!("this is iar:{:#x}", iar);
        if iar >= 0x3fe {
            // spurious
            None
        } else {
            Some(iar as _)
        }
        } else {
            warn!("No available gicc in pending_irq!");
            None
        }
    }
}

pub fn deactivate_irq(irq_num: usize) {
    unsafe {
        if let Some(gicc) = GICC.try_get_mut() {
            gicc.set_eoi(irq_num as _);
        } else {
            warn!("No available gicc in deactivate_irq!");
        }
    }
    
}

pub fn inject_irq(irq_id: usize) {
    unsafe {
        if let Some(gich) = GICH.try_get_mut() {
            let elsr: u64 = (gich.get_elsr1() as u64) << 32 | gich.get_elsr0() as u64;
            let lr_num = gich.get_lrs_num();
            let mut lr_idx = -1 as isize;
            for i in 0..lr_num {
                if (1 << i) & elsr > 0 {
                    if lr_idx == -1 {
                        lr_idx = i as isize;
                    }
                    continue;
                }
    
                // overlap
                let _lr_val = gich.get_lr_by_idx(i) as usize;
                if (i & LR_VIRTIRQ_MASK) == irq_id {
                   return;
                }
            }
            debug!("To Inject IRQ {:#x}, find lr {}", irq_id, lr_idx);
            if lr_idx == -1 {
                return;
            } else {
                let mut val = 0;
    
                val = irq_id as u32;
                val |= LR_PENDING_BIT;
    
                if false
                /* sgi */
                {
                    todo!()
                } else {
                    val |= ((irq_id << 10) & LR_PHYSIRQ_MASK) as u32;
                    val |= LR_HW_BIT;
                }   
    
                debug!("To write lr {:#x} val {:#x}", lr_idx, val);
                gich.set_lr_by_idx(lr_idx as usize, val);
            }
        } else {
            warn!("No available gicc in deactivate_irq!");
        }
    }
    
}


#[repr(C)]
#[derive(Debug, Clone)]
pub struct GicState {
    pub saved_hcr: u32,
    saved_eisr: [u32; GIC_LIST_REGS_NUM / 32],
    saved_elrsr: [u32; GIC_LIST_REGS_NUM / 32],
    saved_apr: u32,
    pub saved_lr: [u32; GIC_LIST_REGS_NUM],
    pub saved_ctlr: u32,
}
impl GicState {
    pub fn default() -> GicState {
        GicState {
            saved_hcr: 0,
            saved_eisr: [0; GIC_LIST_REGS_NUM / 32],
            saved_elrsr: [0; GIC_LIST_REGS_NUM / 32],
            saved_apr: 0,
            saved_lr: [0; GIC_LIST_REGS_NUM],
            saved_ctlr: 0,
        }
    }

    pub fn save_state(&mut self) { 
        unsafe {
        if let Some(gich) = GICH.try_get() {
            self.saved_hcr = gich.get_hcr();
            self.saved_apr = gich.get_apr();
            // todo
            /* 
            for i in 0..(GIC_LIST_REGS_NUM / 32) {
                self.saved_eisr[i] = gich.get_eisr_by_idx(i);
                self.saved_elrsr[i] = gich.get_elrsr_by_idx(i);
            }*/
            for i in 0..gich.get_lrs_num() {
                if self.saved_elrsr[0] & 1 << i == 0 {
                    self.saved_lr[i] = gich.get_lr_by_idx(i);
                } else {
                    self.saved_lr[i] = 0;
                }
            }
        } else {
            warn!("No available gich in save_state!")
        }
        if let Some(gicc) = GICC.try_get() {
            self.saved_ctlr = gicc.get_ctlr();
        }else {
            warn!("No available gicc in save_state!")
        }            
        }
    }

    pub fn restore_state(&self) {
        unsafe {
        if let Some(gich) = GICH.try_get_mut() {
            gich.set_hcr(self.saved_hcr);
            gich.set_apr(self.saved_apr);
            for i in 0..gich.get_lrs_num() {
                gich.set_lr_by_idx(i, self.saved_lr[i]);
            }
        } else {
            warn!("No available gich in restore_state!")
        }
        if let Some(gicc) = GICC.try_get_mut() {
            gicc.set_ctlr(self.saved_ctlr);
        }else {
            warn!("No available gicc in restore_state!")
        }            
        }

    }

}

/* 
pub fn gicc_clear_current_irq(for_hypervisor: bool) {
    let irq = current_cpu().current_irq as u32;
    if irq == 0 {
        return;
    }
    if GICC.is_none() {
        warn!("No available GICC in gicc_clear_current_irq");
        return;
    }
    let gicc = GICC.unwrap();
    // let gicc = &GICC;
    gicc.set_eoi(irq);
    // gicc.EOIR.set(irq);
    if for_hypervisor {
        gicc.set_dir(irq);
    }
    let irq = 0;
    current_cpu().current_irq = irq;
}

pub fn gic_cpu_reset() {
    if GICC.is_none() {
        warn!("No available GICC in gic_cpu_reset");
        return;
    }
    if GICH.is_none() {
        warn!("No available GICH in gic_cpu_reset");
        return;
    }
    let gicc = GICC.unwrap();
    let gich = GICH.unwrap();
    gicc.init();
    gich.init();
}

pub fn gic_lrs() -> usize {
    *GIC_LRS_NUM.lock()
}

pub fn interrupt_arch_clear() {
    gic_cpu_reset();
    gicc_clear_current_irq(true);
}
*/
