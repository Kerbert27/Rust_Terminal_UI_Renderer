use crate::{data::*, model::*};


impl<B: AutodiffBackend> PitchModel<B> {
    pub fn forward_train(
        &self,
        batch: PitchBatch<B>,
    ) -> PitchOutput<B> {
        self.forward(batch.inputs)
    }
}
