use crate::{
    api::{AddEntriesRequest, SampleEntriesRequest, SetBufferSizeRequest},
    buffer::ReplayEntry,
};
use reqwest::blocking::Client;

pub struct ReplayBufferClient {
    client: Client,
    base_url: String,
}

impl ReplayBufferClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    pub fn add_entries(&self, entries: Vec<ReplayEntry>) -> Result<String, String> {
        let mut responses = Vec::new();
        const MAX_CHUNK_SIZE: usize = 100;

        for chunk in entries.chunks(MAX_CHUNK_SIZE) {
            let request = AddEntriesRequest {
                entries: chunk.to_vec(),
            };
            let response = self.send_add_entries_request(request)?;
            responses.push(response);
        }

        Ok("Entries added successfully".to_string())
    }

    pub fn sample_entries(&self, count: usize) -> Result<Vec<ReplayEntry>, String> {
        let request = SampleEntriesRequest { count };
        self.send_sample_entries_request(request)
    }

    pub fn set_buffer_size(&self, size: usize) -> Result<String, String> {
        let request = SetBufferSizeRequest { size };
        self.send_set_buffer_size_request(request)
    }

    fn send_add_entries_request(&self, request: AddEntriesRequest) -> Result<String, String> {
        let url = format!("{}/add_entries", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if response.status().is_success() {
            Ok("Entries added successfully".to_string())
        } else {
            Err(format!("Failed to add entries: {}", response.status()))
        }
    }

    fn send_sample_entries_request(
        &self,
        request: SampleEntriesRequest,
    ) -> Result<Vec<ReplayEntry>, String> {
        let url = format!("{}/sample_entries", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if response.status().is_success() {
            response
                .json::<Vec<ReplayEntry>>()
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            Err(format!("Failed to sample entries: {}", response.status()))
        }
    }

    fn send_set_buffer_size_request(
        &self,
        request: SetBufferSizeRequest,
    ) -> Result<String, String> {
        let url = format!("{}/set_buffer_size", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if response.status().is_success() {
            Ok("Buffer size set successfully".to_string())
        } else {
            Err(format!("Failed to set buffer size: {}", response.status()))
        }
    }
}
