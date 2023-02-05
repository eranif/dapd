use crate::libdapd::InFlightRequest;
use std::collections::VecDeque;

pub struct InFlightRequestQueue {
    vec: VecDeque<InFlightRequest>,
}

impl InFlightRequestQueue {
    pub fn new() -> Self {
        InFlightRequestQueue {
            vec: VecDeque::<InFlightRequest>::new(),
        }
    }
}
