use std::net::SocketAddr;

use crate::buffer::ReplayEntry;
use crate::BUFFER;
use serde::{Deserialize, Serialize};
use warp::Filter;

#[derive(Deserialize, Serialize)]
pub struct AddEntriesRequest {
    pub entries: Vec<ReplayEntry>,
}

#[derive(Deserialize, Serialize)]
pub struct SampleEntriesRequest {
    pub count: usize,
}

#[derive(Deserialize, Serialize)]
pub struct SetBufferSizeRequest {
    pub size: usize,
}

pub async fn run_rest_api(addr: SocketAddr) {
    let add_entries_route = warp::path("add_entries")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_add_entries);

    let sample_entries_route = warp::path("sample_entries")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_sample_entries);

    let set_buffer_size_route = warp::path("set_buffer_size")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_set_buffer_size);

    let routes = add_entries_route
        .or(sample_entries_route)
        .or(set_buffer_size_route);

    warp::serve(routes).run(addr).await;
}

async fn handle_add_entries(body: AddEntriesRequest) -> Result<impl warp::Reply, warp::Rejection> {
    let mut buffer = BUFFER.lock().await;
    buffer.add_entries(body.entries);
    Ok(warp::reply::json(&"Entries added successfully"))
}

async fn handle_sample_entries(
    body: SampleEntriesRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    let start_time = std::time::Instant::now();
    let buffer = BUFFER.lock().await;
    let entries = buffer.sample_n_entries(body.count, &mut rand::thread_rng());
    let elapsed = start_time.elapsed();
    println!("Sampled entries in {:?}", elapsed);
    Ok(warp::reply::json(&entries))
}

async fn handle_set_buffer_size(
    body: SetBufferSizeRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut buffer = BUFFER.lock().await;
    buffer.set_max_size(body.size);
    Ok(warp::reply::json(&"Buffer size set successfully"))
}
