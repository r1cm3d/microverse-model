use candle_core::{Device, Result, Tensor};
use rand::Rng;
use std::fs;

pub struct Dataset {
    data: Vec<u8>,
    context_len: usize,
}

impl Dataset {
    pub fn from_file(path: &str, context_len: usize) -> anyhow::Result<Self> {
        let data = fs::read(path)?;
        Ok(Self { data, context_len })
    }

    pub fn random_batch(
        &self,
        batch_size: usize,
        rng: &mut impl Rng,
        device: &Device,
    ) -> Result<(Tensor, Tensor)> {
        let max_start = self.data.len() - self.context_len - 1;
        let mut inputs = Vec::with_capacity(batch_size * self.context_len);
        let mut targets = Vec::with_capacity(batch_size * self.context_len);

        for _ in 0..batch_size {
            let start = rng.gen_range(0..=max_start);
            for j in 0..self.context_len {
                inputs.push(self.data[start + j] as u32);
                targets.push(self.data[start + j + 1] as u32);
            }
        }

        let input_tensor = Tensor::from_vec(inputs, (batch_size, self.context_len), device)?;
        let target_tensor = Tensor::from_vec(targets, (batch_size, self.context_len), device)?;

        Ok((input_tensor, target_tensor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::fs;

    #[test]
    fn test_dataset_batch_shape() {
        let data: Vec<u8> = (0u8..=255).cycle().take(1000).collect();
        let path = "/tmp/microverse_test_dataset.bin";
        fs::write(path, &data).unwrap();
        let dataset = Dataset::from_file(path, 16).unwrap();
        let device = Device::Cpu;
        let mut rng = StdRng::seed_from_u64(0);
        let (inp, tgt) = dataset.random_batch(4, &mut rng, &device).unwrap();
        assert_eq!(inp.dims(), &[4, 16]);
        assert_eq!(tgt.dims(), &[4, 16]);
    }
}
