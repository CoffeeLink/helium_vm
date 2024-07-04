/// enables the "Mounting" of IO;
#[derive(Debug)]
pub struct IOController;
// should probably work with callbacks, like IO.mount(addr_range, callback: Fn(addr, data))
// also, it should give a warning if 2 "Devices" "Collide" in the address range, but it shouldn't crash.
// so it should iterate over the callbacks for each address when called.
// also ever IO device should have its "update" function mounted too on a add_device_update(UpdateFN)
// this way the CPU should just be able to call UpdateIO after it finishes a "Cycle" (an instruction).
// Maybe make an IO Device trait, idk, it's just a small project.

