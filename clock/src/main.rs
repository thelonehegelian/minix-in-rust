const CLOCK: isize = -3;
const TIMER_MODE: isize = 0x43;
const SQUARE_WAVE: isize = 0x36;
const TIMER0: isize = 0x40;
const TIMER_FREQUENCY: isize = 1193182; /* clock frequency for timer in PC and AT */
const HZ: isize = 60;	/* clock freq (software settable on IBM-PC) */
const TIMER_COUNT: usize = (TIMER_FREQUENCY / HZ ) as usize;
const OKAY: usize = 0;
const CLOCK_IRQ: isize = 0;



// NOTE not sure this will work
#[cfg(target_arch (x86))]
fn outb(port: usize, value: usize) {
    // inline assembly 
    unsafe {
        asm!("outb %al, %dx" :: "{al}"(value), "{dx}"(port) : "memory" : "volatile");
    }
}

fn sys_outb(port: usize, value: usize) -> Result<(), i32> {
    unimplemented!();
    Ok(())    
}

struct IrqHook {

    // NOTE  may want to consider using Rc<RefCell<IrqHook>> or Arc<Mutex<IrqHook>> 
    // to allow shared ownership and interior mutability
    next: Option<Box<IrqHook>>,
    handler: fn(&IrqHook) -> i32,
    irq: isize,
    id: isize,
    proc_nr: isize,
    notify_id: usize, // irq_id_t
    policy: usize,    // irq_policy_t
}

fn put_irq_handler(hook: &IrqHook, irq: isize, handler: fn(&IrqHook) -> i32) {
    // TODO
    let mut id = 0;
    let bitmap: usize = 0;
    let mut line: IrqHook = &irq_hooks[irq];
    
    if (*hook == *line) {
        return;
    }
    // mark id in use 
    bitmap |= 1 << hook.id;

    if (id == 0 ) {
        println!("Too many handlers for IRQ {}", irq);
        return;
    }
    hook.next = None;
    hook.handler = handler;
    hook.irq = irq;
    hook.id = id;
    line = &irq_hooks[irq];

    // TODO enable irq at the hardware level

}


fn clock_handler(hook: &IrqHook) -> i32 {
    // TODO
    return 0;
}

fn enable_irq(hook: &IrqHook) {
    if (hook.proc_nr == None) {
        // TODO enable irq at the hardware level
        return;
    }
    

}

fn init_clock() {
    clock_hook.proc_nr = CLOCK;

    outb(TIMER_MODE, SQUARE_WAVE);
    outb(TIMER0, TIMER_COUNT);
    outb(TIMER0, TIMER_COUNT >> 8);
    put_irq_handler(&clock_hook, CLOCK_IRQ, clock_handler);
    enable_irq(&clock_hook);
}

fn main() {
    init_clock();
}
