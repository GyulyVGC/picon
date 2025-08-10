use std::collections::HashSet;

// use bsd_kvm::{Access, KernProc, Kvm};
use crate::Listener;

pub(crate) fn get_icon_by_path() -> crate::Result<HashSet<Listener>> {
    Err("This OS isn't supported yet".into())
}
