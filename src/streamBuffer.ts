export type Scheduler = {
  schedule: (callback: () => void) => number;
  cancel: (handle: number) => void;
};

export type StreamBuffer = {
  push: (text: string) => void;
  flush: () => void;
  dispose: () => void;
  pendingBytes: () => number;
};

export const MAX_BUFFERED_BYTES = 64 * 1024;

const encoder = new TextEncoder();

export function createStreamBuffer(
  onFlush: (text: string) => void,
  scheduler: Scheduler,
  maxBufferedBytes = MAX_BUFFERED_BYTES
): StreamBuffer {
  let buffered = "";
  let bufferedBytes = 0;
  let handle: number | null = null;
  let disposed = false;

  function deliver() {
    if (handle !== null) {
      scheduler.cancel(handle);
      handle = null;
    }
    if (buffered.length === 0) {
      return;
    }
    const text = buffered;
    buffered = "";
    bufferedBytes = 0;
    onFlush(text);
  }

  return {
    push(text: string) {
      if (disposed || text.length === 0) {
        return;
      }
      buffered += text;
      bufferedBytes += encoder.encode(text).byteLength;
      if (bufferedBytes >= maxBufferedBytes) {
        deliver();
        return;
      }
      if (handle === null) {
        handle = scheduler.schedule(deliver);
      }
    },
    flush: deliver,
    dispose() {
      if (handle !== null) {
        scheduler.cancel(handle);
        handle = null;
      }
      buffered = "";
      bufferedBytes = 0;
      disposed = true;
    },
    pendingBytes() {
      return bufferedBytes;
    }
  };
}
