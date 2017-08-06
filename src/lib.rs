use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::Builder;

pub static EXIT : u8 = 123;
pub static OK : u8 = 321;
pub static DONE : u8 = 345;

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

            let (tx_done, rx_done) : (Sender<u8>, Receiver<u8>) = mpsc::channel();
            let (tx_party_thread, rx_party_thread) : (Sender<T>, Receiver<T>) = mpsc::channel();

            self.sender = Some(tx);
            self.stop_sender = Some(tx_done.clone());

            let builder : Builder = thread::Builder::new();


            let done = tx_done.clone();

            // Each thread will send its id via the channel
            self.thread = Some(builder.spawn(move || {

                loop {

                    let _stopper = rx_stop.recv().unwrap();

                    if _stopper == EXIT {
                        break

                    } else {
                        let _re = rx_party_thread.recv();
                        if _re.is_ok() {
                            _re.unwrap().execute();
                        }

                        done.send(DONE);
                    }
                }
            0

        }).unwrap());

        thread::spawn(move || {

            let mut free_to_process = true;

            loop {

                let mut v : Vec<T> = Vec::new();

                println!("Receiving ID");

                let result = rx_done.recv().unwrap();

                println!("Received: {}", result);

                if result == EXIT {
                    tx_stop.send(EXIT);
                    break;

                } else if result == DONE {
                    free_to_process = true;

                } else if result == OK {
                    v.push(rx.recv().unwrap());
                }

                if result != EXIT && free_to_process && v.len() > 0 {
                    println!("Sending on");
                    tx_stop.send(OK);
                    tx_party_thread.send(v.pop().unwrap());
                    free_to_process = false;

                }
            }
        });

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

    #[test]
    fn test_adding_a_task_to_the_queue_one_after_another() {
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
        q.send_task(Blah {});
        q.send_task(Blah {});
        q.send_task(Blah {});
        q.send_task(Blah {});
        q.send_task(Blah {});
        q.send_task(Blah {});
        q.stop();
        assert_eq!(q.is_running(), false);
        unsafe { assert!(TEST_BOOL); }
    }

}
