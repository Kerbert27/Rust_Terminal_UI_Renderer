use burn::{
    data::{dataloader::batcher::Batcher, dataset::vision::MnistItem},
    prelude::*,
};


#[derive(Clone, Debug)]
pub struct PitchBatch<B: Backend> {
    pub inputs: Tensor<B, 2>,
    pub pitch_types: Tensor<B, 1, Int>,
    pub velocities: Tensor<B, 2>,
    pub zones: Tensor<B, 1, Int>,
}


#[derive(Clone, Default)]
pub struct PitchBatcher;

impl<B: Backend> Batcher<B, PitchExample, PitchBatch<B>> for PitchBatcher {
    fn batch(
        &self,
        items: Vec<PitchExample>,
        device: &B::Device,
    ) -> PitchBatch<B> {
        let inputs = items
            .iter()
            .map(|item| item.input.to_vec())
            .collect::<Vec<_>>();

        let pitch_types = items
            .iter()
            .map(|item| item.pitch_type as i64)
            .collect::<Vec<_>>();

        let velocities = items
            .iter()
            .map(|item| vec![item.velocity])
            .collect::<Vec<_>>();

        let zones = items
            .iter()
            .map(|item| item.zone as i64)
            .collect::<Vec<_>>();

        PitchBatch {
            inputs: Tensor::<B, 2>::from_floats(
                inputs,
                device,
            ),

            pitch_types: Tensor::<B, 1, Int>::from_ints(
                pitch_types,
                device,
            ),

            velocities: Tensor::<B, 2>::from_floats(
                velocities,
                device,
            ),

            zones: Tensor::<B, 1, Int>::from_ints(
                zones,
                device,
            ),
        }
    }
}
