use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Event {
    Cycles,
    Instructions,
    Branches,
    BranchMisses,
}

impl Event {
    #[cfg(target_arch = "x86_64")]
    pub(crate) fn get_internal_name(&self) -> &'static str {
        match self {
            Event::Cycles => "CPU_CLK_UNHALTED.THREAD\0",
            Event::Instructions => "INST_RETIRED.ANY\0",
            Event::Branches => "BR_INST_RETIRED.ALL_BRANCHES\0",
            Event::BranchMisses => "BR_MISP_RETIRED.ALL_BRANCHES\0",
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub(crate) fn get_internal_name(&self) -> &'static str {
        match self {
            Event::Cycles => "FIXED_CYCLES\0",
            Event::Instructions => "FIXED_INSTRUCTIONS\0",
            Event::Branches => "INST_BRANCH\0",
            Event::BranchMisses => "BRANCH_MISPRED_NONSPEC\0",
        }
    }
}

impl FromStr for Event {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cycles" => Ok(Event::Cycles),
            "instructions" => Ok(Event::Instructions),
            "branches" => Ok(Event::Branches),
            "banch-misses" => Ok(Event::BranchMisses),
            _ => Err(()),
        }
    }
}

impl From<&'static str> for Event {
    fn from(s: &'static str) -> Self {
        match s {
            "cycles" => Event::Cycles,
            "instructions" => Event::Instructions,
            "branches" => Event::Branches,
            "branch-misses" => Event::BranchMisses,
            _ => panic!("invalid event name"),
        }
    }
}

impl From<Event> for &'static str {
    fn from(event: Event) -> &'static str {
        match event {
            Event::Cycles => "cycles",
            Event::Instructions => "instructions",
            Event::Branches => "branches",
            Event::BranchMisses => "branch-misses",
        }
    }
}
