use std::{
    fmt::Debug,
    marker::PhantomData
};
use bevy_rapier2d::na::FullPivLU;
use burn::{
    grad_clipping::GradientClippingConfig, module::{AutodiffModule, Module}, nn::loss::{MseLoss, Reduction}, optim::{GradientsParams, Optimizer}, record::{BinGzFileRecorder, FullPrecisionSettings}, tensor::{
        backend::{
            AutodiffBackend,
            Backend
        }, BasicOps, Int, Tensor, TensorKind
    }
};
use rand::{
    random,
    Rng
};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

/* Base Definition START */

pub type ElemType = f32;
type LearningRate = f64;

pub trait Model<B: Backend, T, TrainingOutput, InferenceOutput = TrainingOutput> {
    fn forward(&self, input: T) -> TrainingOutput;
    fn infer(&self, input: T) -> InferenceOutput;
}

pub trait Action: Debug + Copy + Clone + From<u32> + Into<u32> {
    fn random() -> Self {
        rand::rng().random_range(0..Self::size() as u32).into()
    }

    fn enumerate() -> Vec<Self>;

    fn size() -> usize {
        Self::enumerate().len()
    }
}

pub trait State: Debug + Copy + Clone {
    type Data;
    
    fn to_tensor<B: Backend>(&self) -> Tensor<B, 1>;

    fn size() -> usize;
}

pub trait Environment: Debug {
    type StateType: State;
    type ActionType: Action;
    type RewardType: Debug + Clone + Into<ElemType>;

    const MAX_STEPS: usize = usize::MAX;

    fn new(visualized: bool) -> Self;

    fn state(&self) -> Self::StateType;

    fn reset(&mut self) -> Snapshot<Self>;

    fn step(&mut self, action: Self::ActionType) -> Snapshot<Self>;
}

#[derive(Debug)]
pub struct Snapshot<E: Environment + ?Sized> {
    pub state: E::StateType,
    pub reward: E::RewardType,
    pub done: bool,
}

impl<E: Environment> Snapshot<E> {
    pub fn new(state: E::StateType, reward: E::RewardType, done: bool) -> Self {
        Self { state, reward, done }
    }

    pub fn state(&self) -> &E::StateType {
        &self.state
    }

    pub fn reward(&self) -> &E::RewardType {
        &self.reward
    }

    pub fn done(&self) -> bool {
        self.done
    }
}

pub trait Agent<E: Environment> {
    fn react(&self, state: &E::StateType) -> Option<E::ActionType>;
}

pub type MemoryIndices = Vec<usize>;

pub fn sample_indices(indices: MemoryIndices, size: usize) -> MemoryIndices {
    let mut rng = rand::rng();
    let mut sampled_indices = Vec::with_capacity(size);
    let mut indices = indices.clone();

    for _ in 0..size {
        let index = rng.random_range(0..indices.len());
        sampled_indices.push(indices.remove(index));
    }

    sampled_indices
}

pub fn get_batch<B: Backend, const CAP: usize, T, K: TensorKind<B> + BasicOps<B>>(
    data: &ConstGenericRingBuffer<T, CAP>,
    indices: &MemoryIndices,
    converter: impl Fn(&T) -> Tensor<B, 1, K>
) -> Tensor<B, 2, K> {
    Tensor::cat(
        indices
            .iter()
            .filter_map(|&index| data.get(index))
            .map(converter)
            .collect::<Vec<_>>(),
            0
    )
    .reshape([indices.len() as i32, -1])
}

pub struct Memory<E: Environment, B: Backend, const CAP: usize> {
    state: ConstGenericRingBuffer<E::StateType, CAP>,
    next_state: ConstGenericRingBuffer<E::StateType, CAP>,
    action: ConstGenericRingBuffer<E::ActionType, CAP>,
    reward: ConstGenericRingBuffer<E::RewardType, CAP>,
    done: ConstGenericRingBuffer<bool, CAP>,
    environment: E,
    backend: B
}

