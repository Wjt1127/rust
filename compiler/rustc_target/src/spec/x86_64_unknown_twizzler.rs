use crate::spec::{StackProbeType, Target};

const LINKER_SCRIPT: &str = include_str!("./x86_64_unknown_twizzler_linker_script.ld");
pub fn target() -> Target {
    let mut base = super::twizzler_base::opts();
    base.cpu = "x86-64".to_string();
    base.max_atomic_width = Some(64);
    base.features = "+rdrnd,+rdseed".to_string();
    // don't use probe-stack=inline-asm until rust#83139 and rust#84667 are resolved
    base.stack_probes = StackProbeType::Call;
    base.link_script = Some(LINKER_SCRIPT.to_string());

    Target {
        llvm_target: "x86_64-unknown-twizzler".to_string(),
        pointer_width: 64,
        data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
            .to_string(),
        arch: "x86_64".to_string(),
        options: base,
    }
}
