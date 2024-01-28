use core::{
    mem::MaybeUninit,
    ptr::{addr_of_mut, NonNull},
    sync::atomic::{AtomicUsize, Ordering},
};

use spin::Lazy;
use x86_64::instructions::interrupts::without_interrupts;

pub static mut QUEUE_BUFFER: MaybeUninit<[UiEvent; 128]> = MaybeUninit::zeroed();
pub static mut UI_EVT_QUEUE: Lazy<UiEventQueue> = Lazy::new(|| UiEventQueue {
    events: unsafe { NonNull::new_unchecked(addr_of_mut!(QUEUE_BUFFER).cast()) },
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
});

pub fn push_event(event: UiEvent) {
    unsafe { UI_EVT_QUEUE.push(event) };
}

pub fn pop_event() -> Option<UiEvent> {
    unsafe { UI_EVT_QUEUE.pop() }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiEvent {
    ScrollDown,
    ScrollUp,
    WriteStr(&'static str),
}

const UI_QUEUE_SIZE: usize = 128;

pub struct UiEventQueue {
    pub events: NonNull<[UiEvent; UI_QUEUE_SIZE]>,
    pub head: AtomicUsize,
    pub tail: AtomicUsize,
}

impl UiEventQueue {
    pub fn push(&self, event: UiEvent) {
        without_interrupts(|| {
            let mut events = self.events;
            let head = self.head.load(Ordering::SeqCst);
            let tail = self.tail.load(Ordering::SeqCst);
            if (tail + 1) % UI_QUEUE_SIZE == head {
                return;
            }
            unsafe { events.as_mut()[tail] = event };
            self.tail
                .store((tail + 1) % UI_QUEUE_SIZE, Ordering::SeqCst);
        });
    }

    pub fn pop(&self) -> Option<UiEvent> {
        without_interrupts(|| {
            let events = self.events;
            let head = self.head.load(Ordering::SeqCst);
            let tail = self.tail.load(Ordering::SeqCst);
            if head == tail {
                return None;
            }
            let event = unsafe { events.as_ref()[head].clone() };
            self.head
                .store((head + 1) % UI_QUEUE_SIZE, Ordering::SeqCst);
            Some(event)
        })
    }
}
