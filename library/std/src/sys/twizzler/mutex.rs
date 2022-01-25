use crate::cell::UnsafeCell;
use crate::sync::atomic::AtomicU64;
use crate::sync::atomic::Ordering;
use crate::thread::ThreadId;
use twizzler_abi::syscall::{
    sys_thread_sync, ThreadSync, ThreadSyncFlags, ThreadSyncOp, ThreadSyncReference,
    ThreadSyncSleep, ThreadSyncWake,
};

pub struct Mutex {
    lock: AtomicU64,
}

pub type MovableMutex = Box<Mutex>;

unsafe impl Send for Mutex {}

impl Mutex {
    pub const fn new() -> Mutex {
        Mutex { lock: AtomicU64::new(0) }
    }

    #[inline]
    pub unsafe fn init(&mut self) {
        self.lock = AtomicU64::new(0);
    }

    #[inline]
    pub unsafe fn lock(&self) {
        for _ in 0..100 {
            let result = self.lock.compare_exchange_weak(0, 1, Ordering::SeqCst, Ordering::SeqCst);
            if result.is_ok() {
                return;
            }
            core::hint::spin_loop();
        }
        let _ = self.lock.compare_exchange(1, 2, Ordering::SeqCst, Ordering::SeqCst);
        let sleep = ThreadSync::new_sleep(ThreadSyncSleep::new(
            ThreadSyncReference::Virtual(&self.lock),
            2,
            ThreadSyncOp::Equal,
            ThreadSyncFlags::empty(),
        ));
        loop {
            let state = self.lock.swap(2, Ordering::SeqCst);
            if state == 0 {
                break;
            }
            let _ = sys_thread_sync(&mut [sleep], None);
        }
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        if self.lock.swap(0, Ordering::SeqCst) == 1 {
            return;
        }
        for _ in 0..200 {
            if self.lock.load(Ordering::SeqCst) > 0 {
                if self.lock.compare_exchange(1, 2, Ordering::SeqCst, Ordering::SeqCst) != Err(0) {
                    return;
                }
            }
            core::hint::spin_loop();
        }
        let wake =
            ThreadSync::new_wake(ThreadSyncWake::new(ThreadSyncReference::Virtual(&self.lock), 1));
        let _ = sys_thread_sync(&mut [wake], None);
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        self.lock.compare_exchange_weak(0, 1, Ordering::SeqCst, Ordering::SeqCst).is_ok()
    }

    #[inline]
    pub unsafe fn destroy(&self) {}
}

struct ReentrantMutexInner {
    id: ThreadId,
    count: usize,
}

// All empty stubs because this platform does not yet support threads, so lock
// acquisition always succeeds.
pub struct ReentrantMutex {
    inner: UnsafeCell<ReentrantMutexInner>,
    lock: Mutex,
    shared_lock: Mutex,
}

impl ReentrantMutex {
    pub const unsafe fn uninitialized() -> ReentrantMutex {
        ReentrantMutex {
            inner: UnsafeCell::new(ReentrantMutexInner {
                id: crate::mem::transmute(crate::mem::MaybeUninit::<ThreadId>::uninit()),
                count: 0,
            }),
            lock: Mutex::new(),
            shared_lock: Mutex::new(),
        }
    }

    pub unsafe fn init(&mut self) {
        self.inner =
            UnsafeCell::new(ReentrantMutexInner { id: crate::thread::current().id(), count: 0 });
        self.lock = Mutex::new();
        self.shared_lock = Mutex::new();
    }

    pub unsafe fn lock(&self) {
        let myid = crate::thread::current().id();
        self.lock.lock();
        let inner = &mut *self.inner.get();
        if inner.id == myid {
            inner.count += 1;
        } else {
            self.lock.unlock();
            self.shared_lock.lock();
            self.lock.lock();
            inner.id = myid;
            inner.count = 1;
        }
        self.lock.unlock();
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        let myid = crate::thread::current().id();
        if !self.lock.try_lock() {
            return false;
        }
        let inner = &mut *self.inner.get();
        if inner.id == myid {
            inner.count += 1;
        } else {
            self.lock.unlock();
            if !self.shared_lock.try_lock() {
                return false;
            }
            self.lock.lock();
            inner.id = myid;
            inner.count = 1;
        }
        self.lock.unlock();
        true
    }

    pub unsafe fn unlock(&self) {
        self.lock.lock();
        let inner = &mut *self.inner.get();
        if inner.count > 1 {
            inner.count -= 1;
        } else {
            inner.count = 0;
            self.shared_lock.unlock();
        }
        self.lock.unlock();
    }

    pub unsafe fn destroy(&self) {}
}
