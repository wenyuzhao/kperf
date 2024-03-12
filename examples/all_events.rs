use kperf::{Event, KPerf};

fn some_workload() {
    println!("workload start");
    let mut v = vec![];
    for i in 0..100000000 {
        v.push(i);
    }
    let s: usize = v.iter().sum();
    println!("result = {}", s);
    println!("workload end\n");
}

fn main() -> anyhow::Result<()> {
    let mut kperf: KPerf = KPerf::new()?;

    kperf.add_event(false, Event::Cycles)?;
    kperf.add_event(false, Event::Instructions)?;
    kperf.add_event(false, Event::Branches)?;
    kperf.add_event(false, Event::BranchMisses)?;

    kperf.start()?;

    some_workload();

    let counters = kperf.stop()?;

    for (event, value) in counters.iter() {
        println!("{}: {}", Into::<&str>::into(*event), value);
    }

    Ok(())
}
