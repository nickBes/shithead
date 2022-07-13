use std::{
    collections::BinaryHeap,
    time::{Duration, Instant},
};

use futures::{future::BoxFuture, Future};
use tokio::sync::mpsc;

const MIN_TIMEOUT_DELAY: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub struct TimeoutScheduler {
    scheduler_task_handler: SchedulerTaskHandler,
}

impl TimeoutScheduler {
    /// Creates a new timeout scheduler.
    pub fn new() -> Self {
        Self {
            scheduler_task_handler: SchedulerTask::run(),
        }
    }

    /// Runs the given function after the given delay has passed.
    pub fn set_timeout<F, Fut>(&mut self, delay: Duration, f: F)
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let boxed_f = Box::pin(f());
        self.scheduler_task_handler
            .schedule_timeout(ScheduledTimeout {
                run_at: Instant::now() + delay,
                boxed_f,
            });
    }
}

/// A scheduled timeout. This implements ordering and equality traits according to the `run_at`
/// field.
struct ScheduledTimeout {
    run_at: Instant,
    boxed_f: BoxFuture<'static, ()>,
}

impl ScheduledTimeout {
    /// Returns the delay needed to wait for this scheduled timeout, if it is bigger than the
    /// minimum delay.
    pub fn get_delay(&self) -> Option<Duration> {
        let now = Instant::now();
        if self.run_at < now + MIN_TIMEOUT_DELAY {
            // run_at < now + min_delay
            // implies
            // run_at - now < min_delay
            //
            // which means that the delay we'll need to wait for is smaller than min_delay, so
            // return `None`
            None
        } else {
            Some(self.run_at - now)
        }
    }
}

impl std::fmt::Debug for ScheduledTimeout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScheduledTimeout")
            .field("run_at", &self.run_at)
            .field("boxed_f", &"...")
            .finish()
    }
}

impl PartialEq for ScheduledTimeout {
    fn eq(&self, other: &Self) -> bool {
        self.run_at.eq(&other.run_at)
    }
}

impl Eq for ScheduledTimeout {}

impl PartialOrd for ScheduledTimeout {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.run_at.partial_cmp(&other.run_at)
    }
}

impl Ord for ScheduledTimeout {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.run_at.cmp(&other.run_at)
    }
}

struct SchedulerTask {
    /// A heap of scheduled timeouts, ordered such that the timeout with the smallest
    /// [`ScheduledTimeout::run_at`] is at the top of the heap, thus the [`std::cmp::Reverse`].
    scheduled_timeouts: BinaryHeap<std::cmp::Reverse<ScheduledTimeout>>,

    /// A receiver for schedule requests
    schedule_channel_receiver: mpsc::UnboundedReceiver<ScheduleRequest>,

    /// The current delay that the scheduler should wait. This is always the smallest delay for
    /// the timeout which will be first to finish.
    cur_delay: Option<Duration>,
}

impl SchedulerTask {
    /// Creates a new scheduler task and runs it, returning a [`SchedulerTaskHandler`] for it.
    fn run() -> SchedulerTaskHandler {
        // create a communication channel with the scheduler task
        let (sender, receiver) = mpsc::unbounded_channel();

        let mut scheduler_task = SchedulerTask {
            scheduled_timeouts: BinaryHeap::new(),
            schedule_channel_receiver: receiver,
            cur_delay: None,
        };

        // spawn a task for the scheduler task
        tokio::spawn(async move {
            scheduler_task.main_loop().await;
        });

        SchedulerTaskHandler {
            schedule_channel_sender: sender,
        }
    }

    async fn main_loop(&mut self) {
        loop {
            match self.cur_delay {
                Some(cur_delay) => {
                    match tokio::time::timeout(cur_delay, self.schedule_channel_receiver.recv())
                        .await
                    {
                        Ok(recv_result) => {
                            // we received another request before the timeout has occured, update
                            // the current delay according to the new request.
                            let request = recv_result.unwrap();
                            self.handle_schedule_request(request).await;
                        }
                        Err(_) => {
                            // a timeout has occured, run the desired scheduled timeout.
                            // the timeout we are currently waiting for will always be at the top
                            // of the heap, since the heap is ordered such that the smallest run_at
                            // task is at the top.
                            //
                            // so basically we can just run the timeout at the top of the heap.
                            let run = self.scheduled_timeouts.pop().unwrap();
                            run.0.boxed_f.await;

                            // after removing and running the task, update the cur delay.
                            self.update_cur_delay().await;
                        }
                    }
                }
                None => {
                    // if there is no current delay, wait for a schedule request
                    let request = self.schedule_channel_receiver.recv().await.unwrap();
                    self.handle_schedule_request(request).await;
                }
            }
        }
    }

    /// handles the given schedule request and updates `[Self::cur_delay]` accoridingly.
    async fn handle_schedule_request(&mut self, request: ScheduleRequest) {
        // add the scheduled timeout
        self.scheduled_timeouts
            .push(std::cmp::Reverse(request.scheduled_timeout));

        // update the delay according to the current delay.
        self.update_cur_delay().await
    }

    /// Updates the current delay that the scheduler should wait to the delay of the smallest task.
    async fn update_cur_delay(&mut self) {
        // run in a loop because sometimes the delay of the timeout at the top of the heap will be
        // too small that we will just run it without waiting and we'll then have to examine the
        // next top of the heap.
        loop {
            // get the delay of the smallest timeout.
            let delay = match self.scheduled_timeouts.peek() {
                Some(smallest_timeout) => smallest_timeout.0.get_delay(),
                None => {
                    // if there are no timeouts to wait for, just set the delay to `None`.
                    self.cur_delay = None;
                    return;
                }
            };

            // check the delay of this timeout, if it is smaller than the minimum, run the
            // task and check the next one, otherwise
            match delay {
                Some(delay) => {
                    // if the delay of the smallest timeout is bigger than the minimum
                    // timeout, then we must wait for it, so set the current delay to it.
                    self.cur_delay = Some(delay);

                    // we found the smallest delay, so return.
                    return;
                }
                None => {
                    // if the delay of the smallest timeout is smaller than the minimum
                    // timeout, just run it and check the next smallest timeout.
                    let smallest_timeout_owned = self.scheduled_timeouts.pop().unwrap();

                    smallest_timeout_owned.0.boxed_f.await;
                }
            }
        }
    }
}

#[derive(Debug)]
struct SchedulerTaskHandler {
    schedule_channel_sender: mpsc::UnboundedSender<ScheduleRequest>,
}

impl SchedulerTaskHandler {
    fn schedule_timeout(&self, scheduled_timeout: ScheduledTimeout) {
        // this should never fail because the schedule task should never end.
        if self
            .schedule_channel_sender
            .send(ScheduleRequest { scheduled_timeout })
            .is_err()
        {
            panic!("scheduler task has stopped unexpectedly")
        }
    }
}

struct ScheduleRequest {
    scheduled_timeout: ScheduledTimeout,
}
