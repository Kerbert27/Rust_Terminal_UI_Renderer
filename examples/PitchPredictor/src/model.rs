use burn::{
    nn::{
        conv::{Conv2d, Conv2dConfig},
        pool::{AdaptiveAvgPool2d, AdaptiveAvgPool2dConfig},
        Dropout, DropoutConfig, Linear, LinearConfig, Relu,
    },
    prelude::*,
};


#[derive(Module, Debug)]
pub struct PitchModel<B: Backend> {
    layer1: Linear<B>,
    layer2: Linear<B>,

    pitch_type: Linear<B>,
    velocity: Linear<B>,
    zone: Linear<B>,
}

#[derive(Config, Debug)]
pub struct ModelConfig {
    num_classes: usize,
    hidden_size: usize,
    #[config(default = "0.5")]
    dropout: f64,
}

#[derive(Debug)]
pub struct PitchOutput<B: Backend> {
    pub pitch_type: Tensor<B, 2>,
    pub velocity: Tensor<B, 2>,
    pub zone: Tensor<B, 2>,
}


impl<B: Backend> PitchModel<B> {
    pub fn new(
        device: &B::Device,
        num_pitch_types: usize,
    ) -> Self {
        Self {
            layer1: LinearConfig::new(18, 128)
                .init(device),

            layer2: LinearConfig::new(128, 64)
                .init(device),

            pitch_type: LinearConfig::new(
                64,
                num_pitch_types,
            )
            .init(device),

            velocity: LinearConfig::new(64, 1)
                .init(device),

            zone: LinearConfig::new(64, 17)
                .init(device),
        }
    }


    pub fn forward(&self, input: Tensor<B, 2>) -> PitchOutput<B> {
        let x = self.layer1.forward(input);
        let x = Relu::new().forward(x);

        let x = self.layer2.forward(x);
        let x = Relu::new().forward(x);

        PitchOutput {
            pitch_type: self.pitch_type.forward(x.clone()),
            velocity: self.velocity.forward(x.clone()),
            zone: self.zone.forward(x),
        }
    }
}
