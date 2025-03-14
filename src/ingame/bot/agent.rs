/*
use burn::{
    module::{
        Module,
        Param,
        ParamId,
    }, 
    nn::{
        Linear, LinearConfig,
    },
    tensor::{
        activation::relu,
        backend::Backend,
        Tensor
    }
};
use super::dqn::{
    ElemType,
    Model,
    DQN,
    DQNModel,
};

/* Agent Definition START */

#[derive(Module, Debug)]
pub struct Net<B: Backend> {
    layer1: Linear<B>,
    layer2: Linear<B>,
    layer3: Linear<B>,
}

impl<B: Backend> Default for Net<B> {
    fn default() -> Self {
        Self {
            layer1: LinearConfig::new(0, 0).init(&Default::default()),
            layer2: LinearConfig::new(0, 0).init(&Default::default()),
            layer3: LinearConfig::new(0, 0).init(&Default::default()),
        }
    }
}

impl<B: Backend> Net<B> {
    pub fn new(input_size: usize, dense_size: usize, output_soze: usize) -> Self {
        Self {
            layer1: LinearConfig::new(input_size, dense_size).init(&Default::default()),
            layer2: LinearConfig::new(dense_size, dense_size).init(&Default::default()),
            layer3: LinearConfig::new(dense_size, output_soze).init(&Default::default()),
        }
    }

    fn consume(self) -> (Linear<B>, Linear<B>, Linear<B>) {
        (self.layer1, self.layer2, self.layer3)
    }
}

impl<B: Backend> Model<B, Tensor<B, 2>, Tensor<B, 2>> for Net<B> {
    fn forward(&self, input: Tensor<B, 2>) -> Tensor<B, 2> {
        let layer_1_output = relu(self.layer1.forward(input));
        let layer_2_output = relu(self.layer2.forward(layer_1_output));

        relu(self.layer3.forward(layer_2_output))
    }

    fn infer(&self, input: Tensor<B, 2>) -> Tensor<B, 2> {
        self.forward(input)
    }
}

impl<B: Backend> DQNModel<B> for Net<B> {
    fn soft_update(this: Self, that: &Self, tau: ElemType) -> Self {
        let (linear1, linear2, linear3) = this.consume();

        Self {
            layer1: soft_update_linear(linear1, &that.layer1, tau),
            layer2: soft_update_linear(linear2, &that.layer2, tau),
            layer3: soft_update_linear(linear3, &that.layer3, tau),
        }
    }
}

#[allow(unused)]
const MEMORY_SIZE: usize = 4096;
const DENSE_SIZE: usize = 128;
const EPS_DECAY: f64 = 1000.0;
const EPS_START: f64 = 0.9;
const EPS_END: f64 = 0.05;

type FightAgent<E, B> = DQN<E, B, Net<B>>;

/* Agent Definition END */

/* Utils START */

fn soft_update_tensor<const N: usize, B: Backend>(
    this: &Param<Tensor<B, N>>,
    that: &Param<Tensor<B, N>>,
    tau: ElemType,
) -> Param<Tensor<B, N>> {
    let that_weight = that.val();
    let this_weight = this.val();
    let new_weight = this_weight * (1.0 - tau) + that_weight * tau;

    Param::initialized(ParamId::new(), new_weight)
}

pub fn soft_update_linear<B: Backend>(
    this: Linear<B>,
    that: &Linear<B>,
    tau: ElemType,
) -> Linear<B> {
    let weight = soft_update_tensor(&this.weight, &that.weight, tau);
    let bias = match (&this.bias, &that.bias) {
        (Some(this_bias), Some(that_bias)) => Some(soft_update_tensor(this_bias, that_bias, tau)),
        _ => None,
    };

    Linear::<B> { weight, bias }
}

/* Utils END */

*/

/* Rule Base Agent Definition START */

/* Rule Base Agent Definition END */