impl<E: Environment, B: Backend, const CAP: usize> Memory<E, B, CAP> {
    fn default() -> Self {
        Self {
            state: ConstGenericRingBuffer::new(),
            next_state: ConstGenericRingBuffer::new(),
            action: ConstGenericRingBuffer::new(),
            reward: ConstGenericRingBuffer::new(),
            done: ConstGenericRingBuffer::new(),
            environment: E::new(false),
            backend: Default::default()
        }
    }
}

impl <E: Environment, B: Backend, const CAP: usize> Memory<E, B, CAP> {
    pub fn push(
        &mut self,
        state: E::StateType,
        next_state: E::StateType,
        action: E::ActionType,
        reward: E::RewardType,
        done: bool
    ) {
        self.state.push(state);
        self.next_state.push(next_state);
        self.action.push(action);
        self.reward.push(reward);
        self.done.push(done);
    }

    pub fn states(&self) -> &ConstGenericRingBuffer<E::StateType, CAP> {
        &self.state
    }

    pub fn next_states(&self) -> &ConstGenericRingBuffer<E::StateType, CAP> {
        &self.next_state
    }

    pub fn actions(&self) -> &ConstGenericRingBuffer<E::ActionType, CAP> {
        &self.action
    }

    pub fn rewards(&self) -> &ConstGenericRingBuffer<E::RewardType, CAP> {
        &self.reward
    }
    
    pub fn dones(&self) -> &ConstGenericRingBuffer<bool, CAP> {
        &self.done
    }

    pub fn len(&self) -> usize {
        self.state.len()
    }

    pub fn is_empty(&self) -> bool {
        self.state.is_empty()
    }

    pub fn clear(&mut self) {
        self.state.clear();
        self.next_state.clear();
        self.action.clear();
        self.reward.clear();
        self.done.clear();
    }
}

/* Base Definition END */

/* DQN Definition START */

pub trait DQNModel<B: Backend>: Model<B, Tensor<B, 2>, Tensor<B, 2>> {
    fn soft_update(this: Self, target_model: &Self, tau: ElemType) -> Self;
}

pub struct DQNTrainingConfig {
    pub gamma: ElemType,
    pub tau: ElemType,
    pub learning_rate: ElemType,
    pub batch_size: usize,
    pub clip_grad: Option<GradientClippingConfig>
}

impl Default for DQNTrainingConfig {
    fn default() -> Self {
        Self {
            gamma: 0.999,
            tau: 0.005,
            learning_rate: 0.001,
            batch_size: 32,
            clip_grad: Some(GradientClippingConfig::Value(100.0))
        }
    }
}

pub struct DQN<E: Environment, B: Backend, M: DQNModel<B> + Module<B>> {
    target_model: Option<M>,
    state: PhantomData<E::StateType>,
    action: PhantomData<E::ActionType>,
    backend: PhantomData<B>
}

impl<E: Environment, B: Backend, M: DQNModel<B> + Module<B>> Agent<E> for DQN<E, B, M> {
    fn react(&self, state: &E::StateType) -> Option<E::ActionType> {
        Some(convert_tensor_to_action::<E::ActionType, B>(
            self.target_model
                .as_ref()?
                .infer(ref_to_state_tensor(state).unsqueeze())
        ))
    }
}

impl<E: Environment, B: Backend, M: DQNModel<B> + Module<B> + Default> DQN<E, B, M> {
    pub fn new(model: M) -> Self {
        Self {
            target_model: Some(model),
            state: PhantomData,
            action: PhantomData,
            backend: PhantomData
        }
    }

    pub fn model(&self) -> &Option<M> {
        &self.target_model
    }

    pub fn save(&self, path: &str) {
        let recorder = BinGzFileRecorder::<FullPrecisionSettings>::new();
        self.target_model.clone().unwrap().save_file(path, &recorder).expect("failed to save model");
    }

    pub fn load(path: &str) -> Self {
        let recorder = BinGzFileRecorder::<FullPrecisionSettings>::new();
        Self::new(M::default().load_file(path, &recorder, &Default::default()).expect("failed to load model"))
    }
}

