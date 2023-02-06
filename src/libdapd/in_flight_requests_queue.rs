use dap::requests::Request;
use std::collections::VecDeque;

pub struct InFlightRequestQueue {
    vec: VecDeque<Request>,
}

impl InFlightRequestQueue {
    pub fn new() -> Self {
        InFlightRequestQueue {
            vec: VecDeque::<Request>::new(),
        }
    }

    pub fn push(&mut self, request: dap::requests::Request) {
        self.vec.push_back(request);
    }
}
