use arrayvec::ArrayVec;
use spin::Once;

use crate::arch::vcpu::VCpu;
use crate::{HyperCraftHal, HyperResult, HyperError, HostPhysAddr, HostVirtAddr, GuestPhysAddr};


/// The maximum number of CPUs we can support.
pub const MAX_CPUS: usize = 4;

pub const VCPUS_MAX: usize = MAX_CPUS;

/// The set of vCPUs in a VM.
#[derive(Default)]
pub struct VCpusArray{
    inner: [Once<VCpu>; VCPUS_MAX],
}

impl VCpusArray {
    /// Creates a new vCPU tracking structure.
    pub fn new() -> Self {
        Self {
            inner: [Once::INIT; VCPUS_MAX],
            marker: core::marker::PhantomData,
        }
    }

    /// Adds the given vCPU to the set of vCPUs.
    pub fn add_vcpu(&mut self, vcpu: VCpu) -> HyperResult<()> {
        let vcpu_id = vcpu.vcpu_id();
        let once_entry = self.inner.get(vcpu_id).ok_or(HyperError::BadState)?;

        once_entry.call_once(|| vcpu);
        Ok(())
    }

    /// Returns a reference to the vCPU with `vcpu_id` if it exists.
    pub fn get_vcpu(&mut self, vcpu_id: usize) -> HyperResult<&mut VCpu<H>> {
        let vcpu = self
            .inner
            .get_mut(vcpu_id)
            .and_then(|once| once.get_mut())
            .ok_or(HyperError::NotFound)?;
        Ok(vcpu)
    }
}

// Safety: Each VCpu is wrapped with a Mutex to provide safe concurrent access to VCpu.
unsafe impl Sync for VCpusArray {}
unsafe impl Send for VCpusArray {}
