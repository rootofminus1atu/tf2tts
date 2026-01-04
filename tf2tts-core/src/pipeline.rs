use tokio::sync::mpsc::{Sender, Receiver};

pub trait EventProcessor<I, O>: Send + Sync + 'static {
    fn process(&self, input: I) -> Option<O>;
}

impl<I, O, F> EventProcessor<I, O> for F
where
    F: Fn(I) -> Option<O> + Send + Sync + 'static,
{
    fn process(&self, input: I) -> Option<O> {
        (self)(input)
    }
}

pub struct EventNode<I, O, P: EventProcessor<I, O>> {
    pub input: Receiver<I>,
    pub outputs: Vec<Sender<O>>,
    pub processor: P,
}

impl<I: Send + 'static, O: Send + Clone + 'static, P: EventProcessor<I, O>> EventNode<I, O, P> {
    pub fn new(input: Receiver<I>, outputs: Vec<Sender<O>>, processor: P) -> Self {
        Self { input, outputs, processor }
    }

    pub async fn run(mut self) {
        while let Some(item) = self.input.recv().await {
            if let Some(out) = self.processor.process(item) {
                for sender in &self.outputs {
                    let _ = sender.send(out.clone()).await;
                }
            }
        }
    }
}