impl<E: Environment, B: AutodiffBackend, M: DQNModel<B> + Module<B>> DQN<E, B, M> {
    pub fn react_with_epsilon_greedy(
        policy_model: &M,
        state: E::StateType,
        eps_threshold: f64
    ) -> E::ActionType {
        if random::<f64>() > eps_threshold {
            convert_tensor_to_action::<E::ActionType, B>(
                policy_model.forward(to_state_tensor(state).unsqueeze())
            )
        } else {
            Action::random()
        }
    }
}

impl<E: Environment, B: AutodiffBackend, M: DQNModel<B> + AutodiffModule<B>> DQN<E, B, M> {
    pub fn train<const CAP: usize>(
        &mut self,
        mut policy_model: M,
        memory: &Memory<E, B, CAP>,
        optimizer: &mut (impl Optimizer<M, B> + Sized),
        config: &DQNTrainingConfig
    ) -> M {
        let sample_indices = sample_indices((0..memory.len()).collect(), config.batch_size);
        let state_batch = get_batch(memory.states(), &sample_indices, ref_to_state_tensor);
        let action_batch = get_batch(memory.actions(), &sample_indices, ref_to_action_tensor);
        let state_action_values = policy_model.forward(state_batch).gather(1, action_batch);

        let next_state_batch = 
            get_batch(memory.next_states(), &sample_indices, ref_to_state_tensor);
        let target_model = self.target_model.take().unwrap();
        let next_state_values = target_model.forward(next_state_batch).max_dim(1).unsqueeze();

        let not_done_batch = get_batch(memory.dones(), &sample_indices, ref_to_not_done_tensor);
        let reward_batch = get_batch(memory.rewards(), &sample_indices, ref_to_reward_tensor);

        let expected_state_action_values = 
            (next_state_values * not_done_batch).mul_scalar(config.gamma) + reward_batch;
        
        let loss = MseLoss.forward(
            state_action_values,
            expected_state_action_values,
            Reduction::Mean
        );

        policy_model = update_parameters(loss, policy_model, optimizer, config.learning_rate.into());

        self.target_model = Some(<M as DQNModel<B>>::soft_update(
            target_model,
            &policy_model,
            config.tau
        ));

        policy_model
    }
}

/* DQN Definition END */

/* Utils START */

fn to_state_tensor<S: State, B: Backend>(state: S) -> Tensor<B, 1> {
    state.to_tensor()
}

fn ref_to_state_tensor<S: State, B: Backend>(state: &S) -> Tensor<B, 1> {
    to_state_tensor(*state)
}

fn to_action_tensor<A: Action, B: Backend>(action: A) -> Tensor<B, 1, Int> {
    Tensor::<B, 1, Int>::from_ints([action.into() as i32], &Default::default())
}

fn ref_to_action_tensor<A: Action, B: Backend>(action: &A) -> Tensor<B, 1, Int> {
    to_action_tensor(*action)
}

fn to_reward_tensor<B: Backend>(reward: impl Into<ElemType> + Clone) -> Tensor<B, 1> {
    Tensor::from_floats([reward.into()], &Default::default())
}

fn ref_to_reward_tensor<B: Backend>(
    reward: &(impl Into<ElemType> + Clone)
) -> Tensor<B, 1> {
    to_reward_tensor(reward.clone())
}

fn to_ont_done_tensor<B: Backend>(done: bool) -> Tensor<B, 1> {
    Tensor::from_floats([if done {0.0} else {1.0}], &Default::default())
}

fn ref_to_not_done_tensor<B: Backend>(done: &bool) -> Tensor<B, 1> {
    to_ont_done_tensor(*done)
}

fn convert_tensor_to_action<A: Action, B: Backend>(tensor: Tensor<B, 2>) -> A {
    let action = tensor
        .argmax(1)
        .to_data()
        .as_slice::<i64>()
        .unwrap()[0] as u32;
    action.into()
}

fn update_parameters<B: AutodiffBackend, M: AutodiffModule<B>>(
    loss: Tensor<B, 1>,
    module: M,
    optimizer: &mut impl Optimizer<M, B>,
    learning_rate: LearningRate
) -> M {
    let gradients = loss.backward();
    let gradient_params = GradientsParams::from_grads(gradients, &module);
    optimizer.step(learning_rate, module, gradient_params)
}

/* Utils END */