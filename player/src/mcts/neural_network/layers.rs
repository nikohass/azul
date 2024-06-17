#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use ndarray::{Array1, Array2};
use rand::{rngs::SmallRng, Rng as _};

pub fn apply_relu<const N: usize>(input: &[f32], output: &mut [f32; N]) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        debug_assert!(N % 8 == 0, "Input size must be a multiple of 8.");
        debug_assert!(input.len() == N);

        let zero_vec = _mm256_set1_ps(0.0);

        for i in (0..N).step_by(8) {
            // Load 8 floats from the input array
            let input_vec = _mm256_loadu_ps(input.as_ptr().add(i));

            // Apply ReLU: max(input, 0.0)
            let result_vec = _mm256_max_ps(input_vec, zero_vec);

            // Store the result back to the output array
            _mm256_storeu_ps(output.as_mut_ptr().add(i), result_vec);
        }
    }
}

pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

pub trait Layer<T>
where
    T: Copy,
{
    fn weights(&self) -> Array2<T>;
    fn biases(&self) -> Array1<T>;
    fn set_weights(&mut self, weights: Array2<T>);
    fn set_biases(&mut self, biases: &Array1<T>);
}

pub trait InputLayer {
    fn set_input(&mut self, index: usize);
    fn unset_input(&mut self, index: usize);
    fn reset(&mut self);
    fn output(&self) -> &[f32];
}

pub struct DenseLayer {
    weights: Vec<f32>,
    biases: Vec<f32>,
    input_size: usize,
    output_size: usize,
}

impl DenseLayer {
    pub fn new(input_size: usize, output_size: usize) -> Self {
        assert!(input_size % 8 == 0, "Input size must be a multiple of 8.");

        let weights = vec![0.0; input_size * output_size];
        let biases = vec![0.0; output_size];

        DenseLayer {
            weights,
            biases,
            input_size,
            output_size,
        }
    }

    pub fn forward(&self, input: &[f32], output: &mut [f32]) {
        debug_assert!(
            input.len() == self.input_size,
            "Input size: {}, Should be: {}",
            input.len(),
            self.input_size
        );
        debug_assert!(output.len() == self.output_size);

        #[cfg(target_arch = "x86_64")]
        unsafe {
            for (i, output_val) in output.iter_mut().enumerate().take(self.output_size) {
                let mut sum = _mm256_setzero_ps();

                for j in (0..self.input_size).step_by(8) {
                    let input_vec: __m256 = _mm256_loadu_ps(&input[j]);
                    let weight_vec: __m256 =
                        _mm256_loadu_ps(&self.weights[i * self.input_size + j]);
                    sum = _mm256_add_ps(sum, _mm256_mul_ps(input_vec, weight_vec));
                }

                let mut sum_arr = [0.0; 8];
                _mm256_storeu_ps(&mut sum_arr[0], sum);
                *output_val = sum_arr.iter().sum::<f32>() + self.biases[i];
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            for (i, output_val) in output.iter_mut().enumerate().take(self.output_size) {
                *output_val = self.biases[i]
                    + (0..self.input_size)
                        .map(|j| input[j] * self.weights[i * self.input_size + j])
                        .sum::<f32>();
            }
        }
    }

    pub fn initialize_random(&mut self, rng: &mut SmallRng) {
        for weight in self.weights.iter_mut() {
            *weight = rng.gen_range(-0.1..0.1);
        }

        for bias in self.biases.iter_mut() {
            *bias = rng.gen_range(-0.1..0.1);
        }
    }

    pub fn quantize(&mut self) -> QuantizedDenseLayer {
        let quantized_weights: Vec<i8> = self.weights.iter().map(|&w| quantize_i8(w)).collect();
        let quantized_biases: Vec<i32> = self.biases.iter().map(|&b| quantize_i32(b)).collect();

        QuantizedDenseLayer {
            input_size: self.input_size,
            output_size: self.output_size,
            weights: quantized_weights,
            biases: quantized_biases,
        }
    }
}

impl Layer<f32> for DenseLayer {
    fn weights(&self) -> Array2<f32> {
        Array2::from_shape_vec((self.input_size, self.output_size), self.weights.clone()).unwrap()
    }

