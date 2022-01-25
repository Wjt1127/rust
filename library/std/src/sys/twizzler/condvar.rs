use crate::sync::atomic::{AtomicU64, Ordering};
use crate::sys::mutex::Mutex;
use crate::time::Duration;
use twizzler_abi::syscall::{
    sys_thread_sync, ThreadSync, ThreadSyncFlags, ThreadSyncOp, ThreadSyncReference,
    ThreadSyncSleep, ThreadSyncWake,
};

pub struct Condvar {
    count: AtomicU64,
}

pub type MovableCondvar = Condvar;

impl Condvar {
    pub const fn new() -> Condvar {
        Condvar { count: AtomicU64::new(0) }
    }

    #[inline]
    pub unsafe fn init(&mut self) {
        self.count = AtomicU64::new(0);
    }

    #[inline]
    pub unsafe fn notify_one(&self) {
        self.count.fetch_add(1, Ordering::SeqCst);
        let wake =
            ThreadSync::new_wake(ThreadSyncWake::new(ThreadSyncReference::Virtual(&self.count), 1));
        let _ = sys_thread_sync(&mut [wake], None);
    }

    #[inline]
    pub unsafe fn notify_all(&self) {
        self.count.fetch_add(1, Ordering::SeqCst);
        let wake = ThreadSync::new_wake(ThreadSyncWake::new(
            ThreadSyncReference::Virtual(&self.count),
            usize::MAX,
        ));
        let _ = sys_thread_sync(&mut [wake], None);
    }

    pub unsafe fn wait(&self, mutex: &Mutex) {
        let val = self.count.load(Ordering::SeqCst);
        mutex.unlock();
        let sleep = ThreadSync::new_sleep(ThreadSyncSleep::new(
            ThreadSyncReference::Virtual(&self.count),
            val,
            ThreadSyncOp::Equal,
            ThreadSyncFlags::empty(),
        ));
        let _ = sys_thread_sync(&mut [sleep], None);
        mutex.lock();
    }

    pub unsafe fn wait_timeout(&self, mutex: &Mutex, dur: Duration) -> bool {
        let val = self.count.load(Ordering::SeqCst);
        mutex.unlock();
        let sleep = ThreadSync::new_sleep(ThreadSyncSleep::new(
            ThreadSyncReference::Virtual(&self.count),
            val,
            ThreadSyncOp::Equal,
            ThreadSyncFlags::empty(),
        ));
        let res = sys_thread_sync(&mut [sleep], Some(dur));
        mutex.lock();
        res == Ok(false)
    }

    #[inline]
    pub unsafe fn destroy(&self) {}
}
