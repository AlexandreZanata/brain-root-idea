//! Opt-in release soak for the conversation core (test-only).
//!
//! Run with `cargo test --release soak -- --ignored --nocapture`. This measures
//! the core in-process (not the GUI) and prints raw numbers only; the standard
//! test suite ignores it so CI stays deterministic and short.

use std::sync::{mpsc, Arc, Barrier};
use std::time::{Duration, Instant};

use super::runner::ProviderRunner;
use super::runtime::ConversationSession;
use super::wire::ConversationSendRequest;
use crate::provider::contract::{
    Cancellation, CancellationReason, Completion, ModelId, ProviderRequest, StopReason,
    StreamEvent, PROVIDER_CONTRACT_VERSION,
};
use crate::provider::execution::CancellationToken;

const CONVERSATIONS: usize = 500;
const CANCELLATIONS: usize = 200;
const SECOND_BATCH: usize = 500;

fn input(message: &str) -> ConversationSendRequest {
    ConversationSendRequest {
        contract_version: PROVIDER_CONTRACT_VERSION,
        message: message.to_string(),
    }
}

fn rss_kib() -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            if let Some(value) = rest.split_whitespace().next() {
                return value.parse().unwrap_or(0);
            }
        }
    }
    0
}

fn threads() -> usize {
    std::fs::read_dir("/proc/self/task")
        .map(|entries| entries.count())
        .unwrap_or(0)
}

fn microseconds(samples: &mut [Duration]) -> (u128, u128, u128) {
    samples.sort();
    let median = samples[samples.len() / 2];
    let p95 = samples[(samples.len() as f64 * 0.95) as usize - 1];
    let max = samples[samples.len() - 1];
    (median.as_micros(), p95.as_micros(), max.as_micros())
}

struct StoppableRunner {
    entered: mpsc::Sender<()>,
    release: Arc<Barrier>,
}

impl ProviderRunner for StoppableRunner {
    fn model(&self) -> ModelId {
        ModelId::new("stoppable-soak").expect("static model is valid")
    }

    fn stream(
        &self,
        request: &ProviderRequest,
        cancel: &CancellationToken,
        emit: &mut dyn FnMut(StreamEvent),
    ) {
        emit(StreamEvent::Started);
        self.entered.send(()).expect("test receiver remains open");
        self.release.wait();
        if cancel.is_cancelled() {
            emit(StreamEvent::Cancelled {
                cancellation: Cancellation {
                    conversation: request.conversation.clone(),
                    reason: CancellationReason::UserRequested,
                },
            });
        } else {
            emit(StreamEvent::Completed {
                completion: Completion {
                    conversation: request.conversation.clone(),
                    stop_reason: StopReason::EndTurn,
                },
            });
        }
    }
}

fn run_conversations(session: &ConversationSession, count: usize) {
    for index in 0..count {
        let (_, worker) = session
            .start(input(&format!("soak prompt {index}")), |_| {})
            .expect("request is accepted");
        worker.join().expect("worker joins");
        assert!(!session.is_active());
    }
}

#[test]
#[ignore = "release soak: run with cargo test --release soak -- --ignored --nocapture"]
fn fake_conversation_soak_returns_to_baseline() {
    let rss_before = rss_kib();
    let threads_before = threads();

    let session = ConversationSession::debug_fake();
    let conversations_started = Instant::now();
    run_conversations(&session, CONVERSATIONS);
    let conversations_elapsed = conversations_started.elapsed();
    let rss_after_conversations = rss_kib();

    let (entered_sender, entered_receiver) = mpsc::channel();
    let release = Arc::new(Barrier::new(2));
    let cancellable = ConversationSession::with_runner(Arc::new(StoppableRunner {
        entered: entered_sender,
        release: Arc::clone(&release),
    }));
    let mut dispatch_latencies = Vec::with_capacity(CANCELLATIONS);
    let mut cycle_latencies = Vec::with_capacity(CANCELLATIONS);
    let cancellations_started = Instant::now();
    for _ in 0..CANCELLATIONS {
        let (_, worker) = cancellable
            .start(input("soak cancel"), |_| {})
            .expect("request is accepted");
        entered_receiver.recv().expect("worker started");
        let cycle_started = Instant::now();
        cancellable.cancel().expect("cancel is accepted");
        let dispatch = cycle_started.elapsed();
        release.wait();
        worker.join().expect("worker joins");
        dispatch_latencies.push(dispatch);
        cycle_latencies.push(cycle_started.elapsed());
        assert!(!cancellable.is_active());
    }
    let cancellations_elapsed = cancellations_started.elapsed();
    let rss_after_cancellations = rss_kib();

    run_conversations(&session, SECOND_BATCH);
    let rss_after_second_batch = rss_kib();
    let threads_after = threads();

    let (dispatch_median, dispatch_p95, dispatch_max) = microseconds(&mut dispatch_latencies);
    let (cycle_median, cycle_p95, cycle_max) = microseconds(&mut cycle_latencies);

    println!(
        "soak: conversations={CONVERSATIONS} cancelled={CANCELLATIONS} second_batch={SECOND_BATCH} \
         rss_before_kib={rss_before} rss_after_conversations_kib={rss_after_conversations} \
         rss_after_cancellations_kib={rss_after_cancellations} rss_after_second_batch_kib={rss_after_second_batch} \
         threads_before={threads_before} threads_after={threads_after} \
         cancel_dispatch_us_median={dispatch_median} cancel_dispatch_us_p95={dispatch_p95} cancel_dispatch_us_max={dispatch_max} \
         cancel_cycle_us_median={cycle_median} cancel_cycle_us_p95={cycle_p95} cancel_cycle_us_max={cycle_max} \
         conversations_ms={} cancellations_total_ms={}",
        conversations_elapsed.as_millis(),
        cancellations_elapsed.as_millis(),
    );

    assert!(!session.is_active());
    assert!(!cancellable.is_active());
    assert!(
        threads_after <= threads_before + 2,
        "worker threads must not accumulate: before={threads_before} after={threads_after}"
    );
}
