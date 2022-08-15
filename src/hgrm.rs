use std::{ops::Index, slice::Iter};

#[derive(Debug, PartialEq)]
pub struct HGRM {
    pub name: Option<String>,
    pub percentiles: Vec<Percentile>,
    pub mean: f64,
    pub std_deviation: f64,
    pub max: f64,
    pub total_count: u64,
    pub buckets: u64,
    pub sub_buckets: u64,
}

impl HGRM {
    pub fn new() -> Self {
        Self {
            name: None,
            percentiles: vec![],
            mean: 0.0,
            std_deviation: 0.0,
            max: 0.0,
            total_count: 0,
            buckets: 0,
            sub_buckets: 0,
        }
    }

    pub fn set_name(mut self, name: Option<&str>) -> Self {
        self.name = name.map(|s| s.to_string());
        self
    }

    pub fn set_mean(mut self, mean: f64) -> Self {
        self.mean = mean;
        self
    }

    pub fn set_std_deviation(mut self, std_deviation: f64) -> Self {
        self.std_deviation = std_deviation;
        self
    }

    pub fn set_max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    pub fn set_total_count(mut self, total_count: u64) -> Self {
        self.total_count = total_count;
        self
    }

    pub fn set_buckets(mut self, buckets: u64) -> Self {
        self.buckets = buckets;
        self
    }

    pub fn set_sub_buckets(mut self, sub_buckets: u64) -> Self {
        self.sub_buckets = sub_buckets;
        self
    }

    pub fn set_percentiles(mut self, percentiles: Vec<Percentile>) -> Self {
        self.percentiles = percentiles;
        self
    }

    #[allow(dead_code)]
    pub fn add_percentile(
        mut self,
        value: f64,
        percentile: f64,
        total_count: u64,
        one_percentile: OnePercentile,
    ) -> Self {
        self.percentiles.push(Percentile::new(
            value,
            percentile,
            total_count,
            one_percentile,
        ));
        self
    }

    pub fn max_latency(&self) -> f64 {
        self.percentiles
            .iter()
            .fold(f64::MIN, |a, b| a.max(b.value))
    }
}

#[derive(Debug, PartialEq)]
pub struct HGRMs(Vec<HGRM>);

impl HGRMs {
    pub fn new(hgrms: Vec<HGRM>) -> Self {
        Self(hgrms)
    }

    pub fn max_latency(&self) -> f64 {
        self.0.iter().fold(f64::MIN, |a, b| a.max(b.max_latency()))
    }

    pub fn iter(&self) -> Iter<HGRM> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Index<usize> for HGRMs {
    type Output = HGRM;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

impl IntoIterator for HGRMs {
    type Item = HGRM;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<HGRM> for HGRMs {
    fn from_iter<I: IntoIterator<Item = HGRM>>(iter: I) -> Self {
        let mut c = Vec::new();

        for i in iter {
            c.push(i);
        }

        HGRMs(c)
    }
}

#[derive(Debug, PartialEq)]
pub enum OnePercentile {
    Inf,
    Value(f64),
}

#[derive(Debug, PartialEq)]
pub struct Percentile {
    pub value: f64,
    pub percentile: f64,
    pub total_count: u64,
    pub one_percentile: OnePercentile,
}

impl Percentile {
    pub fn new(
        value: f64,
        percentile: f64,
        total_count: u64,
        one_percentile: OnePercentile,
    ) -> Self {
        Self {
            value,
            percentile,
            total_count,
            one_percentile,
        }
    }
}