    fn biases(&self) -> Array1<f32> {
        Array1::from(self.biases.clone())
    }

    fn set_weights(&mut self, weights: Array2<f32>) {
        assert_eq!(weights.shape(), [self.output_size, self.input_size]);
        self.weights.copy_from_slice(weights.as_slice().unwrap());
    }

    fn set_biases(&mut self, biases: &Array1<f32>) {
        assert_eq!(biases.len(), self.output_size);
        self.biases.copy_from_slice(biases.as_slice().unwrap());
    }
}

pub struct EfficentlyUpdatableDenseLayer<const N: usize> {
    weights: Vec<f32>,
    biases: [f32; N],
    input_size: usize,
    output: [f32; N],
}

impl<const N: usize> EfficentlyUpdatableDenseLayer<N> {
    pub fn new(input_size: usize) -> Self {
        assert!(input_size % 8 == 0, "Input size must be a multiple of 8.");
        assert!(N % 8 == 0, "Output size must be a multiple of 8.");

        let weights = vec![0.0; input_size * N];
        let biases = [0.0_f32; N];

        EfficentlyUpdatableDenseLayer {
            weights,
            biases,
            input_size,
            output: biases,
        }
    }

    pub fn initialize_random(&mut self, rng: &mut SmallRng) {
        for weight in self.weights.iter_mut() {
            *weight = rng.gen_range(-1.0..1.0);
        }

        for bias in self.biases.iter_mut() {
            *bias = rng.gen_range(-1.0..1.0);
        }

        self.reset();
    }
}

impl<const N: usize> InputLayer for EfficentlyUpdatableDenseLayer<N> {
    fn set_input(&mut self, index: usize) {
        debug_assert!(index < self.input_size);

        #[cfg(target_arch = "x86_64")]
        unsafe {
            for i in (0..N).step_by(8) {
                let weight_index = i + N * index;
                let output_chunk = _mm256_loadu_ps(self.output.as_ptr().add(i));
                let weights_chunk = _mm256_loadu_ps(self.weights.as_ptr().add(weight_index));
                let result_chunk = _mm256_add_ps(output_chunk, weights_chunk);
                _mm256_storeu_ps(self.output.as_mut_ptr().add(i), result_chunk);
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            for i in 0..N {
                self.output[i] += self.weights[i + N * index];
            }
        }
    }

    fn unset_input(&mut self, index: usize) {
        debug_assert!(index < self.input_size);

        #[cfg(target_arch = "x86_64")]
        unsafe {
            for i in (0..N).step_by(8) {
                let weight_index = i + N * index;
                let output_chunk = _mm256_loadu_ps(self.output.as_ptr().add(i));
                let weights_chunk = _mm256_loadu_ps(self.weights.as_ptr().add(weight_index));
                let result_chunk = _mm256_sub_ps(output_chunk, weights_chunk);
                _mm256_storeu_ps(self.output.as_mut_ptr().add(i), result_chunk);
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            for i in 0..N {
                self.output[i] -= self.weights[i + N * index];
            }
        }
    }

    fn reset(&mut self) {
        self.output = self.biases;
    }

    fn output(&self) -> &[f32] {
        &self.output
    }
}

impl<const N: usize> Layer<f32> for EfficentlyUpdatableDenseLayer<N> {
    fn weights(&self) -> Array2<f32> {
        Array2::from_shape_vec((N, self.input_size), self.weights.clone()).unwrap()
    }

    fn biases(&self) -> Array1<f32> {
        Array1::from(self.biases.to_vec())
    }

    fn set_weights(&mut self, weights: Array2<f32>) {
        let (rows, cols) = weights.dim();
        let mut transformed_weights = vec![0.0; rows * cols];

        // Transform weights to desired order
        for i in 0..rows {
            for j in 0..cols {
                transformed_weights[j * rows + i] = weights[(i, j)];
            }
        }

        assert_eq!(transformed_weights.len(), N * self.input_size);

        self.weights = transformed_weights;
    }

    fn set_biases(&mut self, biases: &Array1<f32>) {
        assert_eq!(biases.len(), N);
        self.biases.copy_from_slice(biases.as_slice().unwrap());
    }
}

pub struct QuantizedDenseLayer {
    weights: Vec<i8>,
    biases: Vec<i32>,
    input_size: usize,
    output_size: usize,
}

impl QuantizedDenseLayer {
    pub fn new(input_size: usize, output_size: usize) -> Self {
        assert!(input_size % 32 == 0, "Input size must be a multiple of 32.");

        let weights = vec![0; input_size * output_size];
        let biases = vec![0; output_size];

        QuantizedDenseLayer {
            weights,
            biases,
            input_size,
            output_size,
        }
    }

    pub fn forward(&self, input: &[i32], output: &mut [i32]) {
        debug_assert!(
            input.len() == self.input_size,
            "Input size: {}, Should be: {}",
            input.len(),
            self.input_size
        );

        // #[cfg(target_arch = "x86_64")]
        // unsafe {
        //     for (i, output_val) in output.iter_mut().enumerate().take(self.output_size) {
        //         let mut sum = _mm256_setzero_si256();

        //         for j in (0..self.input_size).step_by(32) {
        //             let input_vec = _mm256_loadu_si256(input[j..].as_ptr() as *const _);
        //             let weight_vec = _mm256_loadu_si256(
        //                 self.weights[i * self.input_size + j..].as_ptr() as *const _,
        //             );
        //             let product = _mm256_maddubs_epi16(input_vec, weight_vec);
        //             sum = _mm256_add_epi32(sum, _mm256_madd_epi16(product, _mm256_set1_epi16(1)));
        //         }

        //         let mut sum_arr = [0i32; 8];
        //         _mm256_storeu_si256(sum_arr.as_mut_ptr() as *mut _, sum);
        //         *output_val = sum_arr.iter().sum::<i32>() + self.biases[i];
        //     }
        // }

        // #[cfg(not(target_arch = "x86_64"))]
        {
            for (i, output_val) in output.iter_mut().enumerate().take(self.output_size) {
                *output_val = self.biases[i]
                    + (0..self.input_size)
                        .map(|j| input[j] * self.weights[i * self.input_size + j] as i32)
                        .sum::<i32>();
            }
        }
    }
}

// pub fn quantize(x: f32) -> i8 {
//     (x * 127.0).round() as i8
// }

// pub fn dequantize(x: i8) -> f32 {
//     x as f32 / 127.0
// }

const SCALE: f32 = 127.0 / 5.0;

pub fn quantize_i8(x: f32) -> i8 {
    (x * SCALE).round().clamp(-128.0, 127.0) as i8
}

pub fn dequantize_i8(x: i8) -> f32 {
    x as f32 / SCALE
}

pub fn quantize_i32(x: f32) -> i32 {
    (x * SCALE).round().clamp(-2147483648.0, 2147483647.0) as i32
}

pub fn dequantize_i32(x: i32) -> f32 {
    x as f32 / SCALE
}

impl Layer<i32> for QuantizedDenseLayer {
    fn weights(&self) -> Array2<i32> {
        Array2::from_shape_vec(
            (self.input_size, self.output_size),
            self.weights.iter().map(|&x| x as i32).collect(),
        )
        .unwrap()
    }

    fn biases(&self) -> Array1<i32> {
        Array1::from(self.biases.clone())
    }

    fn set_weights(&mut self, weights: Array2<i32>) {
        assert_eq!(weights.shape(), [self.output_size, self.input_size]);
        self.weights.copy_from_slice(
            weights
                .as_slice()
                .unwrap()
                .iter()
                .map(|&x| x as i8)
                .collect::<Vec<i8>>()
                .as_slice(),
        );
    }

    fn set_biases(&mut self, biases: &Array1<i32>) {
        assert_eq!(biases.len(), self.output_size);
        self.biases.copy_from_slice(biases.as_slice().unwrap());
    }
}
