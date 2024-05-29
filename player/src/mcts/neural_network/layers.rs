#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use rand::{rngs::SmallRng, Rng as _};

pub struct DenseLayer {
    weights: Vec<f32>,
    biases: Vec<f32>,
    input_size: usize,
    output_size: usize,
}

impl DenseLayer {
    pub fn new(input_size: usize, output_size: usize) -> Self {
        assert!(input_size % 8 == 0, "Input size must be a multiple of 8.");
        assert!(output_size % 8 == 0, "Output size must be a multiple of 8.");

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
            *weight = rng.gen_range(-1.0..1.0);
        }

        for bias in self.biases.iter_mut() {
            *bias = rng.gen_range(-1.0..1.0);
        }
    }

    pub fn weights(&self) -> &[f32] {
        &self.weights
    }

    pub fn biases(&self) -> &[f32] {
        &self.biases
    }

    pub fn set_weights(&mut self, weights: &[f32]) {
        self.weights.copy_from_slice(weights);
    }

    pub fn set_biases(&mut self, biases: &[f32]) {
        self.biases.copy_from_slice(biases);
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

    pub fn weights(&self) -> &[f32] {
        &self.weights
    }

    pub fn biases(&self) -> &[f32; N] {
        &self.biases
    }

    pub fn set_weights(&mut self, weights: &[f32]) {
        assert_eq!(weights.len(), self.input_size * N);
        // Transpose the weights
        for i in 0..self.input_size {
            for j in 0..N {
                self.weights[j * self.input_size + i] = weights[i * N + j];
            }
        }
        self.reset();
    }

    pub fn set_biases(&mut self, biases: &[f32]) {
        assert_eq!(biases.len(), N);
        self.biases.copy_from_slice(biases);
        self.reset();
    }
}

pub trait InputLayer {
    fn set_input(&mut self, index: usize);
    fn unset_input(&mut self, index: usize);
    fn reset(&mut self);
    fn output(&self) -> &[f32];
}

impl<const N: usize> InputLayer for EfficentlyUpdatableDenseLayer<N> {
    fn set_input(&mut self, index: usize) {
        #[cfg(debug_assertions)]
        {
            assert!(index < self.input_size);
        }

        #[cfg(target_arch = "x86_64")]
        unsafe {
            let weight_slice = &self.weights[..];
            let weight_ptr = weight_slice.as_ptr();

            for i in (0..N).step_by(8) {
                let weight_vec: __m256 = _mm256_loadu_ps(weight_ptr.add(index * N + i));
                let output_vec: __m256 = _mm256_loadu_ps(&self.output[i]);
                let new_output_vec: __m256 = _mm256_add_ps(output_vec, weight_vec);
                _mm256_storeu_ps(&mut self.output[i], new_output_vec);
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            for i in 0..N {
                self.output[i] += self.weights[index * N + i];
            }
        }
    }

    fn unset_input(&mut self, index: usize) {
        #[cfg(debug_assertions)]
        {
            assert!(index < self.input_size);
        }

        #[cfg(target_arch = "x86_64")]
        unsafe {
            let weight_slice = &self.weights[..];
            let weight_ptr = weight_slice.as_ptr();

            for i in (0..N).step_by(8) {
                let weight_vec: __m256 = _mm256_loadu_ps(weight_ptr.add(index * N + i));
                let output_vec: __m256 = _mm256_loadu_ps(&self.output[i]);
                let new_output_vec: __m256 = _mm256_sub_ps(output_vec, weight_vec);
                _mm256_storeu_ps(&mut self.output[i], new_output_vec);
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            for i in 0..N {
                self.output[i] -= self.weights[index * N + i];
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
