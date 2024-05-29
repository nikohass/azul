// use ndarray::Array2;
// use pyo3::prelude::*;
// use replay_buffer::buffer::ReplayEntry;
// use replay_buffer::ReplayBufferClient;
// use std::sync::atomic::AtomicUsize;
// use std::sync::Mutex;

// #[pyclass]
// pub struct DataLoader {
//     local_buffer: Arc<Mutex<Vec<Array2<f32>>>>,
//     batch_size: usize,
//     target_buffer_size: Arc<AtomicUsize>,
// }

// #[pymethods]
// impl DataLoader {
//     #[new]
//     pub fn new(url: &str, batch_size: usize) -> Self {
//         let local_buffer = Arc::new(Mutex::new(Vec::new()));
//         let target_buffer_size = Arc::new(AtomicUsize::new(0));

//         let local_buffer_clone = local_buffer.clone();
//         let target_buffer_size_clone = target_buffer_size.clone();
//         thread::spawn(move || {
//             let client: ReplayBufferClient = todo!();

//             loop {
//                 let current_length;
//                 let target_length;
//                 {
//                     let mut lock = local_buffer_clone.lock().unwrap();
//                     current_length = lock.len();
//                     target_length = target_buffer_size_clone.load(Ordering::Relaxed);
//                 }
//                 if current_length < target_length {
//                     let entries = client.sample_entries(batch_size);
//                     let mut lock = local_buffer_clone.lock().unwrap();

//                 }
//             }
//         });

//         DataLoader {
//             local_buffer,
//             batch_size,
//             target_buffer_size,
//         }
//     }

//     pub fn next_batch(&mut self) -> Array2<f32> {
//         todo!();
//     }
// }
