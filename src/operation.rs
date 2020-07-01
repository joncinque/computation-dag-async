use std::iter::{Product, Sum};
use std::time::Duration;
use std::fmt::Debug;

use tokio::time::delay_for;

#[derive(Clone, Debug)]
pub enum OperationType {
    Default,
    Delay,
    Sum,
    Product,
}

impl Default for OperationType {
    fn default() -> Self {
        OperationType::Default
    }
}

/// Convenience trait to avoid retyping all of the traits every time
pub trait Operable<'a, T: 'static>: Debug + Default + Clone + Product<&'a T> + Sum<&'a T> {}
impl<'a, T: Debug + Default + Clone + Product<&'a T> + Sum<&'a T> + 'static> Operable<'a, T> for T {}

#[derive(Clone)]
pub struct Operation {
    pub operation_type: OperationType,
}

impl Operation {
    pub async fn process<T>(&self, values: &Vec<T>) -> T
    where for<'a> T: Operable<'a, T> + 'static {
        match &self.operation_type {
            OperationType::Default => default(values).await,
            OperationType::Delay => delay(values).await,
            OperationType::Sum => sum(values).await,
            OperationType::Product => product(values).await,
        }
    }
}

pub async fn default<T>(_values: &Vec<T>) -> T
where T: Debug + Default + 'static {
    Default::default()
}

pub async fn delay<T>(_values: &Vec<T>) -> T
where T: Debug + Default + 'static {
    delay_for(Duration::from_secs(2)).await;
    Default::default()
}

pub async fn sum<T>(values: &Vec<T>) -> T
where for<'a> T: Debug + Sum<&'a T> + 'static {
    values.iter().sum()
}

pub async fn product<T>(values: &Vec<T>) -> T
where for<'a> T: Debug + Product<&'a T> + 'static {
    values.iter().product()
}

impl Default for Operation {
    fn default() -> Self {
        let operation_type: OperationType = Default::default();
        Operation { operation_type }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    pub async fn product_valid() {
        let operation_type = OperationType::Product;
        let operation = Operation { operation_type };
        let values = vec![1, 2, 3];
        let result = operation.process(&values).await;
        assert_eq!(result, 6);
    }

    #[tokio::test]
    pub async fn delay_valid() {
        let operation_type = OperationType::Delay;
        let operation = Operation { operation_type };
        let values = vec![1, 2, 3, 4, 5];
        let result = operation.process(&values).await;
        assert_eq!(result, 0);
    }

    #[tokio::test]
    pub async fn sum_valid() {
        let operation_type = OperationType::Sum;
        let operation = Operation { operation_type };
        let values = vec![1, 2, 3, 4, 5];
        let result = operation.process(&values).await;
        assert_eq!(result, 15);
    }

    #[tokio::test]
    pub async fn default_valid() {
        let operation_type = OperationType::Default;
        let operation = Operation { operation_type };
        let values = vec![1, 123125, 2];
        let result = operation.process(&values).await;
        assert_eq!(result, 0);
    }
}
