use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::Builder;

pub static EXIT : u8 = 483717;
pub static OK : u8 = 849122;

pub trait Queueable {
    fn execute(&self);

    /// Get the UID. Default UID is
    /// 0 so there's no need to implemenet
    /// this unless you want to use it elsewhere.
    fn uid(&self) -> u8 {
        0
    }
}

// Internal structure
pub struct Queue <T : 'static + std::marker::Send + Queueable> {
    is_running: bool,
    should_stop: Arc<Mutex<bool>>,
    sender: Option<Sender<T>>,
    stop_sender: Option<Sender<u8>>,
    thread: Option<JoinHandle<u8>>,
    vec: Vec<T>

}

impl <T : std::marker::Send + Queueable> Queue <T> {

    /// Create a new instance of the queue
    ///
    /// # Example:
    ///
    /// ```
    ///
    /// use queue::{Queue, Queueable};
    ///
    /// struct Blah {};
    ///
    /// impl Queueable for Blah {
    ///     fn execute(&self) {
    ///         // Nothing
    ///     }
    /// };
    ///
    /// let mut q : Queue<Blah> = Queue::new();
    /// assert_eq!(q.is_running(), false);
    ///
    /// ```
    ///
    pub fn new() -> Self {

        Queue {
            is_running: false,
            sender: None,
            stop_sender: None,
            thread: None,
            vec: Vec::new(),
            should_stop: Arc::new(Mutex::new(false))
        }
    }

    /// Start the queue
    ///
    /// # Example:
    ///
    /// ```
    ///
    /// use queue::{Queue, Queueable};
    /// struct Blah {};
    /// impl Queueable for Blah {
    ///     fn execute(&self) {
    ///         // Nothing
    ///     }
    /// };
    ///
    /// let mut q : Queue< Blah>= Queue::new();
    /// q.start();
    /// assert!(q.is_running());
    /// q.stop();
    ///
    /// ```
    ///
    pub fn start(&mut self) {
            self.is_running = true;

            let (tx, rx): (Sender<T>, Receiver<T>) = mpsc::channel();
            let (tx_stop, rx_stop) : (Sender<u8>, Receiver<u8>) = mpsc::channel();

            self.sender = Some(tx);
            self.stop_sender = Some(tx_stop);

            let builder : Builder = thread::Builder::new();


            // Each thread will send its id via the channel
            self.thread = Some(builder.spawn(move || {

                loop {

                    let _stopper = rx_stop.recv().unwrap();

                    if _stopper == EXIT {
                        break

                    } else {

                        let _re = rx.recv();
                        if _re.is_ok() {
                            _re.unwrap().execute();
                        }
                    }
                }
            0

        }).unwrap());

    }

    /// Stop the queue.
    ///
    /// # Example:
    ///
    /// ```
    ///
    /// use queue::{Queue, Queueable};
    /// struct Blah {};
    /// impl Queueable for Blah {
    ///     fn execute(&self) {
    ///         // Nothing
    ///     }
    /// };
    ///
    /// let mut q  : Queue<Blah>= Queue::new();
    /// q.start();
    /// q.stop();
    /// assert_eq!(q.is_running(), false);
    /// ```
    ///
    pub fn stop(&mut self) {
        let sender = self.sender.clone().unwrap();
        let stopper = self.stop_sender.clone().unwrap();

        stopper.send(EXIT);

        // Take ownership of the thread and wait for it to close.
        if let Some(handle) = self.thread.take() {
            handle.join().expect("failed to join thread");
        }

        self.is_running = false;
    }

    /// Send a task to be processed by the tasking queue
    pub fn send_task(&mut self, task : T) {
        let sender = self.sender.clone().unwrap();
        self.stop_sender.clone().unwrap().send(OK);
        sender.send(task);
    }

    /// Returns whether or not the queue is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}


#[cfg(test)]
mod tests {
   use {Queue, Queueable};


    #[test]
    fn test_starting_queue() {
        struct Blah {};
         impl Queueable for Blah {
             fn execute(&self) {
                 // Nothing
             }
         };

         let mut q  : Queue<Blah>= Queue::new();
         q.start();
         q.stop();
         assert_eq!(q.is_running(), false);
    }

    static mut TEST_BOOL : bool = false;

    #[test]
    fn test_adding_a_task_to_the_queue() {
        struct Blah;
        impl Queueable for Blah {
            fn execute(&self) {
                unsafe {
                    TEST_BOOL = true;
                }
            }
        };

        let mut q  : Queue<Blah>= Queue::new();
        q.start();
        q.send_task(Blah {});
        q.stop();
        assert_eq!(q.is_running(), false);
        unsafe { assert!(TEST_BOOL); }
    }

}
